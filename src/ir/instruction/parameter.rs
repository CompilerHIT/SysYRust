use crate::ir::{ir_type::IrType, user::User};
use crate::utility::Pointer;

use super::{IList, Instruction};

struct Parameter {
    user: User,
    list: IList,
}

impl Parameter {
    fn make_parameter(ir_type: IrType) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![]);
        let list = IList {
            prev: None,
            next: None,
        };
        Pointer::new(Box::new(Parameter { user, list }))
    }

    pub fn make_int_parameter() -> Pointer<Box<dyn Instruction>> {
        Self::make_parameter(IrType::Int)
    }

    pub fn make_float_parameter() -> Pointer<Box<dyn Instruction>> {
        Self::make_parameter(IrType::Float)
    }
}

impl Instruction for Parameter {
    fn get_type(&self) -> super::InstructionType {
        super::InstructionType::IParameter
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn set_prev(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(node)
    }
    fn next(&self) -> Option<crate::utility::Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }
    fn set_next(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.set_next(node)
    }
    fn remove_self(&mut self) {
        self.list.remove_self()
    }

    fn is_tail(&self) -> bool {
        self.list.is_tail()
    }
    fn is_head(&self) -> bool {
        self.list.is_head()
    }
    fn insert_before(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.insert_before(node)
    }
    fn insert_after(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.insert_after(node)
    }
    fn prev(&self) -> Option<crate::utility::Pointer<Box<dyn Instruction>>> {
        self.list.prev()
    }
}
