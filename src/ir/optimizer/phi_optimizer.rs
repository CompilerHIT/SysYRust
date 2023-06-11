use std::collections::HashSet;

use crate::{
    ir::{
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
    },
    utility::ObjPtr,
};

///! 对于phi的优化主要针对以下几个方面：
///! 1. phi的参数只有一个时，直接替换为该参数
///! 2. phi的参数是自己时，删掉
///! 3. phi的多个参数中有相同的时，删除重复的参数

pub fn phi_run(end: ObjPtr<BasicBlock>) {
    let mut visited = HashSet::new();
    let mut queue = Vec::new();
    queue.insert(0, end);

    // 往回广度优先遍历
    while let Some(bb) = queue.pop() {
        if !visited.contains(&bb) {
            visited.insert(bb);
            phi_optimize(bb);

            for prev in bb.get_up_bb().iter() {
                queue.insert(0, *prev);
            }
        }
    }
}

/// 对于每个基本块，对其中的phi指令进行优化
fn phi_optimize(bb: ObjPtr<BasicBlock>) {
    for phi in get_phi_list(bb) {
        let op = phi.get_operands();
        match op.len() {
            0 => unreachable!("phi has no operand"),
            1 => {
                trace_single_phi(phi);
            }
            _ => {
                remove_self_phi(phi);
                remove_duplicate_phi(phi)
            }
        }
    }
}

/// 删除phi中的重复参数
fn remove_duplicate_phi(mut phi: ObjPtr<Inst>) {
    let mut set = HashSet::new();
    for &op in phi.get_operands() {
        if !set.contains(&op) {
            set.insert(op);
        } else {
            op.as_mut().remove_user(phi.as_ref());
        }
    }

    phi.set_operands(set.into_iter().collect());
}

/// 删除参数为自身的phi
fn remove_self_phi(mut phi: ObjPtr<Inst>) {
    //while let Some(index) = phi.get_operands().iter().position(|&x| x == phi) {
    //phi.remove_operand_by_index(index);
    //}
    loop {
        if let Some(index) = phi.get_operands().iter().position(|&x| x == phi) {
            println!("{index}");
            debug_assert_eq!(phi, phi.get_operands()[index]);
            phi.remove_operand_by_index(index);
        } else {
            break;
        }
    }
}

/// 递归清除phi的单参数问题
fn trace_single_phi(mut phi: ObjPtr<Inst>) {
    // 追踪该参数，匹配参数的类型
    let op = phi.get_operands()[0];

    let mut replace = || {
        for user in phi.get_use_list().iter() {
            user.as_mut().replace_operand(phi, op);
        }
        phi.remove_self();
    };

    match op.get_kind() {
        // 当参数是phi时，进行判断
        InstKind::Phi => match op.get_operands().len() {
            0 => unreachable!("phi has no operand"),
            // 当参数是单参数时，递归清除
            1 => {
                replace();
                trace_single_phi(op);
            }
            _ => {
                replace();
            }
        },
        // 其他情况，直接替换
        _ => {
            replace();
        }
    }
}

/// 获得基本块中的phi指令列表
fn get_phi_list(bb: ObjPtr<BasicBlock>) -> Vec<ObjPtr<Inst>> {
    if bb.is_empty() {
        Vec::new()
    } else {
        let mut phi_list = Vec::new();
        let mut inst = bb.get_head_inst();
        loop {
            if let InstKind::Phi = inst.get_kind() {
                phi_list.push(inst);
                inst = inst.get_next();
            } else {
                break;
            }
        }
        phi_list
    }
}
