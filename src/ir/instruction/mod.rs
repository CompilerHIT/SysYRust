//! src/ir/Instruction/mod.rs

use std::any::Any;

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

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
