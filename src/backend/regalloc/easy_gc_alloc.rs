// a impl of graph color register alloc algo

use super::regalloc::Regalloc;

pub struct Allocator{
    
}

impl Regalloc for Allocator {
    fn alloc(&mut self,func :& crate::backend::func::Func)->super::structs::FuncAllocStat {
        todo!()
    }
}