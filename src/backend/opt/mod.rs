use std::collections::HashMap;

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
mod particular_opt;
mod peephole_pass;

pub struct BackendPass {
    pub module: ObjPtr<AsmModule>,
}

impl BackendPass {
    pub fn new(module: ObjPtr<AsmModule>) -> Self {
        Self { module }
    }

    pub fn run_pass(&mut self, pool: &mut BackendPool) {
        self.block_pass_pre_clear(pool);
        self.clear_pass(pool);
        self.peephole_pass(pool);
    }

    pub fn run_addition_block_pass(&mut self, pool: &mut BackendPool) {
        // 块优化
        self.block_pass(pool);
    }
}
