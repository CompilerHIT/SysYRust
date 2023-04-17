//! src/ir/Instruction/mod.rs
pub mod binary_inst;
pub mod branch_inst;
pub mod call_inst;
pub mod const_int;
pub mod global_const_int;

use self::const_int::ConstInt;
use binary_inst::BinaryOpInst;
use branch_inst::BranchInst;
use global_const_int::GlobalConstInt;

#[derive(Debug)]
pub enum Instruction {
    IBinaryOpInst(BinaryOpInst),
    IBranchInst(BranchInst),
    IConstInt(ConstInt),
    IGlobalConstInt(GlobalConstInt),
}
