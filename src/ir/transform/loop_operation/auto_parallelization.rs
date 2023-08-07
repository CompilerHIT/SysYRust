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
        let mut analyzer = SCEVAnalyzer::new();
        analyzer.set_loop_list(loop_list.get_loop_list().clone());

        for loop_info in loop_list.get_loop_list().iter() {
            if !op.contains(loop_info) && !unop.contains(loop_info) {
                continue;
            }

            // 从当前循环向外层遍历，找到最外层未检测过的循环
            let mut current_loop = *loop_info;
            while let Some(x) = current_loop.get_parent_loop() {
                if !op.contains(&current_loop) && !unop.contains(&current_loop) {
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
                parallelize(current_loop, &mut op, &mut unop, pools);
            }
        }
    });
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
        inst_process_in_bb(bb.get_head_inst(), |inst| match inst.get_kind() {
            InstKind::Load => {
                if inst.get_ptr().get_kind() == InstKind::Gep {
                    let gep = inst.get_ptr();
                    let array = get_gep_ptr(gep);
                    debug_assert_eq!(array.get_kind(), InstKind::Alloca(0));
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
                debug_assert_eq!(array.get_kind(), InstKind::Alloca(0));
                if let Some(x) = write_map.get_mut(&array) {
                    x.insert(gep);
                } else {
                    write_map.insert(array, HashSet::new());
                }
            }
            InstKind::Call(callee) => {
                if call_op.contains(&callee) {
                    flag = false;
                    return;
                }
            }
            _ => {}
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
    if iv_set.len() > 1 {
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

fn parallelize(
    current_loop: ObjPtr<LoopInfo>,
    op: &mut HashSet<ObjPtr<LoopInfo>>,
    unop: &mut HashSet<ObjPtr<LoopInfo>>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    println!("parallelize loop: {:?}", current_loop);
}
