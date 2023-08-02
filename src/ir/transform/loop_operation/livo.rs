use std::collections::HashSet;

use crate::ir::analysis::{
    dominator_tree::DominatorTree,
    scev::{
        scevexp::{SCEVExp, SCEVExpKind},
        SCEVAnalyzer,
    },
};

use super::*;

pub fn livo_run(
    dominator_tree: DominatorTree,
    loop_list: &mut LoopList,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    for loop_info in loop_list.get_loop_list().iter() {
        if !is_break_continue_exist(*loop_info) {
            let mut scev_analyzer = SCEVAnalyzer::new();
            scev_analyzer.set_loop_list(loop_list.get_loop_list().clone());
            livo_in_loop(&dominator_tree, *loop_info, &mut scev_analyzer, pools);
        }
    }
}

fn livo_in_loop(
    dominator_tree: &DominatorTree,
    loop_info: ObjPtr<LoopInfo>,
    scev_analyzer: &mut SCEVAnalyzer,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    // 获得当前循环的归纳变量
    let mut rec_set = HashSet::new();
    loop_info.get_current_loop_bb().iter().for_each(|bb| {
        inst_process_in_bb(bb.get_head_inst(), |inst| {
            if let SCEVExpKind::SCEVMulRecExpr = scev_analyzer.analyze(&inst).get_kind() {
                rec_set.insert(inst);
            }
        })
    });

    let latchs = loop_info.get_latchs();
    debug_assert_eq!(latchs.len(), 1);
    let latch = latchs[0];

    for inst in rec_set.iter() {
        if dominator_tree.is_dominate(&inst.get_parent_bb(), &latch) {
            mul_livo(*inst, loop_info, pools, scev_analyzer);
        }
    }
}

/// 进行乘法指令的归纳变量强度削减
fn mul_livo(
    mut inst: ObjPtr<Inst>,
    loop_info: ObjPtr<LoopInfo>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    scev_analyzer: &mut SCEVAnalyzer,
) {
    let scev_exp = scev_analyzer.analyze(&inst);
    debug_assert!(scev_exp.get_operands().len() >= 2);

    let mut tail_inst = loop_info.get_preheader().get_tail_inst();
    let op_vec = scev_exp
        .get_operands()
        .iter()
        .map(|x| {
            let cur_op_vec = parse_scev_exp(*x, pools);
            cur_op_vec.iter().for_each(|op| {
                tail_inst.insert_before(*op);
            });
            cur_op_vec.last().unwrap().clone()
        })
        .collect::<Vec<_>>();

    let new_inst = parse_step(inst, pools, loop_info, &op_vec);

    inst.get_use_list().clone().iter_mut().for_each(|user| {
        let index = user.get_operand_index(inst);
        user.set_operand(new_inst, index);
    });

    inst.remove_self();
}

fn parse_step(
    mut inst: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    loop_info: ObjPtr<LoopInfo>,
    op_slice: &[ObjPtr<Inst>],
) -> ObjPtr<Inst> {
    let start = op_slice[0];
    let step;

    if op_slice.len() == 2 {
        step = op_slice[1];
    } else {
        step = parse_step(inst, pools, loop_info, &op_slice[1..])
    }

    let mut header = loop_info.get_header();
    let preheader = loop_info.get_preheader();
    let mut phi = pools.1.make_phi(start.get_ir_type());
    header.push_front(phi);

    let adder = pools.1.make_add(phi, step);
    inst.insert_after(adder);

    if header.get_up_bb()[0] == preheader {
        phi.add_operand(start);
        phi.add_operand(adder);
    } else {
        phi.add_operand(adder);
        phi.add_operand(start);
    }

    phi
}

fn parse_scev_exp(
    exp: ObjPtr<SCEVExp>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> Vec<ObjPtr<Inst>> {
    let mut l_d = false;

    if exp.is_scev_constant() {
        return vec![pools.1.make_int_const(exp.get_scev_const())];
    }

    let mut lhs = if exp.get_operands()[0].is_scev_rec() || exp.get_operands()[0].is_scev_unknown()
    {
        l_d = true;
        vec![exp.get_operands()[0].get_bond_inst()]
    } else {
        parse_scev_exp(exp.get_operands()[0], pools)
    };

    let mut r_d = false;
    let mut rhs = if exp.get_operands()[1].is_scev_rec() || exp.get_operands()[1].is_scev_unknown()
    {
        r_d = true;
        vec![exp.get_operands()[1].get_bond_inst()]
    } else {
        parse_scev_exp(exp.get_operands()[1], pools)
    };

    let l_i = lhs.len() - 1;
    let r_i = rhs.len() - 1;
    let result;

    match exp.get_kind() {
        SCEVExpKind::SCEVAddExpr => {
            debug_assert_eq!(exp.get_operands().len(), 2);
            result = pools.1.make_add(lhs[l_i], rhs[r_i]);
        }
        SCEVExpKind::SCEVSubExpr => {
            debug_assert_eq!(exp.get_operands().len(), 2);
            result = pools.1.make_sub(lhs[l_i], rhs[r_i]);
        }
        SCEVExpKind::SCEVMulExpr => {
            debug_assert_eq!(exp.get_operands().len(), 2);
            result = pools.1.make_mul(lhs[l_i], rhs[r_i]);
        }
        _ => {
            unreachable!()
        }
    }

    if l_d {
        lhs.pop();
    }

    if r_d {
        rhs.pop();
    }

    lhs.extend(rhs);
    lhs.push(result);
    lhs
}
