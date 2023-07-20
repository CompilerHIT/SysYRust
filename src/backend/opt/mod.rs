use std::fs::OpenOptions;

pub use crate::backend::block::BB;
use crate::backend::instrs::*;
use crate::backend::module::AsmModule;
use crate::backend::operand;
use crate::backend::operand::*;
use crate::backend::BackendPool;
use crate::log;
pub use crate::utility::ObjPtr;

mod block_pass;
mod clear_pass;
mod peephole_pass;

pub struct BackendPass {
    pub module: ObjPtr<AsmModule>,
}

impl BackendPass {
    pub fn new(module: ObjPtr<AsmModule>) -> Self {
        Self { module }
    }

    pub fn run_pass(&mut self, pool: &mut BackendPool) {
        self.peephole_pass(pool);
        // self.module.generate_row_asm(
        //     &mut OpenOptions::new()
        //         .create(true)
        //         .append(true)
        //         .open("row_asm2.log")
        //         .unwrap(),
        //     pool,
        // );
        // self.module.print_func();
        self.clear_pass();
        // self.module.generate_row_asm(
        //     &mut OpenOptions::new()
        //         .create(true)
        //         .append(true)
        //         .open("row_asm2.log")
        //         .unwrap(),
        //     pool,
        // );
        // 清除无用指令之后开始栈空间重排
        // self.rearrange_stack_slot();
        self.block_pass(pool);
    }

    pub fn run_addition_block_pass(&mut self) {
        self.clear_useless_jump();
    }
}
