use std::collections::{HashMap, HashSet};

use crate::{ir::instruction::InstKind, utility::ObjPtr};

use super::*;
pub fn return_unused(module: &mut Module) {
    let funcs = module.get_all_func();
    let mut call_map = HashMap::new();
    funcs.iter().for_each(|(name, func)| {
        if name.as_str() != "main"
            && !func.is_empty_bb()
            && func.get_return_type() != crate::ir::ir_type::IrType::Void
        {
            call_map.insert(name.to_string(), HashSet::<ObjPtr<Inst>>::new());
        }
    });

    func_process(module, |func_name, func| {
        bfs_inst_process(func.get_head(), |inst| {
            if let InstKind::Call(callee) = inst.get_kind() {
                if call_map.contains_key(&callee) {
                    if inst.get_use_list().len() == 0
                        || (func_name == callee
                            && inst.get_use_list().iter().all(|x| {
                                x.is_return()
                                    || x.is_phi() && x.get_use_list().iter().all(|y| y.is_return())
                            }))
                    {
                        call_map.get_mut(&callee).unwrap().insert(inst);
                    } else {
                        call_map.remove_entry(&callee);
                    }
                }
            }
        });
    });

    call_map.iter().for_each(|(callee, call_set)| {
        // 将函数的返回值类型设置为void
        // 删除函数的所有返回指令
        let mut func = module.get_function(&callee);
        func.set_return_type(crate::ir::ir_type::IrType::Void);
        // 深度优先寻找exit块
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        stack.push(func.get_head());
        while let Some(bb) = stack.pop() {
            if visited.contains(&bb) {
                continue;
            }
            if bb.is_exit() {
                // 找到后把exit块的最后一条指令改为return void
                let mut return_inst = bb.get_tail_inst();
                debug_assert_eq!(return_inst.get_kind(), InstKind::Return);
                return_inst.remove_operand_by_index(0);
                return_inst.set_ir_type(crate::ir::ir_type::IrType::Void);

                break;
            }
            visited.insert(bb);
            stack.extend(bb.get_next_bb());
        }

        // 将所有调用该函数的指令的返回值类型设置为void
        call_set.iter().for_each(|inst| {
            inst.as_mut().set_ir_type(crate::ir::ir_type::IrType::Void);
        });
    });
}
