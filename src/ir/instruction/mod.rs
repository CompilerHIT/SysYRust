//! src/ir/Instruction/mod.rs
pub mod binary_inst;
pub mod branch_inst;
pub mod call_inst;
pub mod const_int;
pub mod global_const_int;

pub enum InstructionType {
    IBinaryOpInst,
    IBranchInst,
    IConstInt,
    IGlobalConstInt,
}

pub trait Instruction {
    fn get_type(&self) -> InstructionType;
}
