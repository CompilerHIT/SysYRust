use super::*;
use crate::cfgir::ir_type_cfg::CfgIrType;
use crate::cfgir::user_cfg::CfgUser;
use crate::utility::Pointer;

pub struct CfgConstInt {
    user: CfgUser,
    bonding: i32,
    name: String,
}

impl CfgConstInt {
    pub fn make_int(bonding: i32, name: String) -> Pointer<Box<dyn CfgInstruction>> {
        let user = CfgUser::make_user(CfgIrType::Int, vec![]);
        Pointer::new(Box::new(CfgConstInt {
            user,
            bonding,
            name,
        }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }
}

impl CfgInstruction for CfgConstInt {
    fn get_type(&self) -> CfgInstructionType {
        CfgInstructionType::IConstInt
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
