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

    pub fn get_operands(&self) -> &Vec<Pointer<Box<dyn Instruction>>> {
        &self.operands
    }
    pub fn get_operands_mut(&mut self) -> &mut Vec<Pointer<Box<dyn Instruction>>> {
        &mut self.operands
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

    pub fn delete_operand(&mut self, operand: &Pointer<Box<dyn Instruction>>) {
        self.use_list = self
            .use_list
            .iter()
            .filter(|&x| x != operand)
            .cloned()
            .collect();
    }

    pub fn push_operand(&mut self, operand: Pointer<Box<dyn Instruction>>) {
        self.operands.push(operand)
    }

    pub fn get_use_list(&mut self) -> &mut Vec<Pointer<Box<dyn Instruction>>> {
        &mut self.use_list
    }

    pub fn used(&mut self, inst: Pointer<Box<dyn Instruction>>) {
        self.use_list.push(inst);
    }

    pub fn delete_user(&mut self, inst: &Pointer<Box<dyn Instruction>>) {
        self.use_list = self
            .use_list
            .iter()
            .filter(|&x| x != inst)
            .cloned()
            .collect();
    }
}
