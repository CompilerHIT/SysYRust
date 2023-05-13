use super::*;
use crate::ir::{basicblock::BasicBlock, ir_type::IrType, user::User};
use crate::utility::Pointer;

pub struct BranchInst {
    user: User,
    next_bb: Vec<Pointer<BasicBlock>>,
    list: IList,
}

impl BranchInst {
    fn make_branch_inst(
        cond: Option<Pointer<Box<dyn Instruction>>>,
        next_bb: Vec<Pointer<BasicBlock>>,
    ) -> Pointer<Box<dyn Instruction>> {
        match cond {
            Some(r) => {
                let inst = BranchInst {
                    user: User::make_user(IrType::Void, vec![r]),
                    next_bb,
                    list: IList {
                        prev: None,
                        next: None,
                    },
                };
                Pointer::new(Box::new(inst))
            }
            None => {
                let inst = BranchInst {
                    user: User::make_user(IrType::Void, vec![]),
                    next_bb,
                    list: IList {
                        prev: None,
                        next: None,
                    },
                };
                Pointer::new(Box::new(inst))
            }
        }
    }

    /// 构造一个条件跳转指令
    pub fn make_cond_br(
        cond: Option<Pointer<Box<dyn Instruction>>>,
        ture_bb: Pointer<BasicBlock>,
        false_bb: Pointer<BasicBlock>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_branch_inst(cond, vec![ture_bb, false_bb])
    }

    /// 构造一个无条件跳转指令
    pub fn make_no_cond_br(next_bb: Pointer<BasicBlock>) -> Pointer<Box<dyn Instruction>> {
        Self::make_branch_inst(None, vec![next_bb])
    }

    /// 判断是否为条件跳转语句
    pub fn is_cond(&self) -> bool {
        self.user.get_operands_size() == 0
    }

    /// 判断是否为无条件跳转语句
    pub fn is_nocond(&self) -> bool {
        !self.is_cond()
    }

    /// 获得条件
    pub fn get_cond(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        if self.user.get_operands_size() == 1 {
            Some(self.user.get_operand(0))
        } else {
            None
        }
    }

    /// 获得条件为真时跳转的BasicBlock
    pub fn get_ture_bb(&self) -> Pointer<BasicBlock> {
        if self.is_cond() {
            self.next_bb[0].clone()
        } else {
            panic!("It's not a condition jump!")
        }
    }

    /// 获得条件为假时跳转的BasicBlock
    pub fn get_false_bb(&self) -> Pointer<BasicBlock> {
        if self.is_cond() {
            self.next_bb[1].clone()
        } else {
            panic!("It's not a condition jump!")
        }
    }

    /// 获得无条件跳转的BasicBlock
    pub fn get_nocond_bb(&self) -> Pointer<BasicBlock> {
        if self.is_nocond() {
            self.next_bb[0].clone()
        } else {
            panic!("It 's not a non-condition jump")
        }
    }
}
impl Instruction for BranchInst {
    fn get_inst_type(&self) -> InstructionType {
        InstructionType::IBranchInst
    }
    fn get_value_type(&self) -> IrType {
        self.user.get_ir_type()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn next(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }

    fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.prev()
    }

    fn insert_before(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.insert_before(node);
    }

    fn insert_after(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.insert_after(node)
    }

    fn is_head(&self) -> bool {
        self.list.is_head()
    }

    fn is_tail(&self) -> bool {
        self.list.is_tail()
    }

    fn set_next(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.set_next(node);
    }

    fn set_prev(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(node);
    }

    fn remove_self(&mut self) {
        self.list.remove_self()
    }
}
