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

pub type FILE_PATH = String;

pub fn generate_asm(path: &str, module: &mut AsmModule){
    module.generator(String::from(path));
}