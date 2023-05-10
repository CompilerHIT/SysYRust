//! src/ir/mod.rs
pub mod basicblock;
pub mod function;
pub mod instruction;
pub mod ir_type;
pub mod module;
pub mod user;
pub mod value;

pub use instruction::binary_inst::BinaryOpInst;
