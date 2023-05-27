mod asm_builder;
mod instrs;
mod generate;
mod block;
mod func;
pub mod operand;
pub mod structs;
pub mod module;
pub mod regalloc;

use std::io::Result;
use std::fs::File;

use crate::backend::module::AsmModule;


pub fn generate_asm(module: &mut AsmModule) {
    module.generator();
}