use std::collections::HashMap;

use crate::{
    ir::{
        basicblock::BasicBlock,
        function::Function,
        instruction::{Inst, InstKind},
        module::Module,
        tools::{bfs_bb_proceess, bfs_inst_process, func_process, inst_process_in_bb},
    },
    utility::ObjPtr,
};

pub fn verify_run(module: &mut Module) -> bool {
    func_process(module, |name, func| {
        func_verify(name, func);
    });
    true
}

fn func_verify(_name: String, func: ObjPtr<Function>) {
    // 构造inst和operands的map
    let mut inst_map = HashMap::new();
    bfs_inst_process(func.get_head(), |inst| {
        inst_map.insert(inst, inst.get_operands().clone());
    });

    // 构造bb和up bb的map
    let mut bb_map = HashMap::new();
    bfs_bb_proceess(func.get_head(), |bb| {
        bb_map.insert(bb, bb.get_up_bb().clone());
    });

    bfs_bb_proceess(func.get_head(), |bb| {
        bb_verify(bb, &mut inst_map, &mut bb_map);
    });

    // 检查bb_map剩下的bb
    for (_, up_bb) in bb_map {
        debug_assert!(up_bb.len() == 0);
    }

    // 检查inst_map剩下的inst
    for (inst, operands) in inst_map {
        for op in operands {
            debug_assert!(
                op.is_param() || op.is_global_var(),
                "inst: {:?} in bb: {}, op: {:?} in bb : {}",
                inst.get_kind(),
                inst.get_parent_bb().get_name(),
                op.get_kind(),
                op.get_parent_bb().get_name()
            );
        }
    }
}

fn bb_verify(
    bb: ObjPtr<BasicBlock>,
    inst_map: &mut HashMap<ObjPtr<Inst>, Vec<ObjPtr<Inst>>>,
    bb_map: &mut HashMap<ObjPtr<BasicBlock>, Vec<ObjPtr<BasicBlock>>>,
) {
    // 检查next bb 和 up bb会不会有重复的
    let next_bb = bb.get_next_bb();
    debug_assert!(
        next_bb.len() <= 2,
        "next bb num > 2, len: {}",
        next_bb.len()
    );
    for next in next_bb {
        let up_bb = bb_map.get_mut(next);
        debug_assert_ne!(up_bb, None);
        let up_bb = up_bb.unwrap();
        let index = up_bb.iter().position(|x| x == &bb).unwrap();
        up_bb.remove(index);
        if up_bb.len() == 0 {
            bb_map.remove(next);
        }
    }

    inst_process_in_bb(bb.get_head_inst(), |inst| {
        inst_verify(inst, inst_map);
    });
}

fn inst_verify(inst: ObjPtr<Inst>, inst_map: &mut HashMap<ObjPtr<Inst>, Vec<ObjPtr<Inst>>>) {
    let prev = inst.get_prev();
    let next = inst.get_next();
    debug_assert_ne!(prev, inst, "prev == inst: {:?}", inst.get_kind());
    debug_assert_ne!(next, inst, "next == inst: {:?}", inst.get_kind());
    debug_assert!(
        if prev == next {
            prev.get_kind() == InstKind::Head
        } else {
            true
        },
        "prev == next: {:?}, prev & inst: {:?}",
        inst,
        prev
    );

    debug_assert_ne!(inst.get_kind(), InstKind::Parameter);
    debug_assert_ne!(inst.get_kind(), InstKind::GlobalConstInt(0));
    debug_assert_ne!(inst.get_kind(), InstKind::GlobalConstFloat(0.0));
    debug_assert_ne!(inst.get_kind(), InstKind::GlobalInt(0));
    debug_assert_ne!(inst.get_kind(), InstKind::GlobalFloat(0.0));

    for user in inst.get_use_list() {
        let user_operands = inst_map.get_mut(user);
        debug_assert_ne!(user_operands, None);
        let user_operands = user_operands.unwrap();
        debug_assert!(user_operands.contains(&inst));
        user_operands.remove(user_operands.iter().position(|x| x == &inst).unwrap());
        if user_operands.len() == 0 {
            inst_map.remove(user);
        }
    }
}
