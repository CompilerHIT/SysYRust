use super::*;
use crate::ir::{
    analysis::{
        dominator_tree::calculate_dominator,
        scev::{scevexp::SCEVExpKind, SCEVAnalyzer},
    },
    instruction::InstKind,
};

/// 尝试对循环进行展开
pub fn loop_unrolling(
    module: &mut Module,
    loop_map: &mut HashMap<String, LoopList>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    func_process(module, |name, _| loop {
        let mut flag = false;
        let loop_list = loop_map.get_mut(&name).unwrap();
        let mut analyzer = SCEVAnalyzer::new();
        analyzer.set_loop_list(loop_list.get_loop_list().clone());
        let mut remove_list = None;
        for loop_info in loop_list.get_loop_list().iter() {
            if loop_info.get_sub_loops().len() == 0 && loop_info.get_current_loop_bb().len() == 2 {
                flag = attempt_loop_unrolling(&mut analyzer, loop_info.clone(), pools);
            }

            if flag {
                remove_list = Some(loop_info.clone());
                break;
            }
        }

        if !flag {
            break;
        } else {
            loop_list.remove_loops(&vec![remove_list.unwrap()]);
        }
    });
}

enum IVC {
    // 递归表达式且只有两个操作数
    Introduction,
    // rhs是常数
    Const,
    // 不是递归表达式
    Nothing,
}

