use crate::cfgir::user_cfg::CfgUser;
use crate::cfgir::{instruction_cfg::*, ir_type_cfg::CfgIrType};
use crate::utility::Pointer;

pub struct CfgGlobalConstInt {
    user: CfgUser,
    bonding: i32,
    name: String,
}

impl CfgGlobalConstInt {
    pub fn make_int(bonding: i32, name: String) -> Pointer<Box<dyn CfgInstruction>> {
        Pointer::new(Box::new(CfgGlobalConstInt {
            user: CfgUser::make_user(CfgIrType::ConstInt, vec![]),
            bonding,
            name,
        }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }

    pub fn get_name(&self) -> &String {
        &(self.name) //todo:是否正确
    }
}

impl CfgInstruction for CfgGlobalConstInt {
    fn get_type(&self) -> CfgInstructionType {
        CfgInstructionType::IGlobalConstInt
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
