use std::collections::HashMap;

use crate::{
    ir::{instruction::InstKind, ir_type::IrType},
    utility::ObjPtr,
};

use super::*;
pub fn array_transform(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    global_array_transform(module, pools);
    local_array_transform(module, pools);
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
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
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
    if let InstKind::Alloca(len) = inst.get_kind() {
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
            let bonding = if index > array.get_int_init().1.len() {
                0
            } else {
                array.get_int_init().1[index].1
            };

            let var = pools.1.make_global_int(bonding);
            map.insert(k, var);
            var
        } else {
            let bondind = if index > array.get_float_init().1.len() {
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
