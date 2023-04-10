use super::Instruction;
use crate::ir::ir_type::IrType;
use crate::ir::value::Value;
use crate::utility::Pointer;

#[derive(Debug)]
pub struct ConstInt {
    value: Value,
    bonding: i32,
}

impl ConstInt {
    pub fn make_int(name: String, bonding: i32) -> Pointer<Instruction> {
        let value = Value::make_value(name, IrType::Int);
        Pointer::new(Instruction::IConstInt(ConstInt { value, bonding }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }
}
