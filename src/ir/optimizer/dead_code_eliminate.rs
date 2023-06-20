//! 死代码删除
//! 基本块内的死代码删除
//! 根据inst的use_list的长度来判断
//! 长度为0但不是死代码的有：
//! 1. call指令
//! 2. store指令
//! 3. ret指令
//! 4. br指令
//! 5. Head指令

use super::*;
use crate::{
    ir::{instruction::InstKind, module::Module},
    utility::ObjPtr,
};

pub fn dead_code_eliminate(module: &mut Module, more_than_once: bool) {
    let mut changed = false;
    loop {
        func_process(module, |name, func| {
            bfs_inst_process(func.get_head(), |inst| {
                changed |= eliminate_inst(inst);
            })
        });

        if !more_than_once || !changed {
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

fn eliminate_inst(mut inst: ObjPtr<Inst>) -> bool {
    let mut changed = false;
    if inst.get_use_list().len() == 0 {
        match inst.get_kind() {
            InstKind::Call(_) | InstKind::Store | InstKind::Return | InstKind::Branch => {}
            _ => {
                changed = true;
                inst.remove_self();
            }
        }
    }

    changed
}
