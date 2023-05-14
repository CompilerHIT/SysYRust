use crate::{
    ir::{ir_type::IrType, user::User},
    utility::Pointer,
};

use super::{IList, Instruction};

pub struct Phi {
    user: User,
    list: IList,
}

impl Phi {
    pub fn new(ir_type: IrType) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![]);
        let inst = Phi {
            user,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    pub fn set_upstream(&mut self, inst: Pointer<Box<dyn Instruction>>) {
        self.user.push_operand(inst)
    }

    pub fn delete_upstream(&mut self, inst: &Pointer<Box<dyn Instruction>>) {
        self.user.delete_operand(inst)
    }

    pub fn get_upstreams(&self) -> &Vec<Pointer<Box<dyn Instruction>>> {
        self.user.get_operands()
    }

    pub fn get_upstreams_mut(&mut self) -> &mut Vec<Pointer<Box<dyn Instruction>>> {
        self.user.get_operands_mut()
    }
}

impl Instruction for Phi {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn get_inst_type(&self) -> super::InstructionType {
        super::InstructionType::IPhi
    }
    fn get_value_type(&self) -> crate::ir::ir_type::IrType {
        self.user.get_ir_type()
    }
    fn is_tail(&self) -> bool {
        self.list.is_tail()
    }
    fn next(&self) -> Option<crate::utility::Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }
    fn prev(&self) -> Option<crate::utility::Pointer<Box<dyn Instruction>>> {
        self.list.prev()
    }
    fn set_next(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.set_next(node)
    }
    fn set_prev(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(node)
    }
    fn insert_before(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.insert_before(node)
    }
    fn insert_after(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.insert_after(node)
    }
    fn remove_self(&mut self) {
        self.list.remove_self()
    }
    fn is_head(&self) -> bool {
        self.list.is_head()
    }
}
