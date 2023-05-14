use std::any::Any;

//todo:
pub mod binary_inst_cfg;
// pub mod branch_inst_cfg;
// pub mod call_inst_cfg;
pub mod const_int_cfg;
pub mod global_const_int_cfg;

pub enum CfgInstructionType {
    IBinaryOpInst,
    IBranchInst,
    IConstInt,
    IGlobalConstInt,
}

pub trait CfgInstruction {
    fn get_type(&self) -> CfgInstructionType;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
