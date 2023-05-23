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

    pub fn get_operand(&self, index: usize) -> ObjPtr<Inst> {
        self.operands[index]
    }

    pub fn find_operand(&self, inst: &Inst) -> Option<usize> {
        self.operands.iter().position(|x| eq(x.as_ref(), inst))
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

    pub fn get_use_list(&self) -> &Vec<ObjPtr<Inst>> {
        &mut self.use_list
    }

    /// 表示当前指令被使用，将其加入use list
    /// 一个指令可能被一个指令重复使用两次，所以可能存在有相同的指令
    pub fn add_user(&mut self, inst: &Inst) {
        self.use_list.push(ObjPtr::new(inst));
    }

    // 找到当前指令在use list中的位置
    fn find_use(&self, inst: &Inst) -> usize {
        self.use_list
            .iter()
            .position(|x| eq(x.as_ref(), inst))
            .unwrap()
    }

    /// 当前指令不再被使用，删除将对方从use list中删除
    pub fn delete_user(&mut self, inst: &Inst) {
        debug_assert!(!self.use_list.contains(&ObjPtr::new(inst)), "delete_user()",);
        let index = self.find_use(inst);
        self.use_list.remove(index);
    }
}
