//! 死代码删除
//! 基本块内的死代码删除
//! 根据inst的use_list的长度来判断
//! 长度为0但不是死代码的有：
//! 1. call指令
//! 2. store指令
//! 3. ret指令
//! 4. br指令
//! 5. Head指令

use std::collections::HashSet;

use crate::{
    ir::{basicblock::BasicBlock, instruction::InstKind},
    utility::ObjPtr,
};

pub fn dead_code_eliminate(end_bb: ObjPtr<BasicBlock>) {
    let mut changed = true;
    while changed {
        changed = false;
        let mut visited = HashSet::new();
        let mut queue = Vec::new();
        queue.insert(0, end_bb);
        while let Some(bb) = queue.pop() {
            if visited.contains(&bb) {
                continue;
            }
            visited.insert(bb);
            changed |= eliminate_bb_inst(bb);
            for pred in bb.get_up_bb() {
                queue.insert(0, pred.clone());
            }
        }
    }
}

fn eliminate_bb_inst(bb: ObjPtr<BasicBlock>) -> bool {
    if bb.is_empty() {
        return false;
    }

    let mut changed = false;
    let mut inst = bb.get_head_inst();
    loop {
        if let InstKind::Head(_) = inst.get_kind() {
            break;
        }

        if inst.get_use_list().len() == 0 {
            match inst.get_kind() {
                InstKind::Call(_) | InstKind::Store | InstKind::Return | InstKind::Branch => {}
                _ => {
                    changed = true;
                    let next_inst = inst.get_next();
                    inst.remove_self();
                    inst = next_inst;
                    continue;
                }
            }
        }
        inst = inst.get_next();
    }

    changed
}
