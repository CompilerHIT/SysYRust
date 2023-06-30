use crate::backend::module::AsmModule;
pub use crate::utility::ObjPtr;
pub use crate::backend::block::BB;
use crate::backend::instrs::*;
use crate::backend::operand::*;

mod peephole_pass;
mod clear_pass;

pub struct BackendPass {
    pub module: ObjPtr<AsmModule>,
}

impl BackendPass {
    pub fn new(module: ObjPtr<AsmModule>) -> Self {
        Self { module }
    }

    pub fn run_pass(&mut self) {
        self.clear_pass();
        self.peephole_pass();
    }
}