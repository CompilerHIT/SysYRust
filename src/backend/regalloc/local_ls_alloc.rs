// 进行了块局部优化的线性扫描寄存器分配
// 对整个函数寄存器分配的问题，取每个块的寄存器分配问题为子问题
// 使用子问题的解来合成整个寄存器分配问题的解

use crate::{utility::ObjPtr, backend::instrs::BB};

use super::regalloc::Regalloc;

pub struct Allocator {
    

}
impl Allocator {
    fn ilr_alloc_for_block(bb:ObjPtr<BB>){
        todo!()
    }
}

impl Regalloc for Allocator{
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> super::structs::FuncAllocStat {

        todo!()
    }
}