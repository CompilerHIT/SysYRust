// 优化的图着色寄存器分配
// 待实现优化
/*
1.基于贪心的color选择和spill选择
2.合并寄存器
 */


// 或者可以认为是没有启发的线性扫描寄存器分配

use std::{collections::{HashMap, HashSet}, fs};
use crate::{backend::regalloc::{easy_gc_alloc,structs::FuncAllocStat}, log_file};

use super::regalloc::{Regalloc, self};

pub struct Allocator {
    easy_gc_allocator:easy_gc_alloc::Allocator
}

impl Allocator {
    pub fn new()->Allocator {
        Allocator { easy_gc_allocator: easy_gc_alloc::Allocator::new() }
    }
}
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> FuncAllocStat {
        let mut out=self.easy_gc_allocator.alloc(func);
        // 检查下分配前的分配结果
        let path="befor_gc_opt.txt";
        log_file!(path,"func:{}\ndstr:{:?}\n\nspillings:{:?}",func.label,out.dstr,out.spillings);

        // 寄存器合并
        regalloc::merge_alloc(func, &mut out.dstr, &mut out.spillings, 
            & self.easy_gc_allocator.ends_index_bb, &mut self.easy_gc_allocator.nums_neighbor_color, &mut self.easy_gc_allocator.availables, &mut self.easy_gc_allocator.interference_graph);

        
        out
    }
}