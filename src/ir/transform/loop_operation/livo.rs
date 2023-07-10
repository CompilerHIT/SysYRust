use std::collections::HashSet;

use crate::ir::analysis::{
    dominator_tree::{self, calculate_dominator, DominatorTree},
    scev::{scevexp::SCEVExpKind, SCEVAnalyzer},
};

use super::*;

pub fn livo_run(
    dominator_tree: DominatorTree,
    loop_list: &mut LoopList,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut scev_analyzer = SCEVAnalyzer::new();
    scev_analyzer.set_loops(loop_list);
    for loop_info in loop_list.get_loop_list().iter() {
        if !is_break_continue_exist(*loop_info) {
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
    let mut inst = loop_info.get_header().get_head_inst();
    while SCEVExpKind::SCEVAddRecExpr != scev_analyzer.analyze(inst).get_kind() && !inst.is_tail() {
        inst = inst.get_next();
    }

    if inst.is_tail() {
        return;
    }

    let latchs = loop_info.get_latchs();

    loop {
        let mut flag = false;
        for user in inst.get_use_list().iter() {
            if !loop_info.is_in_loop(&user.get_parent_bb()) {
                continue;
            }
            if scev_analyzer.analyze(*user).is_scev_mul_expr()
                && latchs
                    .iter()
                    .all(|bb| dominator_tree.is_dominate(&user.get_parent_bb(), bb))
            {
                flag = mul_livo(*user, loop_info, pools, scev_analyzer);
                break;
            }
        }

        if !flag {
            break;
        }
    }
}

/// 进行乘法指令的归纳变量强度削减
fn mul_livo(
    mut inst: ObjPtr<Inst>,
    loop_info: ObjPtr<LoopInfo>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    scev_analyzer: &mut SCEVAnalyzer,
) -> bool {
    let (rec_phi, other) = if let SCEVExpKind::SCEVAddRecExpr =
        scev_analyzer.analyze(inst.get_operands()[0]).get_kind()
    {
        (inst.get_operands()[0], inst.get_operands()[1])
    } else {
        (inst.get_operands()[1], inst.get_operands()[0])
    };

    if loop_info.is_in_loop(&other.get_parent_bb()) {
        return false;
    }

    let rec_exp = scev_analyzer.analyze(rec_phi);
    let start = rec_exp.get_add_rec_start();
    let step = rec_exp.get_add_rec_step();

    // 在preheader中插入start-step乘other的指令
    let sub = pools.1.make_sub(start, step);
    let mut init = pools.1.make_mul(sub, other);
    loop_info
        .get_preheader()
        .get_tail_inst()
        .insert_before(init);
    init.insert_before(sub);

    // 在inst前插入step乘other指令
    let temp = pools.1.make_mul(step, other);
    inst.insert_before(temp);

    // 在inst前插入一条phi加temp的指令
    let phi = find_point_inst(init, inst.get_parent_bb(), pools);
    let phi_add = pools.1.make_add(phi, temp);
    inst.insert_before(phi_add);
    // 把使用phi的地方都替换成phi_add
    phi.get_use_list().clone().iter_mut().for_each(|user| {
        if user.is_phi() {
            let index = user.get_operand_index(phi);
            user.set_operand(phi_add, index);
        }
    });

    // 把所有使用inst的地方都替换成phi_add
    inst.get_use_list().clone().iter_mut().for_each(|user| {
        let index = user.get_operand_index(inst);
        user.set_operand(phi_add, index);
    });

    inst.remove_self();

    scev_analyzer.clear();

    true
}

/// 递归向上找到指定的inst，在寻找的过程中会插phi指令
fn find_point_inst(
    inst: ObjPtr<Inst>,
    current_bb: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<Inst> {
    let mut set = HashSet::new();
    set.insert(inst);
    find_inst_in_set(&mut set, current_bb, pools)
}

/// 递归向上寻找指定的inst集合，在寻找的过程中会插phi指令
fn find_inst_in_set(
    map: &mut HashSet<ObjPtr<Inst>>,
    current_bb: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<Inst> {
    let mut target = None;
    inst_process_in_bb(current_bb.get_head_inst(), |inst| {
        if map.contains(&inst) {
            target = Some(inst);
        }
    });

    if let Some(inst) = target {
        inst
    } else {
        let mut phi = pools.1.make_int_phi();
        current_bb.as_mut().push_front(phi);
        map.insert(phi);
        for bb in current_bb.get_up_bb().iter() {
            phi.add_operand(find_inst_in_set(map, *bb, pools));
        }
        phi
    }
}
