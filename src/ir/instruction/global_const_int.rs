use crate::ir::{instruction::Instruction, ir_type::IrType, value::Value};
use std::{cell::RefCell, rc::Rc};

#[derive(Debug)]
pub struct GlobalConstInt {
    value: Value,
    bonding: i32,
}

impl GlobalConstInt {
    pub fn make_int(name: String, bonding: i32) -> Rc<RefCell<Instruction>> {
        Rc::new(RefCell::new(Instruction::IGlobalConstInt(GlobalConstInt {
            value: Value::make_value(name, IrType::ConstInt),
            bonding,
        })))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }
}
