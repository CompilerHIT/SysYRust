use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        analysis::store_map::get_store_map,
        instruction::{Inst, InstKind},
        module::Module,
        tools::{bfs_bb_proceess, func_process, replace_inst},
    },
    utility::ObjPtr,
};

pub fn load_store_opt(module: &mut Module) {
    let mut func_map = get_store_map(module);
    func_process(module, |func_name, func| {
        bfs_bb_proceess(func.get_head(), |bb| {
            let mut map = HashMap::new();
            let mut inst = bb.get_head_inst();
            while !inst.is_tail() {
                let next = inst.get_next();
                delete_inst(&mut func_map, &mut map, inst, func_name.clone());
                inst = next;
            }
        });
    });
}

pub fn get_global_array_ptr(inst: ObjPtr<Inst>) -> Option<ObjPtr<Inst>> {
    match inst.get_kind() {
        InstKind::Store => {
            let operands = inst.get_operands();
            match operands[0].get_kind() {
                InstKind::Gep => {
                    let operands2 = operands[0].get_operands();
                    match operands2[0].get_kind() {
                        InstKind::Load => {
                            let inst_vec = operands2[0].get_operands();
                            return Some(inst_vec[0]);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        InstKind::Gep => {
            let operands2 = inst.get_operands();
            match operands2[0].get_kind() {
                InstKind::Load => {
                    let inst_vec = operands2[0].get_operands();
                    return Some(inst_vec[0]);
                }
                _ => {}
            }
        }
        InstKind::Load => {
            let operands = inst.get_operands();
            match operands[0].get_kind() {
                InstKind::Gep => {
                    let operands2 = operands[0].get_operands();
                    match operands2[0].get_kind() {
                        InstKind::Load => {
                            let inst_vec = operands2[0].get_operands();
                            return Some(inst_vec[0]);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    None
}

pub fn delete_inst(
    func_map: &mut HashMap<String, HashSet<ObjPtr<Inst>>>,
    map: &mut HashMap<ObjPtr<Inst>, ObjPtr<Inst>>,
    inst: ObjPtr<Inst>,
    func_now: String,
) -> bool {
    match inst.get_kind() {
        InstKind::Load => {
            let operands = inst.get_operands();
            if let Some(inst_old) = map.get(&operands[0]) {
                let operands_temp = inst_old.get_operands();
                match inst_old.get_kind() {
                    InstKind::Load => {
                        replace_inst(inst, inst_old.clone());
                        return true;
                    }
                    InstKind::Store => {
                        replace_inst(inst, operands_temp[1]);
                        return true;
                    }
                    _ => unreachable!(),
                }
            } else {
                map.insert(operands[0], inst);
            }
        }
        InstKind::Store => {
            let operands = inst.get_operands();
            if let Some(inst_old) = map.get(&operands[0]) {
                match inst_old.get_kind() {
                    InstKind::Load => {
                        map.insert(operands[0], inst);
                        return true;
                    }
                    InstKind::Store => {
                        replace_inst(inst_old.clone(), inst);
                        map.insert(operands[0], inst);
                        return true;
                    }
                    _ => unreachable!(),
                }
            } else {
                map.insert(operands[0], inst);
            }
        }
        InstKind::Call(funcname) => {
            let args = inst.get_args();
            for arg in args {
                match arg.get_kind() {
                    InstKind::Gep => {
                        let oprs = arg.get_operands();
                        let ptr = oprs[0];
                        match ptr.get_kind() {
                            InstKind::Alloca(_) => {
                                //todo:对于局部数组
                                for (tgep, _) in map.clone() {
                                    if tgep.get_kind() == InstKind::Gep {
                                        let operands_temp = tgep.get_operands();
                                        if operands_temp[0] == ptr {
                                            map.remove(&tgep);
                                        }
                                    }
                                }
                            }
                            InstKind::Load => {
                                //todo:对于全局数组
                                let operands_temp = ptr.get_operands();
                                for (pptr, j) in map.clone() {
                                    if let Some(ptr2) = get_global_array_ptr(j) {
                                        if ptr2 == operands_temp[0] {
                                            map.remove(&pptr);
                                        }
                                    }
                                }
                            }
                            InstKind::Parameter => {
                                for (tgep, _) in map.clone() {
                                    if tgep.get_kind() == InstKind::Gep {
                                        let operands_temp = tgep.get_operands();
                                        if operands_temp[0] == ptr {
                                            map.remove(&tgep);
                                        }
                                    }
                                }
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    _ => {}
                }
            }
            if let Some(set) = func_map.get(&funcname) {
                //对于全局变量
                for i in set {
                    if let Some(_) = map.remove(i) {
                        continue;
                    }
                    if let Some(ptr) = get_global_array_ptr(*i) {
                        //对于全局数组
                        for (pptr, j) in map.clone() {
                            if let Some(ptr2) = get_global_array_ptr(j) {
                                if ptr2 == ptr {
                                    map.remove(&pptr);
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    false
}
