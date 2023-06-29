use crate::{
    ir::{
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
        tools::bfs_inst_process,
    },
    utility::{ObjPool, ObjPtr},
};

///! 在函数内联之后可能会出现gep计算的指针是一个gep的情况，这种情况可以将两个gep合并为一个gep
pub fn gep_optimize(
    head_bb: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    loop {
        let mut changed = false;
        bfs_inst_process(head_bb, |inst| {
            if let InstKind::Gep = inst.get_kind() {
                if let InstKind::Gep = inst.get_gep_ptr().get_kind() {
                    changed = true;
                    inst_gep_optimize(inst, pools);
                }
            }
        });

        if !changed {
            break;
        }
    }
}

fn inst_gep_optimize(
    inst: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let new_add = get_new_add(
        inst.get_gep_offset(),
        inst.get_gep_ptr().get_gep_offset(),
        pools,
    );
    inst.as_mut().insert_before(new_add);
    inst.as_mut().set_gep_offset(new_add);
    inst.as_mut().set_gep_ptr(inst.get_gep_ptr().get_gep_ptr());
}

fn get_new_add(
    lhs: ObjPtr<Inst>,
    rhs: ObjPtr<Inst>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<Inst> {
    match (lhs.get_kind(), rhs.get_kind()) {
        (InstKind::ConstInt(val_lhs), InstKind::ConstInt(val_rhs)) => {
            let new_val = val_lhs + val_rhs;
            pools.1.make_int_const(new_val)
        }
        (InstKind::GlobalConstInt(val_lhs), InstKind::ConstInt(val_rhs)) => {
            let new_val = val_lhs + val_rhs;
            pools.1.make_int_const(new_val)
        }
        (InstKind::ConstInt(val_lhs), InstKind::GlobalConstInt(val_rhs)) => {
            let new_val = val_lhs + val_rhs;
            pools.1.make_int_const(new_val)
        }
        (InstKind::GlobalConstInt(val_lhs), InstKind::GlobalConstInt(val_rhs)) => {
            let new_val = val_lhs + val_rhs;
            pools.1.make_int_const(new_val)
        }
        _ => pools.1.make_add(lhs, rhs),
    }
}
