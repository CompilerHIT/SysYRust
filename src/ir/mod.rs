//! src/ir/mod.rs
mod basicblock;
mod function;
mod instruction;
mod ir_type;
mod module;
mod parameter;
mod user;
mod value;

pub use instruction::binary_inst::BinaryOpInst;
