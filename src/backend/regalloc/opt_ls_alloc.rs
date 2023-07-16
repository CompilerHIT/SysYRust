// 优化线性寄存器分配
/*
已实现优化:

待实现优化:
1.寄存器合并


 */

use crate::algorithm::graphalgo;
use crate::algorithm::graphalgo::Graph;
use crate::backend::block::BB;
use crate::backend::func::Func;
use crate::backend::operand::Reg;
use crate::backend::regalloc::regalloc::Regalloc;
use crate::backend::regalloc::structs::{FuncAllocStat, RegUsedStat};
use crate::utility::ObjPtr;
use crate::utility::ScalarType;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

// 摆烂的深度优先指令编码简单实现的线性扫描寄存器分配
pub struct Allocator {}

#[derive(Eq, PartialEq)]
struct RegInterval {
    pub reg: Reg,
    pub die: usize,
}

impl RegInterval {
    fn new(reg: Reg, die: usize) -> RegInterval {
        RegInterval { reg: reg, die: die }
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
    //获取某个顺序的块结构
    pub fn order_blocks(blocks: &Vec<ObjPtr<BB>>) {}

    //对于某个块内进行分配
    fn alloc_block(bb: ObjPtr<BB>, colors: &mut HashMap<i32, i32>, spillings: &mut HashSet<i32>) {}
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat {
        func.build_reg_intervals();

        FuncAllocStat {
            stack_size: todo!(),
            bb_stack_sizes: todo!(),
            spillings: todo!(),
            dstr: todo!(),
        }
    }
}
