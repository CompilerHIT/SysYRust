use std::{collections::HashSet, hash::Hash};

use crate::ir::{
    analysis::{
        call_optimize::call_optimize,
        scev::{scevexp::SCEVExp, SCEVAnalyzer},
    },
    instruction::InstKind,
};

use super::{livo::parse_scev_exp, *};
pub fn loop_elimination(
    module: &mut Module,
    loop_map: &mut HashMap<String, LoopList>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let call_op_set = call_optimize(module);
    func_process(module, |name, _| {
        let mut delete_list = vec![];
        let looplist = loop_map.get_mut(&name).unwrap();
        let mut analyzer = SCEVAnalyzer::new();
        analyzer.set_loop_list(looplist.get_loop_list().clone());
        looplist.get_loop_list().iter().for_each(|loop_info| {
            if let Some(exit) = check_one_exiting(*loop_info) {
                loop_induct(*loop_info, &mut analyzer, exit, pools);
                loop_store_eliminate(*loop_info, &mut analyzer, exit, pools);
                loop_dead_code_eliminate(*loop_info, &call_op_set);
                if loop_eliminate(*loop_info, &call_op_set, exit) {
                    delete_list.push(*loop_info);
                    analyzer.clear();
                }
            }
        });
        looplist.remove_loops(&delete_list);
    })
}

