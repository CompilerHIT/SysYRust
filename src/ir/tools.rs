use std::collections::{HashSet, VecDeque};

use crate::utility::ObjPtr;

use super::{basicblock::BasicBlock, function::Function, instruction::Inst, module::Module};

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

/// 从head开始，深度优先先序遍历每一个基本块，并对基本块进行处理
/// # Arguments
/// * `head` - 深度优先遍历的起始点
/// * `predicate` - 对每一个基本块进行处理的闭包，这个闭包接受一个参数，类型为bb,并对bb进行处理
pub fn dfs_pre_order_bb_process<F>(head: ObjPtr<BasicBlock>, mut predicate: F)
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

/// 从head开始，深度优先后序遍历每一个基本块，并对基本块进行处理
/// # Arguments
/// * `head` - 深度优先遍历的起始点
/// * `predicate` - 对每一个基本块进行处理的闭包，这个闭包接受一个参数，类型为bb,并对bb进行处理
pub fn dfs_post_order_bb_process<F>(head: ObjPtr<BasicBlock>, mut predicate: F)
where
    F: FnMut(ObjPtr<BasicBlock>),
{
    let mut visited = HashSet::new();
    let mut stack = Vec::new();
    stack.push(head);
    while let Some(bb) = stack.pop() {
        if bb.get_next_bb().is_empty() || bb.get_next_bb().iter().all(|x| visited.contains(x)) {
            visited.insert(bb);
            predicate(bb);
        } else {
            stack.push(bb);
            for next_bb in bb.get_next_bb().iter() {
                if !visited.contains(next_bb) {
                    stack.push(next_bb.clone());
                }
            }
        }
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

pub fn bfs_find_end_bb(head: ObjPtr<BasicBlock>) -> ObjPtr<BasicBlock> {
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

/// 用一条指令代替另一条指令
/// # Arguments
/// * 'inst_old'
/// * 'inst_new'
/// # Arguments
/// * 'inst_old' - 被代替的指令
/// * 'inst_new' - 新任命的指令
pub fn replace_inst(inst_old: ObjPtr<Inst>, inst_new: ObjPtr<Inst>) {
    let use_list = inst_old.get_use_list().clone();
    for user in use_list {
        let index = user.get_operand_index(inst_old);
        user.as_mut().set_operand(inst_new, index);
    }
    inst_old.as_mut().remove_self();
}
