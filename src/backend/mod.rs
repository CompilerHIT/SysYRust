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


pub fn generate_asm(in_path: &str, path: &str, module: &mut AsmModule) {
    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(e) => panic!("Create output path error: {}", e),
    };
    writeln!(file, "	.file	\"{}\"", in_path);
    writeln!(file, "	.option pic");
    writeln!(file, "    .text");
    module.generator(&mut file);
    
    writeln!(file, "    .ident	\"GCC: (Ubuntu 9.4.0-1ubuntu1~20.04) 9.4.0\"");
    writeln!(file, "    .section	.note.GNU-stack,\"\",@progbits");
}