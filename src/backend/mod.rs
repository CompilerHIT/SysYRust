mod asm_builder;
mod instrs;
pub mod operand;
pub mod structs;
pub mod module;

use std::io::Result;
use std::fs::File;

use self::module::AsmModule;

// TODO: design program class for main.rs to start
pub fn generate_asm(path: &str, module: &mut AsmModule) -> Result<()> {
    module.generator(&mut File::create(path)?)
}