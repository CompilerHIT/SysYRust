use crate::{ir::{module::Module, tools::{func_process, inst_process_in_bb, dfs_pre_order_bb_process, replace_inst}, basicblock::BasicBlock, instruction::{Inst, InstKind, BinOp, UnOp}, ir_type::IrType}, utility::{ObjPtr, ObjPool}};

use super::global_value_numbering::{self, CongruenceClass, compare_two_inst};

pub fn hoist(module: &mut Module, opt_option: bool,pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
    if opt_option{
        let mut congruence_class = global_value_numbering::gvn(module,opt_option).unwrap();
        func_process(module, |_, func| {
            dfs_pre_order_bb_process(func.get_head(), |bb| {
                let next = bb.get_next_bb().clone();
                if next.len()>1{
                    check_successor(bb,next,&mut congruence_class,pools.1);
                }
            })
        });
    }
}

pub fn check_successor(bb:ObjPtr<BasicBlock>,vec_successors:Vec<ObjPtr<BasicBlock>>,congruence_class:&mut CongruenceClass,pool: &mut ObjPool<Inst>){
    let bb1 = vec_successors[0];
    let bb2 = vec_successors[1];
    inst_process_in_bb(bb1.get_head_inst(), |inst1|{
        inst_process_in_bb(bb2.get_head_inst(), |inst2|{
            if compare_two_inst(inst1, inst2, congruence_class){
                let tail = bb.get_tail_inst();
                let inst_new =make_same_inst(inst1, pool);
                congruence_class.add_inst(inst_new);
                congruence_class.remove_inst(inst1);
                congruence_class.remove_inst(inst2);
                tail.as_mut().insert_before(inst_new);
                replace_inst(inst1, inst_new);
                replace_inst(inst2, inst_new);
            }
        })
    })
}

pub fn make_same_inst(inst_old:ObjPtr<Inst>,pool: &mut ObjPool<Inst>)->ObjPtr<Inst>{
    match inst_old.get_kind() {
        InstKind::Binary(binop) =>{
            let operands = inst_old.get_operands();
            let lhs = operands[0];
            let rhs = operands[1];
            match binop {
                BinOp::Add =>{
                    return pool.make_add(lhs, rhs);
                }
                BinOp::Div =>{
                    return pool.make_div(lhs, rhs);
                }
                BinOp::Eq =>{
                    return pool.make_eq(lhs, rhs);
                }
                BinOp::Ge =>{
                    return pool.make_ge(lhs, rhs);
                }
                BinOp::Gt =>{
                    return pool.make_gt(lhs, rhs);
                }
                BinOp::Le =>{
                    return pool.make_le(lhs, rhs);
                }
                BinOp::Lt =>{
                    return pool.make_lt(lhs, rhs);
                }
                BinOp::Mul =>{
                    return pool.make_mul(lhs, rhs);
                }
                BinOp::Ne =>{
                    return pool.make_ne(lhs, rhs);
                }
                BinOp::Rem =>{
                    return pool.make_rem(lhs, rhs);
                }
                BinOp::Sub =>{
                    return pool.make_sub(lhs, rhs);
                }
            }
        }
        InstKind::Call(_) =>{
            match inst_old.get_ir_type() {
                IrType::Int =>{
                    return pool.make_int_call(inst_old.get_callee().to_string(), inst_old.get_args().clone());
                }
                IrType::Float =>{
                    return pool.make_float_call(inst_old.get_callee().to_string(), inst_old.get_args().clone());
                }
                _=>{}
            }
        }
        InstKind::ConstFloat(f) =>{
            return pool.make_float_const(f);
        }
        InstKind::ConstInt(i) =>{
            return pool.make_int_const(i);
        }
        InstKind::FtoI=>{
            return pool.make_float_to_int(inst_old.get_float_to_int_value());
        }
        InstKind::ItoF =>{
            return pool.make_int_to_float(inst_old.get_int_to_float_value());
        }
        InstKind::Gep =>{
            return pool.make_gep(inst_old.get_gep_ptr(), inst_old.get_gep_offset());
        }
        InstKind::Unary(unop) =>{
            match unop {
                UnOp::Neg =>{
                    return pool.make_neg(inst_old.get_unary_operand());
                }
                UnOp::Not =>{
                    return pool.make_not(inst_old.get_unary_operand());
                }
                UnOp::Pos =>{
                    return pool.make_pos(inst_old.get_unary_operand());
                }
            }
        }
        _=>{}
    }
    todo!("{:?}",inst_old.get_kind())
}