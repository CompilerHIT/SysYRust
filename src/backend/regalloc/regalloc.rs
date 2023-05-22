use crate::backend::func::Func;
use crate::backend::regalloc::structs::FuncAllocStat;


// 该处理下，全局量被翻译到内存中，
// 以函数为寄存器分配的基本单位
pub trait Regalloc {
    fn alloc(&mut self,func :& Func)->FuncAllocStat;
}


