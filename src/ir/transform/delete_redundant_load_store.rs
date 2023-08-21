use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        analysis::{store_map::get_store_map, dominator_tree::{self, DominatorTree, calculate_dominator}},
        instruction::{Inst, InstKind},
        module::Module,
        tools::{bfs_bb_proceess, func_process, replace_inst, bfs_inst_process}, basicblock::BasicBlock,
    },
    utility::{ObjPtr, ObjPool},
};

pub fn load_store_opt(module: &mut Module) -> bool {
    let mut func_map = get_store_map(module);
    let mut changed = false;
    func_process(module, |_func_name, func| {
        bfs_bb_proceess(func.get_head(), |bb| {
            let mut map = HashMap::new();
            let mut inst = bb.get_head_inst();
            while !inst.is_tail() {
                let next = inst.get_next();
                changed |= delete_inst(&mut func_map, &mut map, inst);
                inst = next;
            }
        });
    });
    changed
}

// pub fn load_store_opt_plus(module: &mut Module,pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
//     // todo:收集gep信息
//     println!("load_store_opt_plus");
//     func_process(module, |func_name, func| {
//         let dominator_tree = calculate_dominator(func.get_head());
//         let mut flag = true;
//         if !(func_name=="main".to_string()){
//             return;
//         }
//         bfs_inst_process(func.get_head(), |inst|
//         { // 所有gep偏移均为常量
//             if inst.get_kind()==InstKind::Gep{
//                 println!("gep");
//                 let offset = inst.get_gep_offset();
//                 if !offset.is_int_const(){
//                     flag &= false;
//                 }
//             }
//             if inst.get_kind()==InstKind::Store{
//                 flag &= false;
//             }
//         }
//         );
//         if flag{
//             bfs_inst_process(func.get_head(), |inst|
//         {
//             if inst.get_kind()==InstKind::Store{
//                 println!("{:?}",inst);
//             }
//             if inst.get_kind()==InstKind::Store||inst.get_kind()==InstKind::Load{//遍历,存储每一个地址的load,store操作
//                 // println!("{:?}",inst);
//                 let inst_gep = inst.get_operand(0);
//                 if let Some(vec_tmp) = val_map.get_mut(&inst_gep){
//                     vec_tmp.push(inst);
//                 }else{
//                     val_map.insert(inst_gep, vec![inst]);
//                 }
//             }
//         });
//         }
//         // if flag {
//         //     let mut val_map:HashMap<ObjPtr<Inst>, Vec<ObjPtr<Inst>>> = HashMap::new();
//         //     bfs_inst_process(func.get_head(), |inst|
//         // {
//         //     if inst.get_kind()==InstKind::Store{
//         //         println!("{:?}",inst);
//         //     }
//         //     if inst.get_kind()==InstKind::Store||inst.get_kind()==InstKind::Load{//遍历,存储每一个地址的load,store操作
//         //         // println!("{:?}",inst);
//         //         let inst_gep = inst.get_operand(0);
//         //         if let Some(vec_tmp) = val_map.get_mut(&inst_gep){
//         //             vec_tmp.push(inst);
//         //         }else{
//         //             val_map.insert(inst_gep, vec![inst]);
//         //         }
//         //     }
//         // });
//         //     for (_,vec_l_s) in &mut val_map{
//         //         println!("zheli");
//         //         if vec_l_s.len()==2&&vec_l_s[0].get_kind()==InstKind::Store&&vec_l_s[1].get_kind()==InstKind::Load{
//         //             // println!("zheli");
//         //             if dominator_tree.is_dominate(&vec_l_s[0].get_parent_bb(), &vec_l_s[1].get_parent_bb()){
//         //                 println!("删除指令");
//         //                 replace_inst(vec_l_s[1], vec_l_s[0].get_operand(1));//检查store值是不是operand1
//         //                 vec_l_s.remove(1);
//         //             }
//         //         }
//         //     }
//         //     for (_,vec_l_s) in val_map{
//         //         if vec_l_s.len()==1&&vec_l_s[0].get_kind()==InstKind::Store{
//         //             if vec_l_s[0].get_use_list().len()==0{
//         //                 vec_l_s[0].as_mut().remove_self();
//         //             }
//         //         }
//         //     }


//         // }
//     });
// }

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
                        return false;
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
                                // println!("该参数为局部数组");
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
                                for (tgep, _) in map.clone() {
                                    if tgep.get_kind() == InstKind::Gep {
                                        let operands_temp = tgep.get_operands();
                                        if operands_temp[0] == ptr {
                                            map.remove(&tgep);
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
