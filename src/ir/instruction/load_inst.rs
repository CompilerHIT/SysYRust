use crate::ir::{ir_type::IrType, user::User};
use crate::utility::Pointer;

use super::{IList, Instruction, InstructionType};

pub struct LoadInst {
    user: User,
    list: IList,
}

impl LoadInst {
    fn make_load_inst(
        ir_type: IrType,
        value: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![value]);
        let inst = LoadInst {
            user,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    pub fn make_int_load(value: Pointer<Box<dyn Instruction>>) -> Pointer<Box<dyn Instruction>> {
        Self::make_load_inst(IrType::Int, value)
    }

    pub fn make_float_load(value: Pointer<Box<dyn Instruction>>) -> Pointer<Box<dyn Instruction>> {
        Self::make_load_inst(IrType::Float, value)
    }

    pub fn make_int_ptr_load(
        value: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_load_inst(IrType::IntPtr, value)
    }

    pub fn make_float_ptr_load(
        value: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_load_inst(IrType::FloatPtr, value)
    }

    pub fn get_pointer_operand(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(0)
    }

    pub fn set_pointer_operand(&mut self, ptr: Pointer<Box<dyn Instruction>>) {
        self.user.set_operand(0, ptr)
    }
}

impl Instruction for LoadInst {
    fn get_inst_type(&self) -> super::InstructionType {
        InstructionType::ILoadInst
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn next(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }

    fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.prev()
    }

    fn set_next(&mut self, next: Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(next)
    }

    fn set_prev(&mut self, prev: Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(prev)
    }

    fn insert_before(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.insert_before(node)
    }

    fn insert_after(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.insert_after(node)
    }

    fn is_head(&self) -> bool {
        self.list.is_head()
    }

    fn is_tail(&self) -> bool {
        self.list.is_tail()
    }

    fn remove_self(&mut self) {
        self.list.remove_self()
    }
    fn get_value_type(&self) -> IrType {
        self.user.get_ir_type()
    }
}
