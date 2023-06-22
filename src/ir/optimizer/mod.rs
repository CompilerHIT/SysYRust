use super::{basicblock::BasicBlock, function::Function, instruction::Inst, module::Module};
use crate::utility::{ObjPool, ObjPtr};
use std::collections::{HashSet, VecDeque};

mod dead_code_eliminate;
mod func_inline;
mod phi_optimizer;
mod simplify_cfg;

pub use func_inline::{call_map_gen, CallMap};

pub fn optimizer_run(
    module: &mut Module,
    mut pools: (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    // 在功能点上对phi指令进行优化
    functional_optimizer(module);

    if optimize_flag {
        // 死代码删除
        dead_code_eliminate::dead_code_eliminate(module, true);

        // 简化cfg
        simplify_cfg::simplify_cfg_run(module, &mut pools);

        // 函数内联
        func_inline::inline_run(module, &mut pools);
        // TODO: 性能优化
    }
}

pub fn inst_process_in_bb<F>(mut inst: ObjPtr<Inst>, mut predicate: F)
where
    F: FnMut(ObjPtr<Inst>),
{
    while !inst.is_tail() {
        // 这里需要先获取next，因为predicate可能会删除当前指令
        let next = inst.get_next();
        predicate(inst);
        inst = next;
    }
}

/// 从head开始，广度优先遍历每一个基本块，并对基本块中的指令进行处理
/// # Arguments
/// * `head` - 广度优先遍历的起始点
/// * `predicate` - 对每一个基本块进行处理的闭包，这个闭包接受一个参数，类型为inst,并对inst进行处理
pub fn bfs_inst_process<F>(head: ObjPtr<BasicBlock>, mut predicate: F)
where
    F: FnMut(ObjPtr<Inst>),
{
    bfs_bb_proceess(head, |bb| {
        inst_process_in_bb(bb.get_head_inst(), &mut predicate)
    });
}

/// 从head开始，深度优先遍历每一个基本块，并对基本块中的指令进行处理
/// # Arguments
/// * `head` - 深度优先遍历的起始点
/// * `predicate` - 对每一个基本块进行处理的闭包，这个闭包接受一个参数，类型为inst,并对inst进行处理
pub fn dfs_inst_process<F>(head: ObjPtr<BasicBlock>, mut predicate: F)
where
    F: FnMut(ObjPtr<Inst>),
{
    dfs_bb_process(head, |bb| {
        inst_process_in_bb(bb.get_head_inst(), &mut predicate)
    });
}

/// 从head开始，广度优先遍历每一个基本块，并对基本块进行处理
/// # Arguments
/// * `head` - 广度优先遍历的起始点
/// * `predicate` - 对每一个基本块进行处理的闭包，这个闭包接受一个参数，类型为bb,并对bb进行处理
pub fn bfs_bb_proceess<F>(head: ObjPtr<BasicBlock>, mut predicate: F)
where
    F: FnMut(ObjPtr<BasicBlock>),
{
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(head);
    while let Some(bb) = queue.pop_front() {
        if visited.contains(&bb) {
            continue;
        }
        visited.insert(bb);

        // 先将bb的后继节点加入队列，以防止在处理bb时把bb的结构改变
        for next_bb in bb.get_next_bb().iter() {
            queue.push_back(next_bb.clone());
        }

        predicate(bb);
    }
}

/// 从head开始，深度优先遍历每一个基本块，并对基本块进行处理
/// # Arguments
/// * `head` - 深度优先遍历的起始点
/// * `predicate` - 对每一个基本块进行处理的闭包，这个闭包接受一个参数，类型为bb,并对bb进行处理
pub fn dfs_bb_process<F>(head: ObjPtr<BasicBlock>, mut predicate: F)
where
    F: FnMut(ObjPtr<BasicBlock>),
{
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    stack.push(head);
    while let Some(bb) = stack.pop() {
        if visited.contains(&bb) {
            continue;
        }
        visited.insert(bb);
        // 先将bb的后继节点加入队列，以防止在处理bb时把bb的结构改变
        for next_bb in bb.get_next_bb().iter() {
            stack.push(next_bb.clone());
        }
        predicate(bb);
    }
}

/// 对每一个函数进行处理，但不处理外部函数
/// 对每一个函数进行处理
/// # Arguments
/// * `module`
/// * `predicate`
/// # Arguments
/// * `module` - 进行处理的模块
/// * `predicate` - 对每一个函数进行处理的闭包，这个闭包接受两个参数，第一个参数为函数名，第二个参数为函数
pub fn func_process<F>(module: &mut Module, mut predicate: F)
where
    F: FnMut(String, ObjPtr<Function>),
{
    for (name, func) in module.get_all_func().iter() {
        // 空函数不处理
        if func.is_empty_bb() {
            continue;
        }
        predicate(name.to_string(), func.clone());
    }
}

fn functional_optimizer(module: &mut Module) {
    // 一遍死代码删除
    dead_code_eliminate::dead_code_eliminate(module, false);

    // phi优化
    phi_optimizer::phi_run(module);

    // 全局死代码删除
    dead_code_eliminate::global_eliminate(module);
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
