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

const optls_path: &str = "optls.txt";

// 摆烂的深度优先指令编码简单实现的线性扫描寄存器分配
pub struct Allocator {}

#[derive(Eq, PartialEq)]
struct RegInterval {
    pub reg: Reg,
    pub available: RegUsedStat,
    pub die: i32,
}
impl Display for RegInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.reg, self.die, self.available)
    }
}

impl RegInterval {
    fn new(reg: Reg, die: i32, reg_use_stat: RegUsedStat) -> RegInterval {
        RegInterval {
            reg: reg,
            die: die,
            available: reg_use_stat,
        }
    }
}

impl PartialOrd for RegInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.die.cmp(&other.die))
    }
}
impl Ord for RegInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        // Rust中BinaryHeap的默认实现是大根堆,我们需要的正是大根堆
        self.partial_cmp(other).unwrap()
    }
    fn max(self, other: Self) -> Self {
        let o = self.cmp(&other);
        match o {
            Ordering::Greater => self,
            Ordering::Equal => self,
            Ordering::Less => other,
        }
    }
}

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
