use std::collections::HashSet;

use crate::ir::{
    analysis::{
        call_optimize::call_optimize,
        dependent_analyse::dependency_check,
        scev::{scevexp::SCEVExp, SCEVAnalyzer},
    },
    instruction::InstKind,
};

use super::*;

pub fn auto_paralellization(
    module: &mut Module,
    loop_map: &mut HashMap<String, LoopList>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let call_op = call_optimize(module);
    func_process(module, |name, _| {
        // 遍历所有的循环，从循环外层向内层遍历
        let loop_list = loop_map.get(&name).unwrap();
        let mut op = HashSet::new();
        let mut unop = HashSet::new();
        let mut parallelized = HashSet::new();
        let mut analyzer = SCEVAnalyzer::new();
        analyzer.set_loop_list(loop_list.get_loop_list().clone());
        let mut new_loop_list = Vec::new();

        loop {
            let mut unvisited = false;
            for loop_info in loop_list.get_loop_list().iter() {
                if op.contains(loop_info) || unop.contains(loop_info) {
                    continue;
                }

                unvisited = true;
                // 从当前循环向外层遍历，找到最外层未检测过的循环
                let mut current_loop = *loop_info;
                while let Some(x) = current_loop.get_parent_loop() {
                    if !op.contains(&x) && !unop.contains(&x) {
                        current_loop = x;
                    } else {
                        break;
                    }
                }

                // 从最外层循环开始检测是否可并行化
                if check_parallelization(
                    current_loop,
                    &call_op,
                    &mut op,
                    &mut unop,
                    Vec::new(),
                    Vec::new(),
                    &mut analyzer,
                ) {
                    parallelized_insert(current_loop, &mut parallelized);
                    if let Some(new_loop) = parallelize(current_loop, pools) {
                        new_loop_list.push(new_loop);
                    }
                    analyzer.clear();
                }
            }

            if !unvisited {
                break;
            }
        }
        debug_assert!(op.is_disjoint(&unop));

        debug_assert!(
            loop_list
                .get_loop_list()
                .iter()
                .all(|x| op.contains(x) || unop.contains(x)),
            "loop not in op or unop: {:?}",
            loop_list
                .get_loop_list()
                .iter()
                .find(|x| !op.contains(x) && !unop.contains(x))
                .unwrap()
        );

        // 将能够并行但还未加入到并行化列表的循环加入到并行化列表
        op.iter().for_each(|x| {
            if !parallelized.contains(x) {
                let mut current_loop = *x;
                while let Some(x) = current_loop.get_parent_loop() {
                    if op.contains(&x) && !parallelized.contains(&x) {
                        current_loop = x;
                    } else {
                        break;
                    }
                }
                parallelized_insert(current_loop, &mut parallelized);
                parallelize(current_loop, pools);
                analyzer.clear();
            }
        });

        debug_assert_eq!(op, parallelized);
    });
}

fn parallelized_insert(loop_info: ObjPtr<LoopInfo>, parallelized: &mut HashSet<ObjPtr<LoopInfo>>) {
    let flag = parallelized.insert(loop_info);
    debug_assert!(flag);
    loop_info
        .get_sub_loops()
        .iter()
        .for_each(|x| parallelized_insert(*x, parallelized));
}

