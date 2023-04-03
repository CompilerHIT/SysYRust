use super::{instruction::Instruction, value::Value};
use std::rc::Rc;

pub struct BasicBlock {
    value: Value,
    instruction: Vec<Rc<Instruction>>,
}

impl BasicBlock {
    pub fn make_basicblock(name: String) -> BasicBlock {
        let value = Value::make_value(name, super::ir_type::IrType::BBlock);
        BasicBlock {
            value,
            instruction: Vec::new(),
        }
    }

    pub fn insert(&mut self, inst: Rc<Instruction>, index: usize) {
        self.instruction.insert(index, inst);
    }

    pub fn get_inst(&mut self, index: usize) -> &mut Rc<Instruction> {
        &mut self.instruction[index]
    }
}
