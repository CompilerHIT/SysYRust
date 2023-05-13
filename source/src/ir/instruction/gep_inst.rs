use crate::{
    ir::{ir_type::IrType, user::User},
    utility::Pointer,
};

use super::{IList, Instruction};

pub struct GEPInst {
    user: User,
    list: IList,
}

impl GEPInst {
    fn new(
        ir_type: IrType,
        ptr: Pointer<Box<dyn Instruction>>,
        offset: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![ptr, offset]);
        let inst = GEPInst {
            user,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    pub fn make_int_ptr_gepinst(
        ptr: Pointer<Box<dyn Instruction>>,
        offset: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::new(IrType::IntPtr, ptr, offset)
    }

    pub fn make_float_ptr_gepinst(
        ptr: Pointer<Box<dyn Instruction>>,
        offset: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::new(IrType::FloatPtr, ptr, offset)
    }

    pub fn get_ptr(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(0)
    }

    pub fn set_ptr(&mut self, ptr: Pointer<Box<dyn Instruction>>) {
        self.user.set_operand(0, ptr)
    }

    pub fn get_offset(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(1)
    }

    pub fn set_offset(&mut self, offset: Pointer<Box<dyn Instruction>>) {
        self.user.set_operand(1, offset)
    }
}

impl Instruction for GEPInst {
    fn get_inst_type(&self) -> super::InstructionType {
        super::InstructionType::IGEPInst
    }
    fn get_value_type(&self) -> crate::ir::ir_type::IrType {
        self.user.get_ir_type()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn set_next(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.set_next(node)
    }
    fn set_prev(&mut self, node: crate::utility::Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(node)
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
    fn next(&self) -> Option<crate::utility::Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }
}
