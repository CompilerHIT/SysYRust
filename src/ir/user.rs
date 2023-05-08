use super::{instruction::Instruction, ir_type::IrType, value::Value};
use crate::utility::Pointer;

pub struct User {
    value: Value,
    operands: Vec<Pointer<Box<dyn Instruction>>>,
    use_list: Vec<Pointer<Box<dyn Instruction>>>,
}

impl User {
    pub fn make_user(ir_type: IrType, operands: Vec<Pointer<Box<dyn Instruction>>>) -> User {
        let value = Value::make_value(ir_type);
        User {
            value,
            operands,
            use_list: vec![],
        }
    }

    pub fn get_operand(&self, index: usize) -> Pointer<Box<dyn Instruction>> {
        self.operands[index].clone()
    }

    pub fn get_operands_size(&self) -> usize {
        self.operands.len()
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }

    pub fn set_operand(&mut self, index: usize, operand: Pointer<Box<dyn Instruction>>) {
        self.operands[index] = operand;
    }

    pub fn get_use_list(&mut self) -> &mut Vec<Pointer<Box<dyn Instruction>>> {
        &mut self.use_list
    }
}
