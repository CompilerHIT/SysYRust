// 或者可以认为是没有启发的线性扫描寄存器分配

use std::collections::{HashMap, HashSet};

use crate::backend::regalloc::regalloc::Regalloc;

pub struct Allocator {

}
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> super::structs::FuncAllocStat {
        // let mut dstr=HashMap::new();
        // let mut spillings=HashSet::new();
        // // 对每个块live in live out进行分析

        
        todo!()
    }
}