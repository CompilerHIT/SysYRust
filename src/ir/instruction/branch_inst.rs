use super::Instruction;
use crate::ir::{basicblock::BasicBlock, ir_type::IrType, user::User};
use std::cell::{RefCell, RefMut};
use std::panic;
use std::rc::Rc;

pub struct BranchInst {
    user: User,
    next_bb: Vec<Rc<RefCell<BasicBlock>>>,
}

impl BranchInst {
    fn make_branch_inst(
        name: String,
        cond: Option<Rc<RefCell<Instruction>>>,
        next_bb: Vec<Rc<RefCell<BasicBlock>>>,
    ) -> Rc<RefCell<Instruction>> {
        match cond {
            Some(r) => {
                let inst = BranchInst {
                    user: User::make_user(name, IrType::Void, vec![r]),
                    next_bb,
                };
                Rc::new(RefCell::new(Instruction::EBranchInst(inst)))
            }
            None => {
                let inst = BranchInst {
                    user: User::make_user(name, IrType::Void, vec![]),
                    next_bb,
                };
                Rc::new(RefCell::new(Instruction::EBranchInst(inst)))
            }
        }
    }

    /// 构造一个条件跳转指令
    pub fn make_cond_br(
        name: String,
        cond: Option<Rc<RefCell<Instruction>>>,
        next_bb: Vec<Rc<RefCell<BasicBlock>>>,
    ) -> Rc<RefCell<Instruction>> {
        Self::make_branch_inst(name, cond, next_bb)
    }

    /// 构造一个无条件跳转指令
    pub fn make_no_cond_br(
        name: String,
        next_bb: Rc<RefCell<BasicBlock>>,
    ) -> Rc<RefCell<Instruction>> {
        Self::make_branch_inst(name, None, vec![next_bb])
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
