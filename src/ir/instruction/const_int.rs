use super::*;
use crate::ir::ir_type::IrType;
use crate::ir::user::User;
use crate::utility::Pointer;

pub struct ConstInt {
    user: User,
    bonding: i32,
}

impl ConstInt {
    pub fn make_int(bonding: i32) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(IrType::Int, vec![]);
        Pointer::new(Box::new(ConstInt { user, bonding }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }
}

impl Instruction for ConstInt {
    fn get_type(&self) -> InstructionType {
        InstructionType::IConstInt
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
