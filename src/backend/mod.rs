mod asm_builder;
mod instrs;
mod operand;
mod structs;
mod module;

use std::io::Result;
use std::fs::File;

// TODO: design program class for main.rs to start
// pub fn generate_asm(program: &Program, path: &str) -> Result<()> {
//     program.generate(&mut File::create(path)?, &mut ProgramInfo::new(program))
// }