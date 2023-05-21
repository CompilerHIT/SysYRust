use std::ptr::eq;

use super::{instruction::Inst, ir_type::IrType, value::Value};
use crate::utility::ObjPtr;

pub struct User {
    value: Value,
    operands: Vec<ObjPtr<Inst>>,
    use_list: Vec<ObjPtr<Inst>>,
}

impl User {
    pub fn new(ir_type: IrType, operands: Vec<ObjPtr<Inst>>) -> User {
        let value = Value::new(ir_type);
        User {
            value,
            operands,
            use_list: vec![],
        }
    }

    pub fn get_operands(&self) -> &Vec<ObjPtr<Inst>> {
        &self.operands
    }
    pub fn get_operands_mut(&mut self) -> &mut Vec<ObjPtr<Inst>> {
        &mut self.operands
    }

    pub fn get_operand(&self, index: usize) -> ObjPtr<Inst> {
        self.operands[index]
    }

    pub fn get_operands_size(&self) -> usize {
        self.operands.len()
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }

    pub fn set_operand(&mut self, index: usize, operand: ObjPtr<Inst>) {
        self.operands[index] = operand;
    }

    pub fn push_operand(&mut self, operand: ObjPtr<Inst>) {
        self.operands.push(operand)
    }

    pub fn get_use_list(&mut self) -> &mut Vec<ObjPtr<Inst>> {
        &mut self.use_list
    }

    /// 表示当前指令被使用，将其加入use list
    pub fn used(&mut self, inst: ObjPtr<Inst>) {
        self.use_list.push(inst);
    }

    /// 当前指令不再被使用，删除将对方从use list中删除
    pub fn un_unsed(&mut self, inst: ObjPtr<Inst>) {
        self.use_list = self
            .use_list
            .iter()
            .filter(|x| !eq(x.as_ref(), inst.as_ref()))
            .cloned()
            .collect();
    }
}
