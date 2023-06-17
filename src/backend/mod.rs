mod asm_builder;
pub mod block;
pub mod func;
mod generate;
mod instrs;
pub mod module;
pub mod operand;
pub mod regalloc;
pub mod structs;

use std::fs::File;
use std::io::Write;

use crate::backend::module::AsmModule;
use crate::utility::ObjPool;

use self::func::Func;
use self::instrs::{Context, LIRInst, ObjPtr, BB};

pub struct BackendPool {
    func_pool: ObjPool<Func>,
    block_pool: ObjPool<BB>,
    inst_pool: ObjPool<LIRInst>,
    context_pool: ObjPool<Context>,
}

impl BackendPool {
    pub fn new() -> Self {
        Self {
            func_pool: ObjPool::new(),
            block_pool: ObjPool::new(),
            inst_pool: ObjPool::new(),
            context_pool: ObjPool::new(),
        }
    }

    pub fn put_func(&mut self, func: Func) -> ObjPtr<Func> {
        self.func_pool.put(func)
    }

    pub fn put_block(&mut self, block: BB) -> ObjPtr<BB> {
        self.block_pool.put(block)
    }

    pub fn put_inst(&mut self, inst: LIRInst) -> ObjPtr<LIRInst> {
        self.inst_pool.put(inst)
    }

    pub fn put_context(&mut self, context: Context) -> ObjPtr<Context> {
        self.context_pool.put(context)
    }

    pub fn free_all(&mut self) {
        self.func_pool.free_all();
        self.block_pool.free_all();
        self.inst_pool.free_all();
        self.context_pool.free_all();
    }
}

pub fn generate_asm(in_path: &str, path: &str, row_path: &str, module: &mut AsmModule) {
    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(e) => panic!("Create output path error: {}", e),
    };
    writeln!(file, "	.file	\"{}\"", in_path);
    writeln!(file, "	.option pic");
    writeln!(file, "    .text");
    let mut pool = BackendPool::new();
    let mut file2 = match File::create(row_path) {
        Ok(f) => f,
        Err(e) => panic!("Create output path error: {}", e),
    };
    module.generator(&mut file, &mut file2, &mut pool);

    pool.free_all();

    // writeln!(file, "    .ident	\"GCC: (Ubuntu 9.4.0-1ubuntu1~20.04) 9.4.0\"");
    writeln!(file, "    .section	.note.GNU-stack,\"\",@progbits");
}