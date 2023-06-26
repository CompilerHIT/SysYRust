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

use super::*;
use crate::{
    ir::{analysis::call_optimize::call_optimize, instruction::InstKind, module::Module},
    utility::ObjPtr,
};

pub fn dead_code_eliminate(module: &mut Module, func_call_eliminate: bool) {
    let mut func_optimize = HashSet::new();
    if func_call_eliminate {
        func_optimize = call_optimize(module);
    }
    loop {
        let mut changed = false;
        func_process(module, |_, func| {
            bfs_inst_process(func.get_head(), |inst| {
                changed |= eliminate_inst(inst, func_call_eliminate, &func_optimize);
            })
        });

        if !changed {
            break;
        }
    }
}

pub fn global_eliminate(module: &mut Module) {
    let mut delete_list = Vec::new();

    for (name, value) in module.get_all_var().iter() {
        if value.get_use_list().len() == 0 {
            delete_list.push(name.to_owned().to_owned());
        }
    }

    for name in delete_list {
        module.remove_var(name.as_str());
    }
}

fn eliminate_inst(
    mut inst: ObjPtr<Inst>,
    func_call_eliminate: bool,
    func_optimize: &HashSet<String>,
) -> bool {
    let mut changed = false;
    if inst.get_use_list().len() == 0 {
        match inst.get_kind() {
            InstKind::Call(callee) => {
                if func_call_eliminate && func_optimize.contains(&callee) {
                    changed = true;
                    inst.remove_self();
                }
            }
            InstKind::Store | InstKind::Return | InstKind::Branch => {}
            _ => {
                changed = true;
                inst.remove_self();
            }
        }
    }

    changed
}
