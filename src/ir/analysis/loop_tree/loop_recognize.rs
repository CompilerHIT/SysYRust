use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        analysis::dominator_tree::calculate_dominator, basicblock::BasicBlock, function::Function,
        module::Module, tools::func_process,
    },
    utility::ObjPtr,
};

use super::{LoopList, LoopTree};

/// 循环的识别
/// # Params
/// * `module` - 待识别的模块
/// # Return
/// * `HashMap<String, LoopList>` - 循环的识别结果，key为函数名，value为循环列表。
///Note: 子循环也会在LoopList中，所以在处理的时候可以直接遍历列表，
///然后处理当前循环即可，不用再去处理当前循环的子循环
pub fn loop_recognize(module: &mut Module) -> HashMap<String, LoopList> {
    let mut loops = HashMap::new();
    func_process(module, |name, func| {
        loops.insert(name, loop_recognize_in_function(func));
    });
    loops
}

fn loop_recognize_in_function(func: ObjPtr<Function>) -> LoopList {
    let mut loop_list = LoopList::new();
    let mut visited = HashSet::new();

    let dom_tree = calculate_dominator(func.get_head());
    dom_tree.iter_post_order(|bb| {
        if !visited.contains(&bb) {
            let mut latch = Vec::new();
            bb.get_up_bb().iter().for_each(|up_bb| {
                if dom_tree.is_dominate(&bb, up_bb) {
                    latch.push(up_bb.clone());
                }
            });
            if latch.len() == 0 {
                visited.insert(bb);
            } else {
                visited.extend(recognize_one_loop(&mut loop_list, bb, latch).blocks.clone());
            }
        }
    });

    loop_list
}

fn recognize_one_loop(
    loop_list: &mut LoopList,
    header: ObjPtr<BasicBlock>,
    latchs: Vec<ObjPtr<BasicBlock>>,
) -> ObjPtr<LoopTree> {
    let mut tree = loop_list.pool.put(LoopTree::new(header));
    tree.blocks.push(header);
    let mut visited = HashSet::new();
    // 将header加入visited
    visited.insert(header);

    for latch in latchs {
        let mut stack = vec![latch];
        while let Some(bb) = stack.pop() {
            crate::log_file!("dead_loop", "here {}", bb.get_name());
            if visited.contains(&bb) {
                continue;
            }

            // 如果当前基本块已经在别的循环中
            if loop_list
                .loops
                .iter()
                .any(|loop_tree| loop_tree.blocks.contains(&bb))
            {
                // 将当前块的前继中没有在循环中的块加入stack
                bb.get_up_bb().iter().for_each(|up_bb| {
                    if loop_list
                        .loops
                        .iter()
                        .all(|loop_tree| !loop_tree.blocks.contains(up_bb))
                    {
                        stack.push(up_bb.clone());
                    }
                });

                // 找到当前块所在的循环,并设置相应的子循环和父循环
                let sub_loop = loop_list
                    .loops
                    .iter_mut()
                    .find(|loop_tree| loop_tree.blocks.contains(&bb))
                    .unwrap();
                debug_assert_eq!(sub_loop.parent, None, "loop tree parent should be none");
                sub_loop.parent = Some(tree.clone());
                tree.sub_loops.push(sub_loop.clone());
            } else {
                tree.blocks.push(bb);
                stack.extend(bb.get_up_bb().iter().cloned());
            }
            visited.insert(bb);
        }
    }

    loop_list.loops.push(tree.clone());

    tree
}
