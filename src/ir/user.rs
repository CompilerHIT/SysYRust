use super::instruction::Instruction;
use super::ir_type::IrType;
use super::value::Value;
use crate::utility::Pointer;
use std::cell::RefMut;

#[derive(Debug)]
pub struct User {
    value: Value,
    operands: Vec<Pointer<Instruction>>,
}

impl User {
    pub fn make_user(name: String, ir_type: IrType, operands: Vec<Pointer<Instruction>>) -> User {
        let value = Value::make_value(name, ir_type);
        User { value, operands }
    }

    fn get_operands(&self) -> &Vec<Pointer<Instruction>> {
        &self.operands
    }

    pub fn get_operand(&self, index: usize) -> RefMut<Instruction> {
        self.get_operands()[index].borrow_mut()
    }

    pub fn get_operands_size(&self) -> usize {
        self.operands.len()
    }
}
