use super::*;
use crate::{
    ir::instruction::{Inst, InstKind},
    utility::ObjPtr,
};

///! 对于phi的优化主要针对以下几个方面：
///! 1. phi的参数只有一个时，直接替换为该参数;
///! 2. phi的多个参数相同时，也可以将其消去。
///! 3. phi的参数中多个参数相同，剩下的都指向自己，也可以将其消去。

pub fn phi_run(module: &mut Module) {
    loop {
        let mut changed = false;
        func_process(module, |_, func| {
            bfs_inst_process(func.get_head(), |inst| {
                changed |= phi_optimize(inst);
            })
        });

        if !changed {
            break;
        }
    }
}

fn phi_optimize(mut inst: ObjPtr<Inst>) -> bool {
    let mut changed = false;
    if let InstKind::Phi = inst.get_kind() {
        let not_self = inst
            .get_operands()
            .iter()
            .find(|&&x| x != inst)
            .unwrap()
            .clone();
        if inst
            .get_operands()
            .iter()
            .all(|&x| x == inst || x == not_self)
        {
            changed = true;
            // 将其user中所有使用当前phi的操作数替换为not_self
            while inst.get_use_list().len() != 0 {
                let mut user = inst.get_use_list()[0];
                let index = user.get_operands().iter().position(|x| x == &inst).unwrap();
                user.set_operand(not_self, index);
            }
            inst.remove_self();
        }
    }

    changed
}
