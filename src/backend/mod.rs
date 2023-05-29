mod asm_builder;
mod instrs;
mod generate;
mod block;
mod func;
pub mod operand;
pub mod structs;
pub mod module;
pub mod regalloc;

use std::io::Write;
use std::fs::File;

use crate::backend::module::AsmModule;


pub fn generate_asm(path: &str, module: &mut AsmModule) {
    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(e) => panic!("Create output path error: {}", e),
    };
    writeln!(file, "	.file	\"{}\"", path);
    writeln!(file, "	.option pic");
    writeln!(file, "    .text");
    module.generator(&mut file);
    
    writeln!(file, "    .ident	\"GCC: (Ubuntu 11.3.0-1ubuntu1~22.04.1) 11.3.0\"");
    writeln!(file, "    .section	.note.GNU-stack,\"\",@progbits");
}