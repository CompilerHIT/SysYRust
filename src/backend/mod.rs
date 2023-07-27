mod asm_builder;
pub mod block;
pub mod func;
mod generate;
pub mod instrs;
pub mod module;
pub mod operand;
pub mod opt;
pub mod regalloc;
pub mod simulator;
pub mod structs;

use std::fs::File;
use std::io::Write;

use crate::backend::module::AsmModule;
use crate::backend::opt::BackendPass;
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

pub fn generate_asm(
    in_path: &str,
    path: &str,
    row_path: &str,
    module: &mut AsmModule,
    is_opt: bool,
) {
    let mut file = match File::create(path) {
        Ok(f) => f,
        Err(e) => panic!("Create    output path error: {}", e),
    };
    writeln!(file, "	.file	\"{}\"", in_path).unwrap();
    writeln!(file, "	.option pic").unwrap();
    writeln!(file, "    .text").unwrap();
    let mut pool = BackendPool::new();
    let mut file2 = File::create(row_path).unwrap();

    //构造
    // module.build_v3(&mut file, &mut file2, &mut pool, is_opt);
    module.build_v4(&mut file, &mut file2, &mut pool, is_opt);
    // module.generate_row_asm(&mut file2, &mut pool);

    // 后端优化
    if is_opt {
        BackendPass::new(ObjPtr::new(module)).run_pass(&mut pool);
    }

    // 检查地址溢出，插入间接寻址
    module.handle_overflow(&mut pool);

    //TODO: 块重排
    if is_opt {
        BackendPass::new(ObjPtr::new(module)).run_addition_block_pass();
    }

    //生成抽象汇编
    // module.generate_row_asm(&mut file2, &mut pool);

    //生成汇编
    module.generate_asm(&mut file, &mut pool);

    //释放
    pool.free_all();

    // writeln!(file, "    .ident	\"GCC: (Ubuntu 9.4.0-1ubuntu1~20.04) 9.4.0\"");
    writeln!(file, "    .section	.note.GNU-stack,\"\",@progbits").unwrap();
}
