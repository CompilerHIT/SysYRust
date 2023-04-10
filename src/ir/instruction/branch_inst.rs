use super::Instruction;
use crate::ir::{basicblock::BasicBlock, ir_type::IrType, user::User};
use crate::utility::Pointer;
use std::cell::RefMut;
use std::panic;

#[derive(Debug)]
pub struct BranchInst {
    user: User,
    next_bb: Vec<Pointer<BasicBlock>>,
}

impl BranchInst {
    fn make_branch_inst(
        cond: Option<Pointer<Instruction>>,
        next_bb: Vec<Pointer<BasicBlock>>,
    ) -> Pointer<Instruction> {
        match cond {
            Some(r) => {
                let inst = BranchInst {
                    user: User::make_user(IrType::Void, vec![r]),
                    next_bb,
                };
                Pointer::new(Instruction::IBranchInst(inst))
            }
            None => {
                let inst = BranchInst {
                    user: User::make_user(IrType::Void, vec![]),
                    next_bb,
                };
                Pointer::new(Instruction::IBranchInst(inst))
            }
        }
    }

    /// 构造一个条件跳转指令
    pub fn make_cond_br(
        cond: Option<Pointer<Instruction>>,
        next_bb: Vec<Pointer<BasicBlock>>,
    ) -> Pointer<Instruction> {
        Self::make_branch_inst(cond, next_bb)
    }

    /// 构造一个无条件跳转指令
    pub fn make_no_cond_br(next_bb: Pointer<BasicBlock>) -> Pointer<Instruction> {
        Self::make_branch_inst(None, vec![next_bb])
    }

    /// 判断是否为无条件跳转语句
    pub fn is_cond(&self) -> bool {
        self.user.get_operands_size() == 0
    }

    pub fn get_cond(&self) -> RefMut<Instruction> {
        if self.is_cond() {
            self.user.get_operand(0)
        } else {
            panic!("[Error] get_cond(): No condition BrInst has no condition!")
        }
    }
}
