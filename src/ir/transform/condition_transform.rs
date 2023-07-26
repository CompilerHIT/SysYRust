use crate::{utility::{ObjPtr, ObjPool}, ir::{instruction::{Inst,InstKind, BinOp}, ir_type::IrType}};

//todo:对只有一个int表达式的cond做处理
pub fn check_br(inst:ObjPtr<Inst>,pool:&mut ObjPool<Inst>)->Option<(ObjPtr<Inst>,BinOp,i32)>{
    let cond = inst.get_operands();
    if !cond[0].get_ir_type().is_int(){
        return None;
    }
    match cond[0].get_kind() {
        InstKind::Binary(binop) =>{
            match binop {
                BinOp::Eq|BinOp::Ge|BinOp::Gt|BinOp::Le|BinOp::Lt|BinOp::Ne =>{}
                _=>{
                    let inst_const = pool.make_int_const(0);
                    let inst_new = pool.make_ne(cond[0], inst_const);
                    inst.as_mut().insert_after(inst_const);
                    inst.as_mut().insert_after(inst_new);
                    inst.as_mut().set_operand(inst_new, 0);
                }
            }
        }
        _=>{
            let inst_const = pool.make_int_const(0);
            let inst_new = pool.make_ne(cond[0], inst_const);
            inst.as_mut().insert_after(inst_const);
            inst.as_mut().insert_after(inst_new);
            inst.as_mut().set_operand(inst_new, 0);
        }
    }
    let cond = inst.get_operands();
    if let Some((binop,i)) = check_cond(cond[0]){
        return Some((inst,binop,i));
    }
    None
}

pub fn check_cond(inst:ObjPtr<Inst>)->Option<(BinOp,i32)>{
    if !inst.get_ir_type().is_int(){
        return None;
    }
    match inst.get_kind(){
        InstKind::Binary(binop) =>{
            match binop {
                BinOp::Eq =>{
                    if let Some(i) = check_op(inst){
                        return Some((binop,i));
                    }
                }
                BinOp::Ne =>{
                    if let Some(i) = check_op(inst){
                        return Some((binop,i));
                    }
                }
                _=>{}
            }
        }
        _=>{}
    }
    return None;
}

pub fn check_op(inst:ObjPtr<Inst>) ->Option<i32>{
    let operands = inst.get_operands();
    match operands[0].get_kind() {
        InstKind::ConstInt(i) =>{
            Some(i)
        }
        _ =>{
            match operands[1].get_kind() {
                InstKind::ConstInt(i)  =>{
                    Some(i)
                }
                _ =>{
                    None
                }
            }
        }
    }
}