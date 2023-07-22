use std::collections::{HashMap, HashSet};

use crate::{ir::instruction::InstKind, utility::ObjPtr};

use super::*;

/// 分析全局变量，将未被修改的全局变量转换为常量
pub fn global_var_transform(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    if !optimize_flag {
        return;
    }

    let var_vec: Vec<_> = module
        .get_all_var()
        .iter()
        .map(|x| (x.0.clone(), x.1.clone()).clone())
        .collect();

    // 将只有读的全局变量改为全局常量
    var_vec
        .iter()
        .filter(|(_, x)| x.get_kind() != InstKind::Alloca(0))
        .cloned()
        .for_each(|(name, inst)| {
            transform_nonconst_var(module, pools, name, inst);
        });

    let var_vec: Vec<_> = module
        .get_all_var()
        .iter()
        .map(|x| (x.0.clone(), x.1.clone()).clone())
        .collect();

    // 将对于全局常量的读改为读本地常量
    var_vec
        .iter()
        .filter(|(_, x)| x.is_const())
        .for_each(|(_, x)| change_glo_to_loc(*x, pools));

    // 将只有一个函数在使用的全局变量改为局部变量
    let var_vec: Vec<_> = module
        .get_all_var()
        .iter()
        .map(|x| (x.0.clone(), x.1.clone()).clone())
        .collect();
    // 初始化全局变量和使用这个全局变量的函数的map
    let mut map_var_func = var_vec
        .iter()
        .filter_map(|(_, x)| {
            if x.is_global_var() && x.get_kind() != InstKind::Alloca(0) {
                Some((x.clone(), HashSet::<String>::new()))
            } else {
                None
            }
        })
        .collect::<HashMap<_, _>>();

    // 分析全局变量的使用情况
    func_process(module, |name, func| {
        bfs_inst_process(func.get_head(), |inst| {
            if inst.get_kind() == InstKind::Load {
                let var = inst.get_ptr();
                if var.is_global_var() && var.get_kind() != InstKind::Alloca(0) {
                    map_var_func.get_mut(&var).unwrap().insert(name.clone());
                }
            } else if inst.get_kind() == InstKind::Store {
                let var = inst.get_dest();
                if var.is_global_var() && var.get_kind() != InstKind::Alloca(0) {
                    map_var_func.get_mut(&var).unwrap().insert(name.clone());
                }
            }
        })
    });

    // 将只有一个函数在使用的全局变量改为局部变量
    map_var_func
        .iter()
        .filter(|(_, set)| set.len() == 1)
        .for_each(|(inst, set)| {
            let new_store;
            let init;
            if inst.get_ir_type().is_int() {
                init = pools.1.make_int_const(inst.get_int_bond());
                new_store = pools.1.make_int_store(*inst, init);
            } else {
                debug_assert!(inst.get_ir_type().is_float());
                init = pools.1.make_float_const(inst.get_float_bond());
                new_store = pools.1.make_float_store(*inst, init);
            }
            let mut head_bb = module.get_function(set.iter().next().unwrap()).get_head();
            head_bb.push_front(init);
            head_bb.push_front(new_store);
            change_one_func_use_glo_to_loc(*inst, pools);
        });
}

fn change_one_func_use_glo_to_loc(
    inst: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut var_set = HashSet::new();
    // 先将store的全局变量改为使用store的值
    inst.get_use_list()
        .clone()
        .iter_mut()
        .filter(|x| x.get_kind() == InstKind::Store)
        .for_each(|x| {
            var_set.insert(x.get_value());
            x.remove_self();
        });

    // 将load的全局变量改为使用store的值
    inst.get_use_list()
        .clone()
        .iter_mut()
        .filter(|x| x.get_kind() == InstKind::Load)
        .for_each(|x| {
            let mut find = false;
            let mut inst = x.get_prev();
            while !inst.is_tail() {
                if var_set.contains(&inst) {
                    find = true;
                    x.get_use_list().clone().iter_mut().for_each(|user| {
                        let index = user.get_operand_index(*x);
                        user.set_operand(inst, index);
                    });
                    break;
                }
                inst = inst.get_prev();
            }

            if !find {
                let phi = pools.1.make_phi(x.get_ir_type());
                x.get_parent_bb().push_front(phi);
                value_set(phi, &mut var_set, pools);
                x.get_use_list().clone().iter_mut().for_each(|user| {
                    let index = user.get_operand_index(*x);
                    user.set_operand(phi, index);
                });
            }
        })
}

fn value_set(
    mut phi: ObjPtr<Inst>,
    set: &mut HashSet<ObjPtr<Inst>>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    set.insert(phi);
    phi.get_parent_bb()
        .get_up_bb()
        .clone()
        .iter_mut()
        .for_each(|bb| {
            let mut inst = bb.get_tail_inst();
            let mut find = false;
            while !inst.is_tail() {
                if set.contains(&inst) {
                    find = true;
                    phi.add_operand(inst);
                    break;
                }

                inst = inst.get_prev();
            }

            if !find {
                let new_phi = pools.1.make_phi(phi.get_ir_type());
                bb.push_front(new_phi);
                phi.add_operand(new_phi);
                value_set(new_phi, set, pools);
            }
        });
}

fn transform_nonconst_var(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    name: String,
    inst: ObjPtr<Inst>,
) {
    match inst.get_kind() {
        InstKind::GlobalInt(init) => {
            if inst
                .get_use_list()
                .iter()
                .all(|user| user.get_kind() != InstKind::Store)
            {
                let new_inst = pools.1.make_global_int_const(init);
                inst.get_use_list().clone().iter_mut().for_each(|user| {
                    let index = user.get_operand_index(inst);
                    user.set_operand(new_inst, index);
                });
                module.push_var(format!("{}_const", name), new_inst);
            }
        }
        InstKind::GlobalFloat(init) => {
            if inst
                .get_use_list()
                .iter()
                .all(|user| user.get_kind() != InstKind::Store)
            {
                let new_inst = pools.1.make_global_float_const(init);
                inst.get_use_list().clone().iter_mut().for_each(|user| {
                    let index = user.get_operand_index(inst);
                    user.set_operand(new_inst, index);
                });
                module.push_var(format!("{}_const", name), new_inst);
            }
        }
        InstKind::GlobalConstInt(_) | InstKind::GlobalConstFloat(_) => {}
        _ => unreachable!(),
    }
}

fn change_glo_to_loc(
    inst: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    debug_assert!(inst.is_const());
    inst.get_use_list().clone().iter_mut().for_each(|user| {
        debug_assert_eq!(user.get_kind(), InstKind::Load, "user: {:?}", user);
        let new_inst = match inst.get_kind() {
            InstKind::GlobalConstInt(init) => pools.1.make_int_const(init),
            InstKind::GlobalConstFloat(init) => pools.1.make_float_const(init),
            _ => unreachable!(),
        };
        user.get_use_list().clone().iter_mut().for_each(|x| {
            let index = x.get_operand_index(*user);
            x.set_operand(new_inst, index);
        });
        user.insert_before(new_inst);
        user.remove_self();
    })
}
