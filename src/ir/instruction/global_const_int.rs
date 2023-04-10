use crate::ir::{instruction::Instruction, ir_type::IrType, value::Value};
use crate::utility::Pointer;

#[derive(Debug)]
pub struct GlobalConstInt {
    value: Value,
    bonding: i32,
}

impl GlobalConstInt {
    pub fn make_int(bonding: i32) -> Pointer<Instruction> {
        Pointer::new(Instruction::IGlobalConstInt(GlobalConstInt {
            value: Value::make_value(IrType::ConstInt),
            bonding,
        }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }
}
