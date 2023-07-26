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
        self.block_pass_pre_clear(pool);
        self.clear_pass(pool);
        // 清除无用指令之后开始栈空间重排
        // self.rearrange_stack_slot();
        self.block_pass();
        self.peephole_pass(pool);
    }

    pub fn run_addition_block_pass(&mut self) {
        // 清除空块(包括entry块)
        self.clear_empty_block();
        // jump的目标块如果紧邻，则删除jump语句
        self.clear_useless_jump();
    }
}
