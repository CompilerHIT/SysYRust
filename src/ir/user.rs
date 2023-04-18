use super::{instruction::Instruction, ir_type::IrType, value::Value};
use crate::utility::Pointer;
use std::cell::RefMut;

pub struct User {
    value: Value,
    operands: Vec<Pointer<Box<dyn Instruction>>>,
}

impl User {
    pub fn make_user(ir_type: IrType, operands: Vec<Pointer<Box<dyn Instruction>>>) -> User {
        let value = Value::make_value(ir_type);
        User { value, operands }
    }

    fn get_operands(&self) -> &Vec<Pointer<Box<dyn Instruction>>> {
        &self.operands
    }

    pub fn get_operand(&self, index: usize) -> Pointer<Box<dyn Instruction>> {
        self.get_operands()[index].clone()
    }

    pub fn get_operands_size(&self) -> usize {
        self.operands.len()
    }
}
