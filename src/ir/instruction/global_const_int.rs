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
}

impl Instruction for GlobalConstInt {
    fn get_type(&self) -> InstructionType {
        InstructionType::IGlobalConstInt
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
