use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        instruction::{Inst, InstKind},
        module::Module,
        tools::bfs_inst_process,
    },
    utility::ObjPtr,
};

pub fn call_optimize(module: &mut Module) -> HashSet<String> {
    // 可优化函数集合
    let mut optimizable = HashSet::new();
    // 不确定可优化函数集合
    let mut uncertain = HashMap::new();
    // 不可优化函数集合
    let mut unoptimizable = HashSet::new();

    for (name, func) in module.get_all_func().iter() {
        if name.as_str() == "main" || func.is_empty_bb() {
            // 将main函数和外部函数加入不可优化集合中
            unoptimizable.insert(format!("{}", name));
        } else {
            // 当前函数不确定是否可优化的部分
            let mut uncertain_set = HashSet::new();
            let mut flag = true;
            bfs_inst_process(func.get_head(), |inst| {
                call_removable_test_in_inst(
                    &mut flag,
                    inst,
                    &mut uncertain_set,
                    format!("{}", name),
                    &mut optimizable,
                    &mut unoptimizable,
                )
            });
            if flag {
                optimizable.insert(format!("{}", name));
            } else {
                uncertain.insert(format!("{}", name), uncertain_set);
            }
        }
    }

    loop {
        let mut changed = false;
        let mut to_remove = Vec::new();
        for (name, uncertain_set) in uncertain.iter() {
            let intersection: Vec<_> = uncertain_set.intersection(&unoptimizable).collect();
            if intersection.len() != 0 || unoptimizable.contains(name) {
                changed = true;
                to_remove.push(name.clone());
                unoptimizable.insert(name.clone());
            }
        }

        for name in to_remove.iter() {
            uncertain.remove(name);
        }

        if !changed {
            break;
        }
    }
    let optimizable = optimizable
        .union(&uncertain.keys().cloned().collect())
        .cloned()
        .collect();

    optimizable
}

fn get_gep_ptr(inst: ObjPtr<Inst>) -> ObjPtr<Inst> {
    if let InstKind::Gep = inst.get_kind() {
        get_gep_ptr(inst.get_gep_ptr())
    } else {
        inst
    }
}

fn call_removable_test_in_inst(
    flag: &mut bool,
    inst: ObjPtr<Inst>,
    uncertain_set: &mut HashSet<String>,
    name: String,
    optimizable: &mut HashSet<String>,
    unoptimizable: &mut HashSet<String>,
) {
    match inst.get_kind() {
        InstKind::Call(callee) => {
            if !optimizable.contains(&callee) {
                *flag = false;
                if unoptimizable.contains(&callee) {
                    unoptimizable.insert(format!("{}", name));
                } else {
                    uncertain_set.insert(callee);
                }
            }
        }
        InstKind::Store => {
            let ptr = get_gep_ptr(inst.get_dest());
            match ptr.get_kind() {
                InstKind::GlobalConstInt(_)
                | InstKind::GlobalConstFloat(_)
                | InstKind::GlobalInt(_)
                | InstKind::GlobalFloat(_)
                | InstKind::Parameter
                | InstKind::Load => {
                    // 全局变量和全局数组和外部数组的指针
                    unoptimizable.insert(format!("{}", name));
                    *flag = false;
                }
                InstKind::Alloca(_) => {}
                _ => {
                    unreachable!("call_optimize.rs: Gep get wrong ptr {:?}", ptr.get_kind());
                }
            }
        }
        _ => {}
    }
}
