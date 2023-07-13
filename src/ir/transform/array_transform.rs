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
        // 生成数组长度的全局变量
        let var = make_global_array_2_var(inst, len, pools);

        // 将生成的全局变量放入module中
        let array_name = module
            .get_all_var()
            .iter()
            .find(|(_, array)| *array == inst)
            .unwrap()
            .0
            .clone();
        var.iter().enumerate().for_each(|(i, x)| {
            println!("here i = {}, x = {:?}", i, x);
            module.push_var(format!("{}_{}", array_name, i), x.clone())
        });

        let mut gep_user = Vec::new();
        inst.get_use_list().iter().for_each(|user| {
            if let InstKind::Load = user.get_kind() {
                gep_user.extend(user.get_use_list().clone());
            } else {
                unreachable!("global_inst_transform, user: {:?}", user.get_kind());
            }
        });
        for gep in gep_user.iter() {
            let index = gep.get_gep_offset();
            gep.get_use_list().clone().iter_mut().for_each(|user| {
                debug_assert!(
                    user.get_kind() == InstKind::Store || user.get_kind() == InstKind::Load
                );
                user.set_ptr(var[index.get_int_bond() as usize]);
            })
        }
    } else {
        unreachable!()
    }
}

fn make_global_array_2_var(
    inst: ObjPtr<Inst>,
    len: i32,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> Vec<ObjPtr<Inst>> {
    let mut var = Vec::new();
    match inst.get_ir_type() {
        IrType::IntPtr => {
            let init_array = &inst.get_int_init().1;
            let array_len = init_array.len();
            for i in 0..len {
                let init = if array_len == 0 || (i as usize > array_len - 1) {
                    0
                } else {
                    init_array[i as usize].1
                };

                var.push(pools.1.make_global_int(init));
            }
        }
        IrType::FloatPtr => {
            let init_array = &inst.get_float_init().1;
            for i in 0..len {
                let init = if i as usize > init_array.len() - 1 {
                    0.0
                } else {
                    init_array[i as usize].1
                };
                var.push(pools.1.make_global_float(init));
            }
        }
        _ => unreachable!("make_global_array_2_var, ir_type: {:?}", inst.get_ir_type()),
    }
    var
}
