use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        analysis::dominator_tree::{calculate_dominator, DominatorTree},
        basicblock::BasicBlock,
        function::Function,
        module::Module,
        tools::func_process,
    },
    utility::ObjPtr,
};

use super::{LoopInfo, LoopList};

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
                let blocks = recognize_one_loop(&mut loop_list, bb, latch, &mut visited, &dom_tree);
                visited.extend(blocks.blocks.clone());
            }
        }
    });

    loop_list
}

fn recognize_one_loop(
    loop_list: &mut LoopList,
    header: ObjPtr<BasicBlock>,
    latchs: Vec<ObjPtr<BasicBlock>>,
    visited: &mut HashSet<ObjPtr<BasicBlock>>,
    dom_tree: &DominatorTree,
) -> ObjPtr<LoopInfo> {
    let mut tree = loop_list.pool.put(LoopInfo::new(header));
    tree.blocks.push(header);
    // 将header加入visited
    visited.insert(header);

    for latch in latchs {
        let mut stack = vec![latch];
        while let Some(bb) = stack.pop() {
            // 将当前块的前继中没有在循环中的块加入stack
            bb.get_up_bb().iter().for_each(|up_bb| {
                if up_bb != &header && !dom_tree.is_dominate(&bb, up_bb) {
                    stack.push(up_bb.clone());
                }
            });

            // 找到当前块所在的循环,并设置相应的子循环和父循环
            if visited.contains(&bb) {
                if let Some(sub_loop) = loop_list
                    .loops
                    .iter_mut()
                    .find(|loop_tree| loop_tree.blocks.contains(&bb))
                {
                    debug_assert!(
                        sub_loop.parent.is_none() || sub_loop.parent == Some(tree.clone())
                    );
                    sub_loop.parent = Some(tree.clone());
                    tree.sub_loops.push(sub_loop.clone());
                } else if !tree.blocks.contains(&bb) {
                    tree.blocks.push(bb);
                }
            } else {
                tree.blocks.push(bb);
                stack.extend(bb.get_up_bb().iter().cloned());
                visited.insert(bb);
            }
        }
    }

    loop_list.loops.push(tree.clone());

    tree
}
