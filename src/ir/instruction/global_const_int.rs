use crate::ir::{instruction::*, ir_type::IrType, value::Value};
use crate::utility::Pointer;

pub struct GlobalConstInt {
    value: Value,
    bonding: i32,
}

impl GlobalConstInt {
    pub fn make_int(bonding: i32) -> Pointer<Box<dyn Instruction>> {
        Pointer::new(Box::new(GlobalConstInt {
            value: Value::make_value(IrType::ConstInt),
            bonding,
        }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }
}

impl Instruction for GlobalConstInt {
    fn get_type(&self) -> InstructionType {
        InstructionType::IGlobalConstInt
    }
}
