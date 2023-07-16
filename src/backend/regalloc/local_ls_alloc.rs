// 进行了块局部优化的线性扫描寄存器分配
// 使用动态规划算法在合理时间内获取每个块的寄存器分配结果
// 枚举n*n的时间

use crate::{backend::instrs::BB, utility::ObjPtr};

use super::regalloc::Regalloc;

pub struct Allocator {}
impl Allocator {
    fn best_alloc_for_block(bb: ObjPtr<BB>) {
        todo!()
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> super::structs::FuncAllocStat {
        func.calc_live();
        todo!()
    }
}
