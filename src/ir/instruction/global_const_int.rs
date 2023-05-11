use crate::ir::user::User;
use crate::ir::{instruction::*, ir_type::IrType};
use crate::utility::Pointer;

pub struct GlobalConstInt {
    user: User,
    bonding: i32,
}

impl GlobalConstInt {
    pub fn make_int(bonding: i32) -> Pointer<Box<dyn Instruction>> {
        Pointer::new(Box::new(GlobalConstInt {
            user: User::make_user(IrType::ConstInt, vec![]),
            bonding,
        }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }

    pub fn get_ir_type(&self) -> IrType {
        self.user.get_ir_type()
    }
}

impl Instruction for GlobalConstInt {
    fn get_inst_type(&self) -> InstructionType {
        InstructionType::IGlobalConstInt
    }
    fn get_value_type(&self) -> IrType {
        self.user.get_ir_type()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn next(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        None
    }

    fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        None
    }

    fn insert_before(&mut self, _node: Pointer<Box<dyn Instruction>>) {}
    fn insert_after(&mut self, _node: Pointer<Box<dyn Instruction>>) {}
    fn remove_self(&mut self) {}
    fn is_head(&self) -> bool {
        false
    }
    fn is_tail(&self) -> bool {
        false
    }

    fn set_prev(&mut self, _node: Pointer<Box<dyn Instruction>>) {}
    fn set_next(&mut self, _node: Pointer<Box<dyn Instruction>>) {}
}