fn check_parallelization(
    mut current_loop: ObjPtr<LoopInfo>,
    call_op: &HashSet<String>,
    op: &mut HashSet<ObjPtr<LoopInfo>>,
    unop: &mut HashSet<ObjPtr<LoopInfo>>,
    iv: Vec<ObjPtr<SCEVExp>>,
    bound: Vec<[i32; 2]>,
    analyzer: &mut SCEVAnalyzer,
) -> bool {
    let mut read_map: HashMap<ObjPtr<Inst>, HashSet<_>> = HashMap::new();
    let mut write_map: HashMap<ObjPtr<Inst>, HashSet<_>> = HashMap::new();

    let get_gep_ptr = |inst: ObjPtr<Inst>| -> ObjPtr<Inst> {
        debug_assert_eq!(inst.get_kind(), InstKind::Gep);
        if inst.get_gep_ptr().get_kind() == InstKind::Load {
            inst.get_gep_ptr().get_ptr()
        } else {
            inst.get_gep_ptr()
        }
    };

    // 先检查当前循环
    let mut flag = true;
    current_loop.get_current_loop_bb().iter().for_each(|bb| {
        inst_process_in_bb(bb.get_head_inst(), |inst| {
            if !inst
                .get_use_list()
                .iter()
                .all(|user| current_loop.is_in_current_loop(&user.get_parent_bb()))
            {
                flag = false;
                return;
            }

            match inst.get_kind() {
                InstKind::Load => {
                    if inst.get_ptr().get_kind() == InstKind::Gep {
                        let gep = inst.get_ptr();
                        let array = get_gep_ptr(gep);
                        debug_assert!(
                            array.get_kind() == InstKind::Alloca(0)
                                || array.is_param() && array.get_ir_type().is_pointer()
                        );
                        if let Some(x) = read_map.get_mut(&array) {
                            x.insert(gep);
                        } else {
                            read_map.insert(array, HashSet::new());
                        }
                    }
                }
                InstKind::Store => {
                    // store全局变量不可并行化
                    if inst.get_dest().is_global_var() {
                        flag = false;
                        return;
                    }

                    let gep = inst.get_dest();
                    let array = get_gep_ptr(gep);
                    debug_assert!(
                        array.get_kind() == InstKind::Alloca(0)
                            || array.is_param() && array.get_ir_type().is_pointer()
                    );
                    if let Some(x) = write_map.get_mut(&array) {
                        x.insert(gep);
                    } else {
                        write_map.insert(array, HashSet::new());
                    }
                }
                InstKind::Call(callee) => {
                    if !call_op.contains(&callee) {
                        flag = false;
                        return;
                    }
                }
                _ => {}
            }
        });

        if !flag {
            return;
        }
    });

    if !flag {
        unop.insert(current_loop);
        return false;
    }

    // 找到当前循环的iv
    let mut iv = iv;
    let mut iv_set = HashSet::new();
    let mut inst = current_loop.get_header().get_head_inst();
    while let InstKind::Phi = inst.get_kind() {
        iv_set.insert(inst);
        inst = inst.get_next();
    }
    if iv_set.len() != 1 {
        unop.insert(current_loop);
        return false;
    }

    let new_iv = analyzer.analyze(iv_set.iter().next().unwrap());
    if !new_iv.is_scev_rec_expr() {
        unop.insert(current_loop);
        return false;
    }

    iv.push(new_iv);

    // 找到当前循环的bound
    let mut bound = bound;
    let mut new_bound = [0; 2];
    let start = new_iv.get_operands()[0];
    let exit = current_loop.get_exit_blocks();
    let mut end = None;
    if exit.len() == 1 {
        let end_cond = exit[0].get_tail_inst().get_br_cond();
        let temp;
        match (
            analyzer.analyze(&end_cond.get_lhs()).is_scev_rec(),
            analyzer.analyze(&end_cond.get_rhs()).is_scev_rec(),
        ) {
            (true, false) => {
                temp = end_cond.get_rhs();
            }
            (false, true) => {
                temp = end_cond.get_lhs();
            }
            _ => {
                unop.insert(current_loop);
                return false;
            }
        }
        if temp.is_const() {
            if temp.get_ir_type().is_int() {
                end = Some(temp.get_int_bond());
            } else {
                end = Some(temp.get_float_bond() as i32);
            }
        }
    }
    let inst = new_iv.get_bond_inst();
    debug_assert_eq!(inst.get_operands().len(), 2);
    let step = inst
        .get_operands()
        .iter()
        .find(|x| current_loop.is_in_current_loop(&x.get_parent_bb()))
        .cloned()
        .unwrap();
    match step.get_kind() {
        InstKind::Binary(crate::ir::instruction::BinOp::Add) => {
            if start.is_scev_constant() {
                new_bound[0] = start.get_scev_const();
            } else {
                new_bound[0] = i32::MIN;
            }
            new_bound[1] = end.unwrap_or(i32::MAX);
        }
        InstKind::Binary(crate::ir::instruction::BinOp::Sub) => {
            if start.is_scev_constant() {
                new_bound[1] = start.get_scev_const();
            } else {
                new_bound[1] = i32::MAX;
            }
            new_bound[0] = end.unwrap_or(i32::MIN);
        }
        _ => {
            debug_assert!(true, "step is not add or sub");
            unop.insert(current_loop);
            return false;
        }
    }
    bound.push(new_bound);

    // 检查当前循环是否有依赖
    // 1. 读写冲突
    for (array, read_set) in read_map.iter() {
        if let Some(write_set) = write_map.get(array) {
            for re in read_set.iter() {
                for wr in write_set.iter() {
                    if dependency_check(
                        [*re, *wr],
                        iv.iter()
                            .enumerate()
                            .map(|(i, x)| (x.clone(), bound[i]))
                            .collect(),
                    ) {
                        unop.insert(current_loop);
                        return false;
                    }
                }
            }
        }
    }

    // 2. 写写冲突
    for (array, write_set) in write_map.iter() {
        let write_set2 = write_map.get(array).unwrap();
        for wr in write_set.iter() {
            for wr2 in write_set2.iter() {
                if wr != wr2
                    && dependency_check(
                        [*wr, *wr2],
                        iv.iter()
                            .enumerate()
                            .map(|(i, x)| (x.clone(), bound[i]))
                            .collect(),
                    )
                {
                    unop.insert(current_loop);
                    return false;
                }
            }
        }
    }

    // 检查子循环是否可并行化
    for child in current_loop.get_sub_loops().iter() {
        if !check_parallelization(
            *child,
            call_op,
            op,
            unop,
            iv.clone(),
            bound.clone(),
            analyzer,
        ) {
            unop.insert(current_loop);
            return false;
        }
    }

    op.insert(current_loop);
    true
}

