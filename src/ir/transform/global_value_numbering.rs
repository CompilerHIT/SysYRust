use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        analysis::{
            call_optimize::call_optimize,
            dominator_tree::{calculate_dominator, DominatorTree},
        },
        instruction::{BinOp, Inst, InstKind},
        ir_type::IrType,
        module::Module,
        tools::{bfs_inst_process, func_process, replace_inst},
    },
    utility::ObjPtr,
};

use super::delete_redundant_load_store::load_store_opt;

pub struct Congruence {
    pub vec_class: Vec<Vec<ObjPtr<Inst>>>,
    pub map: HashMap<ObjPtr<Inst>, usize>,
}

pub fn easy_gvn(module: &mut Module) -> bool {
    let mut congruence = Congruence {
        vec_class: vec![],
        map: HashMap::new(),
    };
    let mut changed = false;
    let set = call_optimize(module);
    func_process(module, |_, func| {
        let dominator_tree = calculate_dominator(func.get_head());
            bfs_inst_process(func.get_head(), |inst| {
                changed |= has_val(&mut congruence, inst, &dominator_tree, set.clone())
            });
    });
    changed
}

pub fn gvn(module: &mut Module, opt_option: bool) {
    if opt_option {
        loop {
            let mut changed = false;
            changed |= easy_gvn(module);
            changed |= load_store_opt(module);
            if !changed {
                break;
            }
        }
    }
}

pub fn has_val(
    congrunce: &mut Congruence,
    inst: ObjPtr<Inst>,
    dominator_tree: &DominatorTree,
    set: HashSet<String>,
) -> bool {
    match inst.get_kind() {
        InstKind::Alloca(_)
        | InstKind::Branch
        | InstKind::Head
        | InstKind::Parameter
        | InstKind::Return
        | InstKind::Store
        | InstKind::Load
        | InstKind::GlobalConstFloat(_)
        | InstKind::GlobalConstInt(_)
        | InstKind::GlobalFloat(_)
        | InstKind::GlobalInt(_)
        | InstKind::Phi => {} //todo:phi可以被优化吗
        InstKind::Call(funcname) => {
            if set.contains(&funcname) {
                //纯函数，可复用
                if let Some(_index) = congrunce.map.get(&inst) {
                    return false;
                }
                for vec_congruent in congrunce.vec_class.clone() {
                    if compare_two_inst(inst, vec_congruent[0], &congrunce) {
                        if dominator_tree
                            .is_dominate(&vec_congruent[0].get_parent_bb(), &inst.get_parent_bb())
                        {
                            replace_inst(inst, vec_congruent[0]);
                            return true;
                        } else {
                            for i in 1..vec_congruent.len() {
                                if dominator_tree.is_dominate(
                                    &vec_congruent[i].get_parent_bb(),
                                    &inst.get_parent_bb(),
                                ) {
                                    replace_inst(inst, vec_congruent[i]);
                                    return true;
                                }
                            }
                        }
                        //都没有可以替代这条指令的congruent inst,将这条指令加入congruent inst中
                        if let Some(index) = congrunce.map.get(&vec_congruent[0]) {
                            congrunce.vec_class[*index].push(inst);
                            congrunce.map.insert(inst, *index);
                        }
                        return false;
                    }
                }
            }
        }
        _ => {
            if let Some(_index) = congrunce.map.get(&inst) {
                return false;
            }
            for vec_congruent in congrunce.vec_class.clone() {
                if compare_two_inst(inst, vec_congruent[0], &congrunce) {
                    if dominator_tree
                        .is_dominate(&vec_congruent[0].get_parent_bb(), &inst.get_parent_bb())
                    {
                        replace_inst(inst, vec_congruent[0]);
                        return true;
                    } else {
                        for i in 1..vec_congruent.len() {
                            if dominator_tree.is_dominate(
                                &vec_congruent[i].get_parent_bb(),
                                &inst.get_parent_bb(),
                            ) {
                                replace_inst(inst, vec_congruent[i]);
                                return true;
                            }
                        }
                    }
                    //都没有可以替代这条指令的congruent inst,将这条指令加入congruent inst中
                    if let Some(index) = congrunce.map.get(&vec_congruent[0]) {
                        congrunce.vec_class[*index].push(inst);
                        congrunce.map.insert(inst, *index);
                    }
                    return false;
                }
            }
        }
    }
    let index = congrunce.vec_class.len();
    congrunce.vec_class.push(vec![inst]); //加入新的congruent class
    congrunce.map.insert(inst, index); //增加索引映射
    false
}

