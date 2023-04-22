use super::{
    instruction::{head_inst::HeadInst, Instruction},
    ir_type::IrType,
    value::Value,
};
use crate::utility::Pointer;
pub struct BasicBlock {
    value: Value,
    inst_head: Pointer<Box<dyn Instruction>>,
}

impl BasicBlock {
    /// 构造一个空的BisicBlock
    pub fn make_basicblock() -> Pointer<BasicBlock> {
        let value = Value::make_value(IrType::BBlock);
        let bb = BasicBlock {
            value,
            inst_head: Pointer::new(Box::new(HeadInst::new())),
        };
        Pointer::new(bb)
    }

    /// 检查是否为空的BasicBlock
    pub fn is_empty(&self) -> bool {
        self.inst_head.borrow().next().is_none()
    }

    /// 获取BasicBlock的第一条指令
    pub fn get_head_inst(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.inst_head.borrow().next().clone()
    }

    /// 获取BasicBlock的最后一条指令
    pub fn get_tail_inst(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.inst_head.borrow().prev().clone()
    }

    /// 将指令插入到BasicBlock的最后
    pub fn push_back(&mut self, inst: Pointer<Box<dyn Instruction>>) {
        match self.get_tail_inst() {
            Some(tail) => {
                tail.borrow_mut().insert_after(inst.clone());
            }
            None => {
                let mut head = self.inst_head.borrow_mut();
                let mut inst_b = inst.borrow_mut();
                head.set_next(inst.clone());
                head.set_prev(inst.clone());

                inst_b.set_next(self.inst_head.clone());
                inst_b.set_prev(self.inst_head.clone());
            }
        }
    }

    /// 将指令插入到BasicBlock的最前
    pub fn push_front(&mut self, inst: Pointer<Box<dyn Instruction>>) {
        match self.get_head_inst() {
            Some(head) => {
                head.borrow_mut().insert_before(inst.clone());
            }
            None => {
                let mut head = self.inst_head.borrow_mut();
                let mut inst_b = inst.borrow_mut();
                head.set_next(inst.clone());
                head.set_prev(inst.clone());

                inst_b.set_next(self.inst_head.clone());
                inst_b.set_prev(self.inst_head.clone());
            }
        }
    }
}
