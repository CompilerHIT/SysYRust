use crate::{
    ir::basicblock::BasicBlock,
    ir::instruction::Inst,
    ir::instruction::InstKind,
    ir::module::Module,
    ir::{instruction::BinOp, ir_type::*},
    ir::{
        instruction::UnOp,
        tools::{bfs_inst_process, func_process},
    },
    utility::{ObjPool, ObjPtr},
};

pub fn constant_folding(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    func_process(module, |_, func| loop {
        let mut changed = false;
        bfs_inst_process(func.get_head(), |inst| changed |= fold_inst(inst, pools.1));
        if !changed {
            break;
        }
    })
}

pub fn fold_inst(inst_old: ObjPtr<Inst>, pool: &mut ObjPool<Inst>) -> bool {
    //判断指令类型
    match inst_old.get_kind() {
        //根据指令类型进行常量折叠
        InstKind::Binary(binop) => {
            let operands = inst_old.get_operands();
            match operands[0].get_ir_type() {
                IrType::Float => {
                    if let Some(val_left) = get_fconstant(operands[0]) {
                        if let Some(val_right) = get_fconstant(operands[1]) {
                            match binop {
                                BinOp::Add => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_float_const(val_left + val_right),
                                    );
                                }
                                BinOp::Sub => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_float_const(val_left - val_right),
                                    );
                                }
                                BinOp::Mul => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_float_const(val_left * val_right),
                                    );
                                }
                                BinOp::Div => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_float_const(val_left / val_right),
                                    );
                                }
                                BinOp::Rem => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_float_const(val_left % val_right),
                                    );
                                }
                                BinOp::Eq => {
                                    if val_left == val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ne => {
                                    if val_left != val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Lt => {
                                    if val_left < val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ge => {
                                    if val_left >= val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Gt => {
                                    if val_left > val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Le => {
                                    if val_left <= val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
                IrType::Int => {
                    if let Some(val_left) = get_iconstant(operands[0]) {
                        if let Some(val_right) = get_iconstant(operands[1]) {
                            match binop {
                                BinOp::Add => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_int_const(val_left + val_right),
                                    );
                                }
                                BinOp::Sub => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_int_const(val_left - val_right),
                                    );
                                }
                                BinOp::Mul => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_int_const(val_left * val_right),
                                    );
                                }
                                BinOp::Div => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_int_const(val_left / val_right),
                                    );
                                }
                                BinOp::Rem => {
                                    replace_inst(
                                        inst_old,
                                        pool.make_int_const(val_left % val_right),
                                    );
                                }
                                BinOp::Eq => {
                                    if val_left == val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ne => {
                                    if val_left != val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Lt => {
                                    if val_left < val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ge => {
                                    if val_left >= val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Gt => {
                                    if val_left > val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Le => {
                                    if val_left <= val_right {
                                        replace_inst(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst(inst_old, pool.make_int_const(0));
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }
        InstKind::Unary(unop) => {
            let operands = inst_old.get_operands();
            match operands[0].get_ir_type() {
                IrType::Float => {
                    if let Some(val) = get_fconstant(operands[0]) {
                        match unop {
                            UnOp::Neg => {
                                replace_inst(inst_old, pool.make_float_const(-val));
                            }
                            UnOp::Not => {
                                if val == 0.0 {
                                    replace_inst(inst_old, pool.make_float_const(1.0));
                                } else {
                                    replace_inst(inst_old, pool.make_float_const(0.0));
                                }
                            }
                            UnOp::Pos => {
                                replace_inst(inst_old, pool.make_float_const(val));
                            }
                        }
                        return true;
                    }
                }
                IrType::Int => {
                    if let Some(val) = get_iconstant(operands[0]) {
                        match unop {
                            UnOp::Neg => {
                                replace_inst(inst_old, pool.make_int_const(-val));
                            }
                            UnOp::Not => {
                                if val == 0 {
                                    replace_inst(inst_old, pool.make_int_const(1));
                                } else {
                                    replace_inst(inst_old, pool.make_int_const(0));
                                }
                            }
                            UnOp::Pos => {
                                replace_inst(inst_old, pool.make_int_const(val));
                            }
                        }
                        return true;
                    }
                }
                _ => {
                    unreachable!()
                }
            }
        }
        InstKind::FtoI => {
            let operands = inst_old.get_operands();
            if let Some(val) = get_fconstant(operands[0]) {
                replace_inst(inst_old, pool.make_int_const(val as i32));
                return true;
            }
        }
        InstKind::ItoF => {
            let operands = inst_old.get_operands();
            if let Some(val) = get_iconstant(operands[0]) {
                replace_inst(inst_old, pool.make_float_const(val as f32));
                return true;
            }
        }
        _ => {}
    }
    false
    //判断是否能够被替换
    //能替换则用一条const指令替换该指令，更改使用这条指令的所有指令的操作数指向,删除该指令
}

pub fn replace_inst(inst_old: ObjPtr<Inst>, inst_new: ObjPtr<Inst>) {
    let use_list = inst_old.get_use_list().clone();
    inst_old.as_mut().insert_before(inst_new); //插入新指令
    for user in use_list {
        //将使用过旧指令的指令指向新指令
        let index = user.get_operand_index(inst_old);
        user.as_mut().set_operand(inst_new, index);
    }
    inst_old.as_mut().remove_self(); //丢掉旧指令
}

pub fn get_iconstant(inst: ObjPtr<Inst>) -> Option<i32> {
    match inst.get_kind() {
        InstKind::ConstInt(i) | InstKind::GlobalConstInt(i) | InstKind::GlobalInt(i) => Some(i),
        _ => None,
    }
}

pub fn get_fconstant(inst: ObjPtr<Inst>) -> Option<f32> {
    match inst.get_kind() {
        InstKind::ConstFloat(f) | InstKind::GlobalConstFloat(f) | InstKind::GlobalFloat(f) => {
            Some(f)
        }
        _ => None,
    }
}
