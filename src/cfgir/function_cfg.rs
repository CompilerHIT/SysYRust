use super::{
    basicblock_cfg::CfgBasicBlock, ir_type_cfg::CfgIrType, parameter_cfg::CfgParameter,
    value_cfg::CfgValue,
};
use crate::utility::Pointer;
use std::cell::RefMut;

pub struct CfgFunction {
    value: CfgValue,
    parameters: Vec<CfgParameter>,
    head_block: Pointer<CfgBasicBlock>,
}

impl CfgFunction {
    fn make_function(
        parameters: Vec<CfgParameter>,
        head_block: Pointer<CfgBasicBlock>,
    ) -> CfgFunction {
        CfgFunction {
            value: CfgValue::make_value(CfgIrType::Function),
            parameters,
            head_block,
        }
    }

    pub fn get_head(&self) -> RefMut<CfgBasicBlock> {
        self.head_block.borrow_mut()
    }
}