/// 进行自动并行化的ir修改
fn parallelize(
    mut current_loop: ObjPtr<LoopInfo>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> Option<LoopInfo> {
    let mut iv = current_loop.get_header().get_head_inst();
    debug_assert!(iv.is_phi() && !iv.get_next().is_phi());
    debug_assert_eq!(iv.get_operands().len(), 2);
    let index = iv
        .get_operands()
        .iter()
        .position(|x| {
            x.is_global_var_or_param() || !current_loop.is_in_current_loop(&x.get_parent_bb())
        })
        .unwrap();
    let start = iv.get_operand(index);
    let mut update = iv.get_operand(1 - index);
    let step_index = update.get_operands().iter().position(|x| *x != iv).unwrap();
    let mut step = update.get_operand(step_index);

    let (thread_loop, new_start) = thread_create_ir(current_loop, start, step, pools);

    iv.set_operand(new_start, index);
    let const_4 = pools.1.make_int_const(4);
    let new_step = pools.1.make_mul(step, const_4);
    step.insert_after(const_4);
    step.insert_after(new_step);
    update.set_operand(new_step, step_index);

    let exiting_blocks = current_loop.get_exit_blocks();
    for exiting_block in exiting_blocks {
        let exit_block = exiting_block
            .get_next_bb()
            .iter()
            .find(|x| !current_loop.is_in_loop(x))
            .unwrap()
            .clone();

        thread_exit_ir(exit_block, pools);
    }

    thread_loop
}

/// 申请线程的大致结构
///                             ┌────────────────────────────────┐
///                             │ Pre_header                     │
///                             │                                │
///                             └────────┬───────────────────────┘
///                                      │
///                                      │
///                                      │
///                             ┌────────▼───────────────────────┐
///                             │                                │
///                     ┌───────► i: phi 0 i_add                 │
///                     │       │                                │
///                     │       │ start: phi prestart start_add  │
///                     │       │                                │
///                     │       │ br i < 3                       │
///                     │       ├───────────────┬────────────────┤
///                     │       │   TRUE        │  FALSE         ├──────────────────►┌───────────────────────────────┐
///                     │       │               │                │                   │ jmp                           │
///                     │       └──────┬────────┴────────────────┘       ┌──────────►│                               │
///                     │              │                                 │           └───────────────┬───────────────┘
///                     │       ┌──────▼─────────────────────────┐       │                           │
///                     │       │                                │       │                           │
///                     │       │ thread_id: call thread_create()│       │                           │
///                     │       │                                │       │                           │
///                     │       │ br thread_id == 0              │       │           ┌───────────────▼────────────────┐
///                     │       │                                │       │           │                                │
///                     │       │                                │       │           │ Header                         │
///                     │       ├───────────────┬────────────────┤       │           │                                │
///                     │       │   TRUE        │  FALSE         ├───────┘           └────────────────────────────────┘
///                     │       │               │                │
///                     │       └──────┬────────┴────────────────┘
///                     │              │
///                     │              │
///                     │       ┌──────▼─────────────────────────┐
///                     │       │                                │
///                     │       │ start_add: add start step      │
///                     │       │                                │
///                     │       │ i_add: add i step              │
///                     │       │                                │
///                     │       │ jmp                            │
///                     └───────┤                                │
///                             │                                │
///                             │                                │
///                             └────────────────────────────────┘
fn thread_create_ir(
    mut current_loop: ObjPtr<LoopInfo>,
    prestart: ObjPtr<Inst>,
    step: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> (Option<LoopInfo>, ObjPtr<Inst>) {
    let mut preheader = current_loop.get_preheader();
    let mut header = current_loop.get_header();
    // 从上往下构造ir
    // thread_loop_head
    let mut thread_loop_head = pools
        .0
        .new_basic_block(format!("thread_loop_head_{}", header.get_name()));
    let const_0 = pools.1.make_int_const(0);
    let const_3 = pools.1.make_int_const(3);
    preheader.get_tail_inst().insert_before(const_0);
    preheader.get_tail_inst().insert_before(const_3);

    let mut phi_i = pools.1.make_int_phi();
    let mut phi_start = pools.1.make_int_phi();
    let lt = pools.1.make_lt(phi_i, const_3);

    thread_loop_head.push_back(phi_i);
    thread_loop_head.push_back(phi_start);
    thread_loop_head.push_back(lt);
    thread_loop_head.push_back(pools.1.make_br(lt));

    // thread_loop_call
    let mut thread_loop_call = pools
        .0
        .new_basic_block(format!("thread_loop_call_{}", header.get_name()));
    let call = pools
        .1
        .make_int_call("hitsz_thread_create".to_string(), Vec::new());
    let ne = pools.1.make_eq(call, const_0);

    thread_loop_call.push_back(call);
    thread_loop_call.push_back(ne);
    thread_loop_call.push_back(pools.1.make_br(ne));

    // thread_loop_update
    let mut thread_loop_update = pools
        .0
        .new_basic_block(format!("thread_loop_update_{}", header.get_name()));
    let start_add = pools.1.make_add(phi_start, step);
    let i_add = pools.1.make_add(phi_i, step);

    thread_loop_update.push_back(start_add);
    thread_loop_update.push_back(i_add);
    thread_loop_update.push_back(pools.1.make_jmp());

    // thread_loop_jmp
    let mut thread_loop_jmp = pools
        .0
        .new_basic_block(format!("thread_loop_jmp_{}", header.get_name()));
    thread_loop_jmp.push_back(pools.1.make_jmp());

    // 修改phi的参数
    phi_i.add_operand(const_0);
    phi_i.add_operand(i_add);

    phi_start.add_operand(prestart);
    phi_start.add_operand(start_add);

    // 修改cfg结构
    preheader.replace_next_bb(header, thread_loop_head);
    header.replace_up_bb(preheader, thread_loop_jmp);

    thread_loop_head.set_next_bb(vec![thread_loop_jmp, thread_loop_call]);
    thread_loop_head.set_up_bb(vec![preheader, thread_loop_update]);

    thread_loop_call.set_next_bb(vec![thread_loop_jmp, thread_loop_update]);
    thread_loop_call.set_up_bb(vec![thread_loop_head]);

    thread_loop_update.set_next_bb(vec![thread_loop_head]);
    thread_loop_update.set_up_bb(vec![thread_loop_call]);

    thread_loop_jmp.set_next_bb(vec![header]);
    thread_loop_jmp.set_up_bb(vec![thread_loop_head, thread_loop_call]);

    // 修改循环信息
    // 将当前循环的preheader设置为thread_loop_jmp
    current_loop.set_pre_header(thread_loop_jmp);
    // 将thread_loop_jmp加入父循环
    let mut thread_loop = None;
    if let Some(mut p_loop) = current_loop.get_parent_loop() {
        p_loop.add_bbs(vec![thread_loop_jmp]);
        thread_loop = Some(LoopInfo::new_loop(
            Some(p_loop),
            Some(preheader),
            thread_loop_head,
            Some(vec![thread_loop_update]),
            Some(vec![thread_loop_head, thread_loop_call]),
            vec![
                thread_loop_head,
                thread_loop_call,
                thread_loop_update,
                thread_loop_jmp,
            ],
            vec![],
        ));
    }

    (thread_loop, phi_start)
}

/// 线程退出的大致结构
///           ┌───────────────────────┐
///           │ exiting_block         │
///           │                       │
///           └─────────┬─────────────┘
///                     │
///           ┌─────────▼────────────┐
///           │ exit_block           │
///           │ thread_exit()        │
///           │                      │
///           └──────────────────────┘
fn thread_exit_ir(
    exiting_block: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let call = pools
        .1
        .make_void_call("hitsz_thread_join".to_string(), vec![]);
    let mut inst = exiting_block.get_head_inst();
    while let InstKind::Phi = inst.get_kind() {
        inst = inst.get_next();
    }
    inst.insert_before(call);
}
