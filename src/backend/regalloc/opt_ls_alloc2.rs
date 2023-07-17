// 优化线性寄存器分配
/*
已实现优化:

待实现优化:
1.寄存器合并


 */

use biheap::bivec::order;
use biheap::BiHeap;

use super::regalloc;
use super::structs::BlockAllocStat;
use crate::algorithm::graphalgo;
use crate::algorithm::graphalgo::Graph;
use crate::backend::block::{self, BB};
use crate::backend::func::Func;
use crate::backend::instrs::LIRInst;
use crate::backend::operand::Reg;
use crate::backend::regalloc::regalloc::Regalloc;
use crate::backend::regalloc::structs::{FuncAllocStat, RegUsedStat};
use crate::frontend::ast::Continue;
use crate::utility::ObjPtr;
use crate::utility::ScalarType;
use crate::{log_file, log_file_uln};
use core::panic;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::{write, Display};
use std::hash::Hash;
use std::slice::Windows;

const optls_path: &str = "optls2.txt";

struct Allocator {}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {}
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat {
        func.build_reg_intervals();
        let mut colors: HashMap<i32, i32> = HashMap::new();
        let mut spillings: HashSet<i32> = HashSet::new();
        let spill_costs = regalloc::estimate_spill_cost(func);
        // let
        let available = regalloc::init_availables(func);
        // let blocks = Allocator::order_blocks(&func.blocks);

        //TODO,寄存器分裂
        let (stack_size, bb_stack_sizes) = regalloc::countStackSize(func, &spillings);
        FuncAllocStat {
            stack_size: stack_size,
            bb_stack_sizes: bb_stack_sizes,
            spillings: spillings,
            dstr: colors,
        }
    }
}
