use std::collections::HashSet;

use crate::utility::ObjPtr;

use self::phi_optimizer::phi_run;

use super::{basicblock::BasicBlock, module::Module};

mod phi_optimizer;

pub fn optimizer_run(module: &mut Module, optimize_flag: bool) {
    // 在功能点上对phi指令进行优化
    functional_optimizer(module);

    if optimize_flag {
        // TODO: 性能优化
    }
}

fn functional_optimizer(module: &mut Module) {
    for (_, func) in module.get_all_func().iter() {
        // 空函数不优化
        if func.is_empty_bb() {
            continue;
        }

        let end_bb = bfs_find_end_bb(func.get_head());

        // TODO: 一遍简单的冗余代码删除

        // phi优化
        phi_run(end_bb);
    }
}

fn bfs_find_end_bb(head: ObjPtr<BasicBlock>) -> ObjPtr<BasicBlock> {
    // 如果只有一个块，那么这个块就是end_bb
    if !head.has_next_bb() {
        return head;
    }

    let mut visited = HashSet::new();
    let mut queue = Vec::new();
    visited.insert(head);

    queue.insert(0, head.get_next_bb());
    while let Some(vec_bb) = queue.pop() {
        for bb in vec_bb.iter() {
            if !bb.has_next_bb() {
                return bb.clone();
            }

            if !visited.contains(bb) {
                visited.insert(*bb);
                queue.insert(0, bb.get_next_bb());
            }
        }
    }

    unreachable!("can't find end bb")
}
