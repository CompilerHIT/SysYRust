use super::Instruction;
use crate::ir::{basicblock::BasicBlock, ir_type::IrType, user::User};
use std::rc::Rc;

struct BranchInst {
    user: User,
    cond: Option<Rc<Instruction>>,
    next_bb: Vec<Rc<BasicBlock>>,
}

impl BranchInst {
    fn make_branch_inst(
        name: String,
        cond: Option<Rc<Instruction>>,
        next_bb: Vec<Rc<BasicBlock>>,
    ) -> Rc<BranchInst> {
        Rc::new(BranchInst {
            user: User::make_user(name, IrType::Void),
            cond,
            next_bb,
        })
    }

    pub fn make_cond_br(
        name: String,
        cond: Option<Rc<Instruction>>,
        next_bb: Vec<Rc<BasicBlock>>,
    ) -> Rc<BranchInst> {
        Self::make_branch_inst(name, cond, next_bb)
    }

    pub fn make_no_cond_br(name: String, next_bb: Rc<BasicBlock>) -> Rc<BranchInst> {
        Self::make_branch_inst(name, None, vec![next_bb])
    }
}
