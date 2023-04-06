//! src/ir/Instruction/mod.rs
pub mod binary_inst;
pub mod branch_inst;
pub mod call_inst;

use binary_inst::BinaryOpInst;
use branch_inst::BranchInst;

pub enum Instruction {
    EBinaryOpInst(BinaryOpInst),
    EBranchInst(BranchInst),
}
