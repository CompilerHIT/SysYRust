use std::collections::HashSet;

use crate::{
    ir::{
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
        module::Module,
    },
    utility::{ObjPool, ObjPtr},
};

use super::{func_process, inst_process_in_bb, phi_optimizer};

///! 对于block的优化
///! 1. 删除无法到达的block：除头block外没有前继的就是无法到达的
///! 2. 合并只有一个后继和这个后继只有一个前继的block
///! 3. 删除无法到达的分支

pub fn simplify_cfg_run(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    func_process(module, |_, func| {
        remove_unreachable_bb(func.get_head(), pools);
    });

    phi_optimizer::phi_run(module);

    func_process(module, |_, func| {
        merge_one_line_bb(func.get_head());
    });
}

fn merge_one_line_bb(head: ObjPtr<BasicBlock>) {
    let mut bb_list = get_bb_list(head);

    loop {
        let mut changed = false;
        let mut delete_bb = None;
        for bb in bb_list.iter() {
            // 不考虑尾
            if bb.is_exit() {
                continue;
            }

            if bb_has_jump(bb.clone()) && bb.get_next_bb()[0].get_up_bb().len() == 1 {
                changed = true;
                delete_bb = Some(bb.get_next_bb()[0].clone());
                merge_bb(bb.clone());
                break;
            }
        }

        // 删除已经合并的block
        if let Some(bb) = delete_bb {
            let index = bb_list.iter().position(|x| x.clone() == bb).unwrap();
            bb_list.remove(index);
        }

        if !changed {
            break;
        }
    }
}

pub fn merge_bb(mut bb: ObjPtr<BasicBlock>) {
    let next_bb = bb.get_next_bb()[0].clone();
    if next_bb.is_exit() {
        bb.clear_next_bb();
    } else {
        let bb_next_0 = next_bb.get_next_bb()[0].clone();
        bb.replace_next_bb(next_bb, bb_next_0);
        bb_next_0.as_mut().replace_up_bb(next_bb, bb);

        if !bb_has_jump(next_bb) {
            let bb_next_1 = next_bb.get_next_bb()[1].clone();
            bb.add_next_bb(next_bb);
            bb.replace_next_bb(next_bb, bb_next_1);
            bb_next_1.as_mut().replace_up_bb(next_bb, bb);
        }
        next_bb.as_mut().clear_next_bb();
        next_bb.as_mut().clear_up_bb();
    }

    // 在进行phi优化后，单前继的块不会有phi指令
    if let InstKind::Phi = next_bb.get_head_inst().get_kind() {
        unreachable!("只有单前继的块不会有phi, bb: {}", next_bb.get_name());
    }

    // 移动剩下的指令
    bb.get_tail_inst().remove_self();
    inst_process_in_bb(next_bb.get_head_inst(), |inst| {
        inst.as_mut().move_self();
        bb.push_back(inst);
    });
}

fn remove_unreachable_bb(
    head: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut deleted = HashSet::new();

    let bb_list = get_bb_list(head);

    loop {
        let mut changed = false;
        for bb in bb_list.iter() {
            // 不考虑头和尾
            if bb.is_entry() || bb.is_exit() {
                continue;
            }

            // 如果没有前继或者前继都在deleted集里，那么当前bb是无法到达的
            if (bb.get_up_bb().is_empty() || bb.get_up_bb().iter().all(|bb| deleted.contains(bb)))
                && !deleted.contains(bb)
            {
                deleted.insert(bb);
                changed = true;
            }

            // jump指令不检查
            if bb_has_jump(bb.clone()) {
                continue;
            }

            // 检查是否分支不可达，并删除掉不可达的路径
            changed |= check_bb(bb.clone(), pools);
        }

        if !changed {
            break;
        }
    }

    // 将从头部不可达的bb也加入到deleted集里
    let bb_list_now = get_bb_list(head);
    for bb in bb_list.iter() {
        if !bb_list_now.contains(bb) {
            deleted.insert(bb);
        }
    }

    // 删除掉这些不可达的bb
    for &bb in deleted.iter() {
        let should_be_deleted: Vec<ObjPtr<BasicBlock>> = bb
            .get_next_bb()
            .iter()
            .filter(|x| !deleted.contains(x))
            .cloned()
            .collect();

        for next_bb in should_be_deleted.iter() {
            bb.as_mut().remove_next_bb(next_bb.clone());
        }

        remove_bb_self(bb.clone());
    }
}

fn remove_bb_self(bb: ObjPtr<BasicBlock>) {
    if bb.is_empty() {
        return;
    }
    let inst = bb.get_head_inst();
    inst_process_in_bb(inst, |x| {
        x.as_mut().remove_self();
    });
}

/// 检查分支是否无法到达
/// 如果无法到达，那么删除到达这个分支的路径
fn check_bb(
    bb: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> bool {
    let mut changed = false;
    let cond = bb.get_tail_inst().get_br_cond();

    match cond.get_kind() {
        InstKind::ConstInt(value) => {
            if value == 0 {
                let next_bb = bb.get_next_bb()[1].clone();
                bb.as_mut().remove_next_bb(next_bb);
            } else {
                let next_bb = bb.get_next_bb()[0].clone();
                bb.as_mut().remove_next_bb(next_bb);
            }
            bb.get_tail_inst().remove_self();
            bb.as_mut().push_back(pools.1.make_jmp());
            changed = true;
        }
        InstKind::ConstFloat(value) => {
            if value == 0.0 {
                bb.as_mut().remove_next_bb(bb.get_next_bb()[1].clone());
            } else {
                bb.as_mut().remove_next_bb(bb.get_next_bb()[0].clone());
            }
            bb.get_tail_inst().remove_self();
            bb.as_mut().push_back(pools.1.make_jmp());
            changed = true;
        }

        _ => {}
    }

    changed
}

fn get_bb_list(head: ObjPtr<BasicBlock>) -> Vec<ObjPtr<BasicBlock>> {
    let mut queue = Vec::new();
    let mut visited = HashSet::new();
    queue.insert(0, head);
    while let Some(bb) = queue.pop() {
        if !visited.contains(&bb) {
            visited.insert(bb.clone());
            queue.extend(bb.get_next_bb().iter().cloned());
        }
    }
    visited.iter().cloned().collect::<Vec<_>>()
}

fn bb_has_jump(bb: ObjPtr<BasicBlock>) -> bool {
    bb.get_tail_inst().is_br_jmp()
}
