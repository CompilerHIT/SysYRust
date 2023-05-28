mod asm_builder;
mod instrs;
mod generate;
mod block;
mod func;
pub mod operand;
pub mod structs;
pub mod module;
pub mod regalloc;

use std::fs::File;

use crate::backend::module::AsmModule;


pub fn generate_asm(path: &str, module: &mut AsmModule) {
    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(e) => panic!("Create output path error: {}", e),
    };
    module.generator(&mut file)
}