// 优化的图着色寄存器分配
// 待实现优化
/*
1.基于贪心的color选择和spill选择
2.合并寄存器
 */

use crate::{
    backend::{
        operand::Reg,
        regalloc::{easy_gc_alloc, structs::FuncAllocStat},
    },
    container::bitmap::Bitmap,
    log_file, log_file_uln,
};
use core::panic;
use std::{
    collections::{HashMap, HashSet, LinkedList, VecDeque},
    fs,
};

use super::regalloc::{self, Regalloc};

pub struct Allocator {}

// 针对只有一个块的函数的最优化
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> FuncAllocStat {
        func.calc_live_for_alloc_reg();
        if func.blocks.len() != 2 {
            unreachable!();
        }
        //当函数只有一个块的时候才会跑该优化

        todo!();
    }
}
