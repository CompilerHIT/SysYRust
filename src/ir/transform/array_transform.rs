use std::collections::HashMap;

use crate::{
    ir::{instruction::InstKind, ir_type::IrType},
    utility::ObjPtr,
};

use super::*;

pub fn array_optimize(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    if optimize_flag {
        array_transform(module, pools);
        array_init_optimize(module);
        array_first_load_optimize(module, pools);
    }
}

/// 将对数组索引都是编译时已知的数组进行变量化
fn array_transform(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    global_array_transform(module, pools);
    local_array_transform(module, pools);
}

/// 对局部数组的store指令转变为初始化
fn array_init_optimize(module: &mut Module) {
    func_process(module, |_, func| {
        bfs_inst_process(func.get_head(), |inst| {
            if let InstKind::Alloca(_) = inst.get_kind() {
                reinit_array(inst);
            }
        })
    })
}

/// 对局部数组的第一次load指令的优化
fn array_first_load_optimize(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    func_process(module, |_, func| {
        bfs_inst_process(func.get_head(), |inst| {
            if let InstKind::Alloca(_) = inst.get_kind() {
                first_load_optimize(inst, pools);
            }
        })
    })
}

fn first_load_optimize(
    array_inst: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut inst = array_inst.get_next();
    while !inst.is_tail() {
        if inst.get_kind() == InstKind::Call("Nothing".to_string())
            && inst.get_operands().iter().any(|&op| {
                if let InstKind::Gep = op.get_kind() {
                    op.get_gep_ptr() == array_inst
                } else {
                    op == array_inst
                }
            })
        {
            break;
        }

        let mut flag = false;
        if inst.get_kind() == InstKind::Load
            && !inst.is_global_var_load()
            && !inst.is_global_array_load()
            && inst.get_ptr().get_gep_ptr() == array_inst
            && inst.get_ptr().get_gep_offset().is_const()
        {
            let index = inst.get_ptr().get_gep_offset().get_int_bond() as usize;
            flag = true;
            if IrType::IntPtr == array_inst.get_ir_type() {
                let init_array = &array_inst.get_int_init().1;
                let init;
                if index >= init_array.len() {
                    init = 0;
                } else {
                    init = init_array[index].1;
                }
                let value = pools.1.make_int_const(init);
                inst.insert_before(value);
                inst.get_use_list().clone().iter_mut().for_each(|user| {
                    let index = user.get_operand_index(inst);
                    user.set_operand(value, index);
                })
            } else if IrType::FloatPtr == array_inst.get_ir_type() {
                let init_array = &array_inst.get_float_init().1;
                let init;
                if index >= init_array.len() {
                    init = 0.0;
                } else {
                    init = init_array[index].1;
                }

                let value = pools.1.make_float_const(init);
                inst.insert_before(value);
                inst.get_use_list().clone().iter_mut().for_each(|user| {
                    let index = user.get_operand_index(inst);
                    user.set_operand(value, index);
                })
            } else {
                unreachable!(
                    "first_load_optimize: {:?} ir_type not support {:?}",
                    array_inst,
                    array_inst.get_ir_type()
                );
            }
        }
        let mut old = inst;
        inst = inst.get_next();
        if flag {
            old.remove_self();
        }
    }
}

fn reinit_array(mut array_inst: ObjPtr<Inst>) {
    let mut inst = array_inst.get_next();

    let check = |inst: ObjPtr<Inst>| {
        inst.get_kind() == InstKind::Store
            && !inst.is_global_var_store()
            && inst.get_dest().get_gep_ptr() == array_inst
            && inst.get_dest().get_gep_offset().is_const()
    };

    if IrType::IntPtr == array_inst.get_ir_type() {
        let mut init = array_inst.get_int_init().1.clone();
        while !inst.is_tail() {
            let mut flag = false;
            if check(inst) {
                let index = inst.get_dest().get_gep_offset().get_int_bond() as usize;
                if index >= init.len() {
                    init.resize(index + 1, (false, 0));
                }
                if inst.get_value().is_const() {
                    flag = true;
                    init[index] = (false, inst.get_value().get_int_bond());
                } else {
                    init[index] = (true, 0);
                }
            }
            let mut old = inst;
            inst = inst.get_next();
            if flag {
                old.remove_self();
            }
        }
        array_inst.set_int_init(true, init);
    } else if IrType::FloatPtr == array_inst.get_ir_type() {
        let mut init = array_inst.get_float_init().1.clone();
        while !inst.is_tail() {
            let mut flag = false;
            if check(inst) {
                let index = inst.get_dest().get_gep_offset().get_int_bond() as usize;
                if index >= init.len() {
                    init.resize(index + 1, (false, 0.0));
                }
                if inst.get_value().is_const() {
                    flag = true;
                    init[index] = (false, inst.get_value().get_float_bond());
                } else {
                    init[index] = (true, 0.0);
                }
            }
            let mut old = inst;
            inst = inst.get_next();
            if flag {
                old.remove_self();
            }
        }
        array_inst.set_float_init(true, init);
    } else {
        unreachable!(
            "reinit_array error: {:?} ir type {:?}",
            array_inst,
            array_inst.get_ir_type()
        );
    }
}

