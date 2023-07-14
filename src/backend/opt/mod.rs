use crate::backend::module::AsmModule;
pub use crate::utility::ObjPtr;
pub use crate::backend::block::BB;
use crate::backend::instrs::*;
use crate::backend::operand;
use crate::backend::operand::*;
use crate::backend::BackendPool;
use crate::log;

mod peephole_pass;
mod clear_pass;
mod block_pass;

pub struct BackendPass {
    pub module: ObjPtr<AsmModule>,
}

impl BackendPass {
    pub fn new(module: ObjPtr<AsmModule>) -> Self {
        Self { module }
    }

    pub fn run_pass(&mut self, pool: &mut BackendPool) {
        self.peephole_pass(pool);
        self.clear_pass();
        self.block_pass(pool);
    }

    pub fn run_addition_block_pass(&mut self) {
        self.clear_useless_jump();
    }
}