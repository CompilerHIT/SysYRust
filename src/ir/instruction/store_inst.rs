use crate::{
    ir::{ir_type::IrType, user::User},
    utility::Pointer,
};

use super::{IList, Instruction};

pub struct StoreInst {
    user: User,
    list: IList,
}

impl StoreInst {
    fn make_store_inst(
        ir_type: IrType,
        dest_ptr: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![dest_ptr]);
        let inst = StoreInst {
            user,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    pub fn make_int_store_inst(
        dest_ptr: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_store_inst(IrType::Int, dest_ptr)
    }

    pub fn make_float_store_inst(
        dest_ptr: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_store_inst(IrType::Float, dest_ptr)
    }

    pub fn get_dest_ptr(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(0)
    }
}

impl Instruction for StoreInst {
    fn get_inst_type(&self) -> super::InstructionType {
        super::InstructionType::IStoreInst
    }
    fn get_value_type(&self) -> IrType {
        self.user.get_ir_type()
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
    fn set_next(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.set_next(node)
    }
    fn remove_self(&mut self) {
        self.list.remove_self()
    }
    fn insert_before(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.insert_before(node)
    }
    fn insert_after(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.insert_after(node)
    }
    fn is_head(&self) -> bool {
        self.list.is_head()
    }
    fn is_tail(&self) -> bool {
        self.list.is_tail()
    }
    fn prev(&self) -> Option<crate::utility::Pointer<Box<dyn Instruction>>> {
        self.list.prev()
    }
    fn next(&self) -> Option<crate::utility::Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }
}
