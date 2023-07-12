use crate::{ir::instruction::InstKind, utility::ObjPtr};

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
    gep_user.iter().all(|x| x.get_gep_offset().is_const())
}

fn global_inst_transform(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    inst: ObjPtr<Inst>,
) {
    if let InstKind::Alloca(len) = inst.get_kind() {
    } else {
        unreachable!()
    }
}
