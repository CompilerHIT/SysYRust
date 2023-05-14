use super::{instruction_cfg::CfgInstruction, ir_type_cfg::CfgIrType, value_cfg::CfgValue};
use crate::utility::Pointer;
use std::cell::RefMut;

pub struct CfgUser {
    value: CfgValue,
    operands: Vec<Pointer<Box<dyn CfgInstruction>>>,
}

impl CfgUser {
    pub fn make_user(
        ir_type: CfgIrType,
        operands: Vec<Pointer<Box<dyn CfgInstruction>>>,
    ) -> CfgUser {
        let value = CfgValue::make_value(ir_type);
        CfgUser { value, operands }
    }

    fn get_operands(&self) -> &Vec<Pointer<Box<dyn CfgInstruction>>> {
        &self.operands
    }

    pub fn get_operand(&self, index: usize) -> Pointer<Box<dyn CfgInstruction>> {
        self.get_operands()[index].clone()
    }

    pub fn get_operands_size(&self) -> usize {
        self.operands.len()
    }
}