fn global_array_transform(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let var_vec: Vec<_> = module
        .get_all_var()
        .iter()
        .map(|x| (x.0.clone(), x.1.clone()).clone())
        .collect();
    var_vec
        .iter()
        .filter(|(_, x)| x.get_kind() == InstKind::Alloca(0))
        .cloned()
        .for_each(|(name, inst)| {
            if array_analyze(inst) {
                global_inst_transform(module, pools, inst);
                module.delete_var(&name);
            }
        })
}

fn local_array_transform(
    _module: &mut Module,
    _pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    // TODO: 本地数组优化
    // 目前发现这个东西不值得做
}

fn array_analyze(inst: ObjPtr<Inst>) -> bool {
    debug_assert_eq!(inst.get_kind(), InstKind::Alloca(0));
    let mut gep_user = Vec::new();
    inst.get_use_list().iter().for_each(|user| {
        if let InstKind::Load = user.get_kind() {
            gep_user.extend(user.get_use_list());
        } else {
            debug_assert_eq!(user.get_kind(), InstKind::Gep);
            gep_user.push(user);
        }
    });
    gep_user.iter().all(|x| {
        x.get_gep_offset().is_const()
            && x.get_use_list()
                .iter()
                .all(|user| user.get_kind() != InstKind::Call("None".to_string()))
    })
}

fn global_inst_transform(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    inst: ObjPtr<Inst>,
) {
    if let InstKind::Alloca(_len) = inst.get_kind() {
        // 将相应的全局变量放入一个HashMap中，等使用到这个变量的时候再生成
        let mut var = HashMap::new();

        // 获得数组的名字
        let array_name = module
            .get_all_var()
            .iter()
            .find(|(_, array)| *array == inst)
            .unwrap()
            .0
            .clone();

        // 获得所有使用数组的gep指令
        let mut gep_user = Vec::new();
        inst.get_use_list().iter().for_each(|user| {
            if let InstKind::Load = user.get_kind() {
                gep_user.extend(user.get_use_list().clone());
            } else {
                unreachable!("global_inst_transform, user: {:?}", user.get_kind());
            }
        });

        // 将这些gep指令的使用者的ptr替换为相应的全局变量
        for gep in gep_user.iter() {
            let index = gep.get_gep_offset();
            gep.get_use_list().clone().iter_mut().for_each(|user| {
                debug_assert!(
                    user.get_kind() == InstKind::Store || user.get_kind() == InstKind::Load
                );
                user.set_ptr(variable_get(
                    inst,
                    pools,
                    &array_name,
                    index.get_int_bond() as usize,
                    &mut var,
                ));
            })
        }

        // 将申请的全局变量放入module中
        let var: Vec<_> = var
            .iter()
            .map(|(name, inst)| (name.clone(), inst.clone()))
            .collect();
        for (name, inst) in var {
            module.push_var(name, inst);
        }
    } else {
        unreachable!()
    }
}

/// 从HashMap中获得相应的全局变量，如果没有，则生成
fn variable_get(
    array: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    array_name: &String,
    index: usize,
    map: &mut HashMap<String, ObjPtr<Inst>>,
) -> ObjPtr<Inst> {
    let k = format!("{}_{}", array_name, index);
    if map.contains_key(&k) {
        map.get(&k).unwrap().clone()
    } else {
        if let IrType::IntPtr = array.get_ir_type() {
            let bonding = if index >= array.get_int_init().1.len() {
                0
            } else {
                array.get_int_init().1[index].1
            };

            let var = pools.1.make_global_int(bonding);
            map.insert(k, var);
            var
        } else {
            let bondind = if index >= array.get_float_init().1.len() {
                0.0
            } else {
                array.get_float_init().1[index].1
            };
            let var = pools.1.make_global_float(bondind);
            map.insert(k, var);
            var
        }
    }
}
