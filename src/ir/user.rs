use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use super::instruction::Instruction;
use super::ir_type::IrType;
use super::value::Value;

#[derive(Debug)]
pub struct User {
    value: Value,
    operands: Vec<Rc<RefCell<Instruction>>>,
}

impl User {
    pub fn make_user(
        name: String,
        ir_type: IrType,
        operands: Vec<Rc<RefCell<Instruction>>>,
    ) -> User {
        let value = Value::make_value(name, ir_type);
        User { value, operands }
    }

    fn get_operands(&self) -> &Vec<Rc<RefCell<Instruction>>> {
        &self.operands
    }

    pub fn get_operand(&self, index: usize) -> RefMut<Instruction> {
        (*self.get_operands()[index]).borrow_mut()
    }

    pub fn get_operands_size(&self) -> usize {
        self.operands.len()
    }
}
