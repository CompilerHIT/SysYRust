use crate::{ir::instruction::InstKind, utility::ObjPtr};

use super::*;

/// 分析全局变量，将未被修改的全局变量转换为常量
pub fn global_var_transform(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
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

    // 将对于全局常量的读改为读本地常量
    var_vec
        .iter()
        .filter(|(_, x)| x.is_const())
        .for_each(|(_, x)| change_glo_to_loc(*x, pools))
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