fn attempt_loop_unrolling(
    analyzer: &mut SCEVAnalyzer,
    mut loop_info: ObjPtr<LoopInfo>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> bool {
    if loop_info.get_sub_loops().len() != 0 || loop_info.get_current_loop_bb().len() != 2 {
        return false;
    }

    debug_assert_eq!(loop_info.get_exit_blocks().len(), 1);

    let end_cond = loop_info.get_exit_blocks()[0].get_tail_inst().get_br_cond();
    let lhs_scev = analyzer.analyze(&end_cond.get_lhs());
    let rhs_scev = analyzer.analyze(&end_cond.get_rhs());
    let start;
    let step;
    let end;
    let round;

    match (
        if lhs_scev.is_scev_constant() {
            IVC::Const
        } else if lhs_scev.is_scev_rec()
            && lhs_scev.get_operands().len() == 2
            && lhs_scev.get_operands().iter().all(|x| x.is_scev_constant())
        {
            IVC::Introduction
        } else {
            IVC::Nothing
        },
        end_cond.get_kind(),
        if rhs_scev.is_scev_constant() {
            IVC::Const
        } else if rhs_scev.is_scev_rec()
            && rhs_scev.get_operands().len() == 2
            && rhs_scev.get_operands().iter().all(|x| x.is_scev_constant())
        {
            IVC::Introduction
        } else {
            IVC::Nothing
        },
    ) {
        (IVC::Introduction, InstKind::Binary(crate::ir::instruction::BinOp::Le), IVC::Const) => {
            // i <= 常数
            start = lhs_scev.get_operands()[0].get_scev_const();
            step = lhs_scev.get_operands()[1].get_scev_const();
            end = rhs_scev.get_scev_const();
            debug_assert!(step > 0);
            round = (end - start) / step + 1;
        }
        (IVC::Introduction, InstKind::Binary(crate::ir::instruction::BinOp::Lt), IVC::Const) => {
            // i < 常数
            start = lhs_scev.get_operands()[0].get_scev_const();
            step = lhs_scev.get_operands()[1].get_scev_const();
            end = rhs_scev.get_scev_const();
            debug_assert!(step > 0);
            if (end - start) % step == 0 {
                round = (end - start) / step;
            } else {
                round = (end - start) / step + 1;
            }
        }
        (IVC::Const, InstKind::Binary(crate::ir::instruction::BinOp::Le), IVC::Introduction) => {
            // 常数 <= i
            start = rhs_scev.get_operands()[0].get_scev_const();
            step = rhs_scev.get_operands()[1].get_scev_const();
            end = lhs_scev.get_scev_const();
            debug_assert!(step < 0);
            round = (end - start) / step + 1;
        }
        (IVC::Const, InstKind::Binary(crate::ir::instruction::BinOp::Lt), IVC::Introduction) => {
            // 常数 < i
            start = rhs_scev.get_operands()[0].get_scev_const();
            step = rhs_scev.get_operands()[1].get_scev_const();
            end = lhs_scev.get_scev_const();
            debug_assert!(step < 0);
            if (end - start) % step == 0 {
                round = (end - start) / step;
            } else {
                round = (end - start) / step + 1;
            }
        }
        (IVC::Introduction, InstKind::Binary(crate::ir::instruction::BinOp::Gt), IVC::Const) => {
            // i > 常数
            start = lhs_scev.get_operands()[0].get_scev_const();
            step = lhs_scev.get_operands()[1].get_scev_const();
            end = rhs_scev.get_scev_const();
            debug_assert!(step < 0);
            if (end - start) % step == 0 {
                round = (end - start) / step;
            } else {
                round = (end - start) / step + 1;
            }
        }
        (IVC::Introduction, InstKind::Binary(crate::ir::instruction::BinOp::Ge), IVC::Const) => {
            // i >= 常数
            start = lhs_scev.get_operands()[0].get_scev_const();
            step = lhs_scev.get_operands()[1].get_scev_const();
            end = rhs_scev.get_scev_const();
            round = (end - start) / step + 1;
        }
        (IVC::Const, InstKind::Binary(crate::ir::instruction::BinOp::Gt), IVC::Introduction) => {
            // 常数 > i
            start = rhs_scev.get_operands()[0].get_scev_const();
            step = rhs_scev.get_operands()[1].get_scev_const();
            end = lhs_scev.get_scev_const();
            debug_assert!(step > 0);
            if (end - start) % step == 0 {
                round = (end - start) / step;
            } else {
                round = (end - start) / step + 1;
            }
        }
        (IVC::Const, InstKind::Binary(crate::ir::instruction::BinOp::Ge), IVC::Introduction) => {
            // 常数 >= i
            start = rhs_scev.get_operands()[0].get_scev_const();
            step = rhs_scev.get_operands()[1].get_scev_const();
            end = lhs_scev.get_scev_const();
            debug_assert!(step > 0);
            round = (end - start) / step + 1;
        }
        _ => return false,
    }

    if round > 1000 {
        return false;
    }

    one_block_loop_full_unrolling(loop_info, pools, round);
    analyzer.clear();
    true
}

/// 对循环体内只有一个基本块，且循环次数已知的循环进行完全展开
fn one_block_loop_full_unrolling(
    loop_info: ObjPtr<LoopInfo>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    round: i32,
) {
    let bodys = loop_info.get_current_loop_bb();

    debug_assert_eq!(bodys.len(), 2);
    let mut body = if bodys[0] == loop_info.get_header() {
        bodys[1]
    } else {
        bodys[0]
    };

    let mut insts = Vec::new();
    let mut phi = Vec::new();
    let mut map = HashMap::new();

    let mut inst = loop_info.get_header().get_head_inst();
    while let InstKind::Phi = inst.get_kind() {
        phi.push(inst);
        inst = inst.get_next();
    }

    // 初始化map
    for inst in phi.iter() {
        let value = if loop_info.is_in_current_loop(&inst.get_operands()[0].get_parent_bb()) {
            inst.get_operands()[0]
        } else {
            inst.get_operands()[1]
        };
        map.insert(inst.clone(), value);
    }

    // 初始化last_insts
    let mut last_insts = Vec::new();
    let mut inst = body.get_head_inst();
    while !inst.is_br() {
        last_insts.push(inst);
        inst = inst.get_next();
    }

    // 将循环体内的指令复制round次
    for _i in 1..round {
        let mut current_insts = Vec::new();
        for &inst in last_insts.iter() {
            let mut new_inst = pools.1.put(inst.as_ref().clone());
            debug_assert_ne!(inst, new_inst);
            current_insts.push(new_inst);

            // 更新map
            map.insert(inst, new_inst);

            // 设置new_inst的操作数
            new_inst.as_mut().set_operands(
                new_inst
                    .get_operands()
                    .clone()
                    .iter()
                    .map(|x| {
                        if let Some(y) = map.get(&x) {
                            y.clone()
                        } else {
                            x.clone()
                        }
                    })
                    .collect(),
            );
            // 增加use关系
            new_inst.set_users(vec![]);
            new_inst
                .get_operands()
                .iter()
                .for_each(|x| x.as_mut().add_user(new_inst.as_ref()));
        }

        // 更新map中phi的映射
        phi.iter().for_each(|x| {
            map.insert(*x, map.get(map.get(x).unwrap()).unwrap().clone());
        });

        last_insts = current_insts.clone();
        insts.extend(current_insts.clone());
    }
    // 将对于原始循环内的指令的使用替换为对最后一次循环中的指令的使用
    phi.iter().for_each(|x| {
        x.get_use_list().clone().iter_mut().for_each(|user| {
            if !loop_info.is_in_current_loop(&user.get_parent_bb()) {
                let operand_index = user.get_operand_index(*x);
                user.set_operand(map.get(x).unwrap().clone(), operand_index);
            }
        });
    });
    let mut index = 0;
    inst_process_in_bb(body.get_head_inst(), |x| {
        x.get_use_list().clone().iter_mut().for_each(|user| {
            if !loop_info.is_in_current_loop(&user.get_parent_bb()) {
                let operand_index = user.get_operand_index(x);
                user.set_operand(last_insts[index].clone(), operand_index);
            }
        });
        index = index + 1;
    });

    // 将生成的指令插入循环体
    let mut tail = body.get_tail_inst();
    insts.iter_mut().for_each(|x| tail.insert_before(*x));

    // 修改cfg
    let mut header = loop_info.get_header();
    let mut exit;
    let header_next = header.get_next_bb().clone();
    if loop_info.is_in_current_loop(&header_next[0]) {
        exit = header.get_next_bb()[1];
        header.set_next_bb(vec![header_next[0]]);
    } else {
        exit = header.get_next_bb()[0];
        header.set_next_bb(vec![header_next[1]]);
    };
    body.remove_next_bb(header);
    body.set_next_bb(vec![exit]);
    let upbb = exit.get_up_bb().clone();
    exit.set_up_bb(
        upbb.iter()
            .map(|x| if *x == header { body } else { *x })
            .collect(),
    );
    header.get_tail_inst().remove_self();
    header.push_back(pools.1.make_jmp());

    // 修改循环信息
    if let Some(mut parent) = loop_info.get_parent_loop() {
        parent.remove_sub_loop(loop_info);
        parent.add_bbs(loop_info.get_current_loop_bb().clone());
    }
}
