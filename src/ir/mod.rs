//! src/ir/mod.rs
mod basicblock;
pub mod function;
pub mod instruction;
mod ir_type;
pub mod module;
mod parameter;
mod user;
mod value;

pub use instruction::binary_inst::BinaryOpInst;
