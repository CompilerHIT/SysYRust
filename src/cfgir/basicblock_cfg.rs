use super::{instruction_cfg::CfgInstruction, ir_type_cfg::CfgIrType, value_cfg::CfgValue};
use crate::utility::Pointer;

pub struct CfgBasicBlock {
    value: CfgValue,
    instruction: Vec<Pointer<Box<dyn CfgInstruction>>>,
}

impl CfgBasicBlock {
    /// 构造一个空的BisicBlock
    pub fn make_basicblock() -> CfgBasicBlock {
        let value = CfgValue::make_value(CfgIrType::BBlock);
        CfgBasicBlock {
            value,
            instruction: Vec::new(),
        }
    }

    /// 在index处插入一条指令
    pub fn insert(&mut self, inst: Pointer<Box<dyn CfgInstruction>>, index: usize) {
        self.instruction.insert(index, inst);
    }
}
