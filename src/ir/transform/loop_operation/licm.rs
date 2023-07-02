use crate::{
    ir::{
        analysis::loop_tree::{LoopInfo, LoopList},
        instruction::InstKind,
        tools::inst_process_in_bb,
    },
    utility::ObjPtr,
};

use super::*;

pub fn licm_run(
    loop_list: &mut LoopList,
    _pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    loop {
        let mut flag = false;
        // 先把当前循环的循环不变量放到循环的preheader中
        loop {
            let mut changed = false;
            for loop_info in loop_list.get_loop_list() {
                changed |= licm_one_loop(loop_info.clone());
            }

            if !changed {
                break;
            }
            flag |= changed;
        }

        // 把子循环的循环不变量放到父循环的preheader中
        loop {
            let mut changed = false;
            for loop_info in loop_list.get_loop_list() {
                changed |= licm_sub_loops(loop_info.clone());
            }
            if !changed {
                break;
            }
            flag |= changed;
        }

        if !flag {
            break;
        }
    }
}

fn licm_one_loop(loop_info: ObjPtr<LoopInfo>) -> bool {
    let preheader = loop_info.get_preheader();
    let mut changed = false;
    let mut tail_inst = preheader.get_tail_inst();
    for bb in loop_info.get_current_loop_bb().clone() {
        if bb == preheader {
            continue;
        }

        inst_process_in_bb(bb.get_head_inst(), |mut inst| {
            if is_loop_invariant(inst, loop_info) {
                match inst.get_kind() {
                    InstKind::Alloca(_) | InstKind::Load | InstKind::Return | InstKind::Branch => {}
                    _ => {
                        changed = true;
                        inst.move_self();
                        tail_inst.insert_before(inst);
                    }
                }
            }
        });
    }

    changed
}

fn licm_sub_loops(loop_info: ObjPtr<LoopInfo>) -> bool {
    let mut changed = false;
    for sub_loop in loop_info.get_sub_loops().clone() {
        let sub_preheader = sub_loop.get_preheader();
        let mut tail_inst = loop_info.get_preheader().get_tail_inst();
        inst_process_in_bb(sub_preheader.get_head_inst(), |mut inst| {
            if is_loop_invariant(inst, loop_info) {
                inst.move_self();
                tail_inst.insert_before(inst);
                changed = true;
            }
        });
    }
    changed
}

fn is_loop_invariant(inst: ObjPtr<Inst>, loop_info: ObjPtr<LoopInfo>) -> bool {
    inst.get_operands().iter().all(|op| {
        op.is_global_var()
            || !(loop_info.is_in_current_loop(&op.get_parent_bb())
                && op.get_parent_bb() != loop_info.get_preheader())
            || match op.get_kind() {
                InstKind::ConstInt(_) | InstKind::ConstFloat(_) => true,
                _ => false,
            }
    })
}