fn check_one_exiting(loop_info: ObjPtr<LoopInfo>) -> Option<[ObjPtr<BasicBlock>; 2]> {
    if loop_info.get_sub_loops().len() != 0 {
        None
    } else {
        let mut exit = loop_info
            .get_current_loop_bb()
            .iter()
            .filter_map(|bb| {
                if let Some(next_bb) = bb
                    .get_next_bb()
                    .iter()
                    .find(|next_bb| !loop_info.is_in_current_loop(next_bb))
                {
                    Some([bb.clone(), next_bb.clone()])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if exit.len() == 1 {
            exit.pop()
        } else {
            None
        }
    }
}

fn loop_induct(
    loop_info: ObjPtr<LoopInfo>,
    analyzer: &mut SCEVAnalyzer,
    exiting_exit: [ObjPtr<BasicBlock>; 2],
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut inst_list = vec![];
    let [exiting, _exit] = exiting_exit;

    // 找到被循环外使用的变量
    loop_info.get_current_loop_bb().iter().for_each(|bb| {
        inst_process_in_bb(bb.get_head_inst(), |inst| {
            let mut users = vec![];
            inst.get_use_list().iter().for_each(|user| {
                if !loop_info.is_in_current_loop(&user.get_parent_bb()) {
                    users.push(*user)
                }
            });
            inst_list.push((inst, users));
        })
    });

    if inst_list.len() == 0 {
        return;
    }

    let round = parse_round(
        analyzer,
        loop_info,
        exiting.get_tail_inst().get_br_cond(),
        pools,
    );
    if round.is_none() {
        return;
    }
    let round = round.unwrap();
    // 循环变量归纳
    for (inst, mut users) in inst_list {
        let inst_scev = analyzer.analyze(&inst);
        if users.len() > 0 && inst_scev.is_scev_rec() {
            let new_inst = parse_inst(
                loop_info,
                loop_info.get_preheader().get_tail_inst(),
                round,
                &inst_scev.get_operands(),
                pools,
            );
            users.iter_mut().for_each(|user| {
                let index = user.get_operand_index(inst);
                user.set_operand(new_inst, index);
            })
        }
    }
}

fn loop_store_eliminate(
    loop_info: ObjPtr<LoopInfo>,
    analyzer: &mut SCEVAnalyzer,
    exit: [ObjPtr<BasicBlock>; 2],
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut array_store: HashMap<ObjPtr<Inst>, ObjPtr<Inst>> = HashMap::new();
    let mut insts: Vec<ObjPtr<Inst>> = vec![];
    let bbs = loop_info.get_current_loop_bb();
    for bb in bbs {
        inst_process_in_bb(bb.get_head_inst(), |inst| {
            if inst.is_global_array_load() {
                let dest = inst.get_dest();
                if let Some(store) = array_store.get(&dest) {
                    if let Some(index) = insts.iter().position(|x| x == store) {
                        insts.remove(index);
                    }
                } else {
                    array_store.insert(dest, inst);
                }
            } else if inst.is_array_store() {
                let dest = inst.get_dest();
                if let Some(store) = array_store.get(&dest) {
                    if let Some(index) = insts.iter().position(|x| x == store) {
                        insts.remove(index);
                    }
                } else {
                    array_store.insert(dest, inst);
                    insts.push(inst)
                }
            }
        })
    }

    let mut tail = loop_info.get_preheader().get_tail_inst();
    let round = parse_round(
        analyzer,
        loop_info,
        exit[0].get_tail_inst().get_br_cond(),
        pools,
    );

    let check_value = |value: ObjPtr<Inst>| -> bool {
        !value.is_global_var()
            && (value.is_param() || !loop_info.is_in_current_loop(&value.get_parent_bb()))
    };

    insts.iter_mut().for_each(|store| {
        let gep = store.get_dest();
        let value = store.get_value();
        if loop_info.is_in_current_loop(&gep.get_parent_bb()) && check_value(value) {
            if let Some(round) = round {
                let array = if gep.get_gep_ptr().is_array() || gep.get_gep_ptr().is_param() {
                    gep.get_gep_ptr()
                } else {
                    gep.get_gep_ptr().get_ptr()
                };
                let offset = gep.get_gep_offset();
                let iv = analyzer.analyze(&offset);
                if iv.is_scev_rec_expr()
                    && iv.get_operands()[1].is_scev_constant()
                    && iv.get_operands()[1].get_scev_const() == 1
                {
                    let start = if iv.get_operands()[0].is_scev_constant() {
                        let constx = pools
                            .1
                            .make_int_const(iv.get_operands()[0].get_scev_const());
                        tail.insert_before(constx);
                        constx
                    } else {
                        iv.get_operands()[0].get_bond_inst()
                    };
                    let new_gep_start = pools.1.make_gep(array, start);
                    let const_4 = pools.1.make_int_const(4);
                    let round_4 = pools.1.make_mul(round, const_4);
                    let memset = pools.1.make_void_call(
                        "hitsz_memset".to_string(),
                        vec![new_gep_start, value, round_4],
                    );
                    tail.insert_before(new_gep_start);
                    tail.insert_before(const_4);
                    tail.insert_before(round_4);
                    tail.insert_before(memset);
                    store.remove_self();
                }
            }
        } else if check_value(value) {
            store.move_self();
            tail.insert_before(*store);
        }
    })
}

fn loop_eliminate(
    loop_info: ObjPtr<LoopInfo>,
    call_op_set: &HashSet<String>,
    exit: [ObjPtr<BasicBlock>; 2],
) -> bool {
    let mut insts = vec![];
    let bbs = loop_info.get_current_loop_bb();
    for bb in bbs {
        inst_process_in_bb(bb.get_head_inst(), |inst| {
            insts.push(inst);
        })
    }

    let check_inst_essential = |inst: &ObjPtr<Inst>| -> bool {
        inst.is_br()
            || !inst.is_store()
                && if let InstKind::Call(callee) = inst.get_kind() {
                    call_op_set.contains(&callee)
                } else {
                    true
                }
                && inst
                    .get_use_list()
                    .iter()
                    .all(|user| loop_info.is_in_current_loop(&user.get_parent_bb()))
    };

    if insts.iter().all(|x| check_inst_essential(x)) {
        insts.iter_mut().for_each(|x| x.remove_self());
        let mut preheader = loop_info.get_preheader();
        let [exiting, mut exit] = exit;
        preheader.set_next_bb(vec![exit]);
        exit.replace_up_bb(exiting, preheader);
        true
    } else {
        false
    }
}

fn loop_dead_code_eliminate(loop_info: ObjPtr<LoopInfo>, call_op_set: &HashSet<String>) {
    let mut visited: HashSet<&ObjPtr<Inst>> = HashSet::new();

    let mut insts = vec![];
    let bbs = loop_info.get_current_loop_bb();
    for bb in bbs {
        inst_process_in_bb(bb.get_head_inst(), |inst| {
            insts.push(inst);
        })
    }

    let check_call_op = |inst: ObjPtr<Inst>| -> bool {
        if let InstKind::Call(callee) = inst.get_kind() {
            call_op_set.contains(&callee)
        } else {
            true
        }
    };

    let mut delete_list: HashSet<ObjPtr<Inst>> = HashSet::new();
    insts.iter().for_each(|inst| {
        if !visited.contains(&inst) {
            let mut current = HashSet::new();
            let mut queue = vec![inst];
            let mut flag = true;
            while let Some(current_inst) = queue.pop() {
                current.insert(current_inst);
                if !current_inst.is_store()
                    && !current_inst.is_br()
                    && check_call_op(*current_inst)
                    && current_inst.get_use_list().iter().all(|user| {
                        (user.is_global_var_or_param()
                            || loop_info.is_in_current_loop(&user.get_parent_bb()))
                            && !user.is_br()
                            && !user.is_store()
                            && check_call_op(*user)
                    })
                {
                    queue.extend(
                        current_inst
                            .get_use_list()
                            .iter()
                            .filter(|user| !current.contains(user)),
                    );
                } else {
                    flag = false;
                    break;
                }
            }
            if flag {
                delete_list.extend(current.clone());
            }
            visited.extend(current);
        }
    });
    delete_list.iter().for_each(|x| x.as_mut().remove_self());
}

fn parse_round(
    analyzer: &mut SCEVAnalyzer,
    loop_info: ObjPtr<LoopInfo>,
    cond: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> Option<ObjPtr<Inst>> {
    if !cond.is_cond() {
        return None;
    }
    let lhs = analyzer.analyze(&cond.get_lhs());
    let rhs = analyzer.analyze(&cond.get_rhs());

    let check_current_iv = |op: &ObjPtr<SCEVExp>| -> bool {
        op.is_scev_rec_expr() && loop_info == op.get_in_loop().unwrap()
    };

    let parse_answer = |start: ObjPtr<SCEVExp>,
                        end: ObjPtr<SCEVExp>,
                        can_be_equal: bool,
                        pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)|
     -> ObjPtr<Inst> {
        let mut tail = loop_info.get_preheader().get_tail_inst();
        let start = parse_one_inst(loop_info, start, tail, pools);
        let end = parse_one_inst(loop_info, end, tail, pools);

        let mut minus = pools.1.make_sub(end, start);
        tail.insert_before(minus);
        if can_be_equal {
            let const_1 = pools.1.make_int_const(1);
            minus = pools.1.make_add(minus, const_1);
            tail.insert_before(const_1);
            tail.insert_before(minus);
        }
        minus
    };

    let check_eq = |op: InstKind| match op {
        InstKind::Binary(crate::ir::instruction::BinOp::Le)
        | InstKind::Binary(crate::ir::instruction::BinOp::Ge)
        | InstKind::Binary(crate::ir::instruction::BinOp::Eq) => true,
        _ => false,
    };

    match (check_current_iv(&lhs), check_current_iv(&rhs)) {
        (true, false) => {
            if rhs.get_in_loop() == Some(loop_info) {
                None
            } else {
                let start = lhs.get_operands()[0];
                let step = lhs.get_operands()[1];
                if step.is_scev_constant() && step.get_scev_const() == 1 {
                    Some(parse_answer(start, rhs, check_eq(cond.get_kind()), pools))
                } else {
                    None
                }
            }
        }
        (false, true) => {
            if lhs.get_in_loop() == Some(loop_info) {
                None
            } else {
                let start = rhs.get_operands()[0];
                let step = rhs.get_operands()[1];
                if step.is_scev_constant() && step.get_scev_const() == 1 {
                    Some(parse_answer(start, lhs, check_eq(cond.get_kind()), pools))
                } else {
                    None
                }
            }
        }
        _ => None,
    }
}

fn parse_inst(
    loop_info: ObjPtr<LoopInfo>,
    mut tail: ObjPtr<Inst>,
    round: ObjPtr<Inst>,
    operands: &[ObjPtr<SCEVExp>],
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<Inst> {
    let start = parse_one_inst(loop_info, operands[0], tail, pools);
    let step = if operands.len() > 2 {
        parse_inst(loop_info, tail, round, &operands[1..], pools)
    } else {
        parse_one_inst(loop_info, operands[1], tail, pools)
    };
    let mul = pools.1.make_mul(step, round);
    let add = pools.1.make_add(start, mul);
    tail.insert_before(mul);
    tail.insert_before(add);
    add
}

fn parse_one_inst(
    loop_info: ObjPtr<LoopInfo>,
    op: ObjPtr<SCEVExp>,
    mut tail: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<Inst> {
    if !op.is_scev_constant() && (op.is_scev_unknown() || op.get_in_loop() != Some(loop_info)) {
        op.get_bond_inst()
    } else {
        let vec = parse_scev_exp(op, pools);
        vec.iter().for_each(|x| tail.insert_before(*x));
        vec.last().unwrap().clone()
    }
}
