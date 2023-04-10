use super::{instruction::Instruction, ir_type::IrType, value::Value};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct BasicBlock {
    value: Value,
    instruction: Vec<Rc<RefCell<Instruction>>>,
}

impl BasicBlock {
    /// 构造一个空的BisicBlock
    pub fn make_basicblock() -> BasicBlock {
        let value = Value::make_value(IrType::BBlock);
        BasicBlock {
            value,
            instruction: Vec::new(),
        }
    }

    /// 在index处插入一条指令
    pub fn insert(&mut self, inst: Rc<RefCell<Instruction>>, index: usize) {
        self.instruction.insert(index, inst);
    }
}
