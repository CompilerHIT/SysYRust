use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        instruction::{Inst, InstKind},
        module::Module,
        tools::{bfs_bb_proceess, func_process},
    },
    utility::ObjPtr,
};

pub fn get_store_map(module: &mut Module) -> HashMap<String, HashSet<ObjPtr<Inst>>> {
    let mut func_map: HashMap<String, HashSet<ObjPtr<Inst>>> = HashMap::new();
    func_process(module, |func_name, func| {
        let mut set = HashSet::new();
        bfs_bb_proceess(func.get_head(), |bb| {
            let mut inst = bb.get_head_inst();
            while !inst.is_tail() {
                let next = inst.get_next();
                match inst.get_kind() {
                    InstKind::Store => {
                        let operands = inst.get_operands();
                        set.insert(operands[0]);
                    }
                    _ => {}
                }
                inst = next;
            }
        });
        func_map.insert(func_name.clone(), set);
    });
    func_map
}
