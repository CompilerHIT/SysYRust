#![allow(arithmetic_overflow)]
use crate::{
    frontend::context::Type,
    ir::basicblock::BasicBlock,
    ir::instruction::Inst,
    ir::instruction::InstKind,
    ir::{instruction::BinOp, ir_type::*},
    ir::{
        instruction::UnOp,
        tools::{bfs_inst_process, func_process},
    },
    ir::{module::Module, tools::replace_inst},
    utility::{ObjPool, ObjPtr},
};

pub fn constant_folding(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    flag: bool,
) {
    func_process(module, |_, func| loop {
        let mut changed = false;
        bfs_inst_process(func.get_head(), |inst| changed |= fold_inst(inst, pools.1));
        if !changed {
            break;
        }
    });
    if flag {
        func_process(module, |_, func| loop {
            let mut changed = false;
            bfs_inst_process(func.get_head(), |inst| {
                changed |= fold_mixed_binst(inst, pools.1)
            });
            if !changed {
                break;
            }
        });
        func_process(module, |_, func| {
            bfs_inst_process(func.get_head(), |inst| {
                convert_add_inst(inst, pools.1);
            })
        })
    }
}

pub fn check_mul_inst(
    inst_old: ObjPtr<Inst>,
    inst1: ObjPtr<Inst>,
    inst2: ObjPtr<Inst>,
    pool: &mut ObjPool<Inst>,
) -> bool {
    match inst1.get_kind() {
        InstKind::Binary(binop) => match binop {
            BinOp::Mul => {
                let operands1 = inst1.get_operands();
                if operands1[0] == inst2 {
                    match operands1[1].get_kind() {
                        InstKind::ConstInt(i) => {
                            let inst_const = pool.make_int_const(i  + 1);
                            inst_old.as_mut().insert_before(inst_const);
                            let inst_new = pool.make_mul(inst2, inst_const);
                            inst_old.as_mut().insert_before(inst_new);
                            replace_inst(inst_old, inst_new);
                            return true;
                        }
                        _ => {}
                    }
                } else if operands1[1] == inst2 {
                    match operands1[0].get_kind() {
                        InstKind::ConstInt(i) => {
                            let inst_const = pool.make_int_const(i + 1);
                            inst_old.as_mut().insert_before(inst_const);
                            let inst_new = pool.make_mul(inst2, inst_const);
                            inst_old.as_mut().insert_before(inst_new);
                            replace_inst(inst_old, inst_new);
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        _ => {}
    }
    false
}

pub fn convert_add_inst(inst: ObjPtr<Inst>, pool: &mut ObjPool<Inst>) {
    match inst.get_kind() {
        InstKind::Binary(binop) => {
            match binop {
                BinOp::Add => {
                    let operands = inst.get_operands();
                    if operands[0] == operands[1] && inst.get_ir_type() == IrType::Int {
                        //同一操作数的整数加法改为乘法指令
                        let inst_const = pool.make_int_const(2);
                        inst.as_mut().insert_before(inst_const);
                        let inst_new = pool.make_mul(inst_const, operands[0]);
                        inst.as_mut().insert_before(inst_new);
                        replace_inst(inst, inst_new);
                    } else {
                        if !check_mul_inst(inst, operands[0], operands[1], pool) {
                            check_mul_inst(inst, operands[1], operands[0], pool);
                        }
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// 折叠操作数之一为常量的指令流
pub fn fold_mixed_binst(inst_old: ObjPtr<Inst>, pool: &mut ObjPool<Inst>) -> bool {
    let operands = inst_old.get_operands();
    match inst_old.get_kind() {
        InstKind::Binary(_) => {
            if let Some((num_operand, num_const1, _)) = check_mixed_binst(inst_old) {
                match operands[num_operand].get_kind() {
                    InstKind::Binary(_) => {
                        if let Some((num_unknown, num_const2, tp)) =
                            check_mixed_binst(operands[num_operand])
                        {
                            fold_two_mixed_binst(
                                inst_old,
                                num_operand,
                                num_const1,
                                num_unknown,
                                num_const2,
                                tp,
                                pool,
                            )
                        } else {
                            false
                        }
                    }
                    _ => false,
                }
            } else {
                false
            }
        }
        _ => false,
    }
    // todo!()
}

pub fn check_mixed_binst(inst: ObjPtr<Inst>) -> Option<(usize, usize, Type)> {
    //第一个返回值是非常量inst
    let operands = inst.get_operands();
    match operands[0].get_kind() {
        InstKind::ConstFloat(_) | InstKind::GlobalConstFloat(_) | InstKind::GlobalFloat(_) => {
            match operands[1].get_kind() {
                InstKind::ConstFloat(_)
                | InstKind::GlobalConstFloat(_)
                | InstKind::GlobalFloat(_)
                | InstKind::ConstInt(_)
                | InstKind::GlobalConstInt(_)
                | InstKind::GlobalInt(_) => {
                    unreachable!("基础常量折叠不完全")
                }
                _ => Some((1, 0, Type::Float)),
            }
        }
        InstKind::ConstInt(_) | InstKind::GlobalConstInt(_) | InstKind::GlobalInt(_) => {
            match operands[1].get_kind() {
                InstKind::ConstFloat(_)
                | InstKind::GlobalConstFloat(_)
                | InstKind::GlobalFloat(_)
                | InstKind::ConstInt(_)
                | InstKind::GlobalConstInt(_)
                | InstKind::GlobalInt(_) => {
                    unreachable!("基础常量折叠不完全")
                }
                _ => Some((1, 0, Type::Int)),
            }
        }
        _ => match operands[1].get_kind() {
            InstKind::ConstFloat(_) | InstKind::GlobalConstFloat(_) | InstKind::GlobalFloat(_) => {
                Some((0, 1, Type::Float))
            }
            InstKind::ConstInt(_) | InstKind::GlobalConstInt(_) | InstKind::GlobalInt(_) => {
                Some((0, 1, Type::Int))
            }
            _ => None,
        },
    }
}
pub fn fold_two_mixed_binst(
    inst: ObjPtr<Inst>,
    num_operand: usize,
    num_const1: usize,
    num_unknown: usize,
    num_const2: usize,
    tp: Type,
    pool: &mut ObjPool<Inst>,
) -> bool {
    // 处理累加累乘指令
    let operands1 = inst.get_operands();
    let inst_operand = operands1[num_operand];
    let const1 = operands1[num_const1];
    let operands2 = inst_operand.get_operands();
    let inst_unknown = operands2[num_unknown];
    let const2 = operands2[num_const2];

    let mut iflag = true;
    match tp {
        Type::Float => {
            iflag = false;
        }
        _ => {}
    }
    match inst.get_kind() {
        InstKind::Binary(binop) => match binop {
            BinOp::Add => match inst_operand.get_kind() {
                InstKind::Binary(binop2) => match binop2 {
                    BinOp::Add => {
                        if iflag {
                            let inst_result =
                                pool.make_int_const(const1.get_int_bond() + const2.get_int_bond());
                            let inst_new = pool.make_add(inst_result, inst_unknown);
                            replace_inst_with_new(inst, inst_new);
                            inst_new.as_mut().insert_before(inst_result);
                        }
                        return true;
                    }
                    BinOp::Sub => {
                        if iflag {
                            if num_unknown == 0 {
                                let inst_result = pool
                                    .make_int_const(const1.get_int_bond() - const2.get_int_bond() );
                                let inst_new = pool.make_add(inst_result, inst_unknown);
                                replace_inst_with_new(inst, inst_new);
                                inst_new.as_mut().insert_before(inst_result);
                            } else {
                                let inst_result = pool
                                    .make_int_const(const1.get_int_bond() + const2.get_int_bond() );
                                let inst_new = pool.make_sub(inst_result, inst_unknown);
                                replace_inst_with_new(inst, inst_new);
                                inst_new.as_mut().insert_before(inst_result);
                            }
                        }
                        return true;
                    }
                    _ => {
                        return false;
                    }
                },
                _ => {
                    unreachable!()
                }
            },
            BinOp::Sub => match inst_operand.get_kind() {
                InstKind::Binary(binop2) => match binop2 {
                    BinOp::Sub => {
                        if iflag {
                            if num_unknown == 1 && num_operand == 1 {
                                let inst_result = pool
                                    .make_int_const(const1.get_int_bond() - const2.get_int_bond() );
                                let inst_new = pool.make_add(inst_result, inst_unknown);
                                replace_inst_with_new(inst, inst_new);
                                inst_new.as_mut().insert_before(inst_result);
                            } else if num_unknown == 1 && num_operand == 0 {
                                let inst_result = pool
                                    .make_int_const(-const1.get_int_bond() + const2.get_int_bond() );
                                let inst_new = pool.make_sub(inst_result, inst_unknown);
                                replace_inst_with_new(inst, inst_new);
                                inst_new.as_mut().insert_before(inst_result);
                            } else if num_unknown == 0 && num_operand == 0 {
                                let inst_result = pool
                                    .make_int_const(const1.get_int_bond() + const2.get_int_bond());
                                let inst_new = pool.make_sub(inst_unknown, inst_result);
                                replace_inst_with_new(inst, inst_new);
                                inst_new.as_mut().insert_before(inst_result);
                            } else if num_unknown == 0 && num_operand == 1 {
                                let inst_result = pool
                                    .make_int_const(const1.get_int_bond() + const2.get_int_bond());
                                let inst_new = pool.make_sub(inst_result, inst_unknown);
                                replace_inst_with_new(inst, inst_new);
                            }
                        }
                        return true;
                    }
                    BinOp::Add => {
                        if iflag {
                            if num_operand == 0 {
                                let inst_result = pool
                                    .make_int_const(const2.get_int_bond() - const1.get_int_bond() );
                                let inst_new = pool.make_add(inst_result, inst_unknown);
                                replace_inst_with_new(inst, inst_new);
                                inst_new.as_mut().insert_before(inst_result);
                            } else {
                                let inst_result = pool
                                    .make_int_const(const1.get_int_bond() - const2.get_int_bond());
                                let inst_new = pool.make_sub(inst_result, inst_unknown);
                                replace_inst_with_new(inst, inst_new);
                                inst_new.as_mut().insert_before(inst_result);
                            }
                        }
                        return true;
                    }
                    _ => {
                        return false;
                    }
                },
                _ => {
                    unreachable!()
                }
            },
            BinOp::Mul => match inst_operand.get_kind() {
                InstKind::Binary(binop2) => match binop2 {
                    BinOp::Mul => {
                        if iflag {
                            let inst_result =
                                pool.make_int_const(const1.get_int_bond() * const2.get_int_bond() );
                            let inst_new = pool.make_mul(inst_result, inst_unknown);
                            replace_inst_with_new(inst, inst_new);
                            inst_new.as_mut().insert_before(inst_result);
                        }
                        return true;
                    }
                    _ => {
                        return false;
                    }
                },
                _ => {
                    unreachable!()
                }
            },
            _ => {
                return false;
            }
        },
        _ => {
            unreachable!()
        }
    }
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
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_float_const(val_left + val_right),
                                    );
                                }
                                BinOp::Sub => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_float_const(val_left - val_right),
                                    );
                                }
                                BinOp::Mul => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_float_const(val_left * val_right),
                                    );
                                }
                                BinOp::Div => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_float_const(val_left / val_right),
                                    );
                                }
                                BinOp::Rem => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_float_const(val_left % val_right),
                                    );
                                }
                                BinOp::Eq => {
                                    if val_left == val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ne => {
                                    if val_left != val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Lt => {
                                    if val_left < val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ge => {
                                    if val_left >= val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Gt => {
                                    if val_left > val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Le => {
                                    if val_left <= val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
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
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_int_const(val_left + val_right ),
                                    );
                                }
                                BinOp::Sub => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_int_const(val_left  - val_right),
                                    );
                                }
                                BinOp::Mul => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_int_const(val_left  * val_right),
                                    );
                                }
                                BinOp::Div => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_int_const(val_left / val_right ),
                                    );
                                }
                                BinOp::Rem => {
                                    replace_inst_with_new(
                                        inst_old,
                                        pool.make_int_const(val_left % val_right),
                                    );
                                }
                                BinOp::Eq => {
                                    if val_left == val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ne => {
                                    if val_left != val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Lt => {
                                    if val_left < val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Ge => {
                                    if val_left >= val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Gt => {
                                    if val_left > val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
                                    }
                                }
                                BinOp::Le => {
                                    if val_left <= val_right {
                                        replace_inst_with_new(inst_old, pool.make_int_const(1));
                                    } else {
                                        replace_inst_with_new(inst_old, pool.make_int_const(0));
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
                                replace_inst_with_new(inst_old, pool.make_float_const(-val));
                            }
                            UnOp::Not => {
                                if val == 0.0 {
                                    replace_inst_with_new(inst_old, pool.make_float_const(1.0));
                                } else {
                                    replace_inst_with_new(inst_old, pool.make_float_const(0.0));
                                }
                            }
                            UnOp::Pos => {
                                replace_inst_with_new(inst_old, pool.make_float_const(val));
                            }
                        }
                        return true;
                    }
                }
                IrType::Int => {
                    if let Some(val) = get_iconstant(operands[0]) {
                        match unop {
                            UnOp::Neg => {
                                replace_inst_with_new(inst_old, pool.make_int_const(-val));
                            }
                            UnOp::Not => {
                                if val == 0 {
                                    replace_inst_with_new(inst_old, pool.make_int_const(1));
                                } else {
                                    replace_inst_with_new(inst_old, pool.make_int_const(0));
                                }
                            }
                            UnOp::Pos => {
                                replace_inst_with_new(inst_old, pool.make_int_const(val));
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
                replace_inst_with_new(inst_old, pool.make_int_const(val as i32));
                return true;
            }
        }
        InstKind::ItoF => {
            let operands = inst_old.get_operands();
            if let Some(val) = get_iconstant(operands[0]) {
                replace_inst_with_new(inst_old, pool.make_float_const(val as f32));
                return true;
            }
        }
        _ => {}
    }
    false
    //判断是否能够被替换
    //能替换则用一条const指令替换该指令，更改使用这条指令的所有指令的操作数指向,删除该指令
}

pub fn replace_inst_with_new(inst_old: ObjPtr<Inst>, inst_new: ObjPtr<Inst>) {
    inst_old.as_mut().insert_before(inst_new); //插入新指令
    replace_inst(inst_old, inst_new);
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