pub fn compare_two_inst(inst1: ObjPtr<Inst>, inst2: ObjPtr<Inst>, congrunce: &Congruence) -> bool {
    let tpflag = inst1.get_ir_type() == inst2.get_ir_type();
    if inst1.get_kind() == inst2.get_kind() && tpflag {
        match inst1.get_kind() {
            InstKind::Gep => {
                let operands1 = inst1.get_operands();
                let operands2 = inst2.get_operands();
                return compare_two_inst_with_index(operands1[0], operands2[0], congrunce)
                    && compare_two_inst_with_index(operands1[1], operands2[1], congrunce);
            }
            InstKind::Unary(unop1) => match inst2.get_kind() {
                InstKind::Unary(unop2) => {
                    let operands1 = inst1.get_operands();
                    let operands2 = inst2.get_operands();
                    return unop1 == unop2
                        && compare_two_inst_with_index(operands1[0], operands2[0], congrunce);
                }
                _ => unreachable!(),
            },
            InstKind::ConstInt(i1) => match inst2.get_kind() {
                InstKind::ConstInt(i2) => {
                    if i1 == i2 {
                        return true;
                    } else {
                        return false;
                    }
                }
                _ => {
                    unreachable!()
                }
            },
            InstKind::ConstFloat(f1) => match inst2.get_kind() {
                InstKind::ConstFloat(f2) => {
                    if f1 == f2 {
                        return true;
                    } else {
                        return false;
                    }
                }
                _ => {
                    unreachable!()
                }
            },
            InstKind::FtoI => {
                let operands1 = inst1.get_operands();
                let operands2 = inst2.get_operands();
                return compare_two_inst_with_index(operands1[0], operands2[0], congrunce);
            }
            InstKind::ItoF => {
                let operands1 = inst1.get_operands();
                let operands2 = inst2.get_operands();
                return compare_two_inst_with_index(operands1[0], operands2[0], congrunce);
            }
            InstKind::Binary(binop1) => match inst2.get_kind() {
                InstKind::Binary(binop2) => {
                    if binop1 == binop2 && inst1.get_ir_type() == IrType::Int {
                        match binop1 {
                            BinOp::Add | BinOp::Eq | BinOp::Mul | BinOp::Ne => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                return compare_two_operands(operands1, operands2, congrunce);
                            }
                            _ => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                if compare_two_inst_with_index(
                                    operands1[0],
                                    operands2[0],
                                    congrunce,
                                ) && compare_two_inst_with_index(
                                    operands1[1],
                                    operands2[1],
                                    congrunce,
                                ) {
                                    return true;
                                } else {
                                    return false;
                                }
                            }
                        }
                    } else if binop1 == binop2 && inst1.get_ir_type() == IrType::Float {
                        match binop1 {
                            BinOp::Eq | BinOp::Ne => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                return compare_two_operands(operands1, operands2, congrunce);
                            }
                            _ => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                if compare_two_inst_with_index(
                                    operands1[0],
                                    operands2[0],
                                    congrunce,
                                ) && compare_two_inst_with_index(
                                    operands1[1],
                                    operands2[1],
                                    congrunce,
                                ) {
                                    return true;
                                } else {
                                    return false;
                                }
                            }
                        }
                    } //todo:比较指令
                }
                _ => unreachable!(),
            },
            _ => {}
        }
    } else if tpflag {
        match inst1.get_kind() {
            InstKind::Binary(binop1) => match inst2.get_kind() {
                InstKind::Binary(binop2) => {
                    let operands1 = inst1.get_operands();
                    let operands2 = inst2.get_operands();
                    if (binop1 == BinOp::Ge && binop2 == BinOp::Lt)
                        || (binop1 == BinOp::Gt && binop2 == BinOp::Le)
                        || (binop1 == BinOp::Le && binop2 == BinOp::Gt)
                        || (binop1 == BinOp::Lt && binop2 == BinOp::Ge)
                    {
                        return compare_two_inst_with_index(operands1[0], operands2[1], congrunce)
                            && compare_two_inst_with_index(operands1[1], operands2[0], congrunce);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    false
}

pub fn compare_two_inst_with_index(
    inst1: ObjPtr<Inst>,
    inst2: ObjPtr<Inst>,
    congrunce: &Congruence,
) -> bool {
    if inst1 == inst2 {
        //针对全局指针
        return true;
    }
    if let Some(index1) = congrunce.map.get(&inst1) {
        if let Some(index2) = congrunce.map.get(&inst2) {
            if index1 == index2 {
                return true;
            }
        }
    }
    false
}

pub fn compare_two_operands(
    operands1: &Vec<ObjPtr<Inst>>,
    operands2: &Vec<ObjPtr<Inst>>,
    congrunce: &Congruence,
) -> bool {
    if compare_two_inst_with_index(operands1[0], operands2[0], congrunce)
        && compare_two_inst_with_index(operands1[1], operands2[1], congrunce)
    {
        return true;
    } else if compare_two_inst_with_index(operands1[1], operands2[0], congrunce)
        && compare_two_inst_with_index(operands1[0], operands2[1], congrunce)
    {
        return true;
    }
    false
}
