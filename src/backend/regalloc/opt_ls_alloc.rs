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
pub struct Allocator {
    
    passed_bbs:HashSet<ObjPtr<BB>>,
    count_spill_cost:HashMap<i32,i32>,  //发生spill的代价

}

#[derive(Eq, PartialEq)]
struct RegInterval {
    pub id: i32,
    pub end: usize,
}

impl RegInterval {
    fn new(id: i32, end: usize) -> RegInterval {
        RegInterval { id, end }
    }
}

impl PartialOrd for RegInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.end.cmp(&other.end))
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
        Allocator {
            passed_bbs: HashSet::new(),
            count_spill_cost: HashMap::new(),
        }
    }

    fn alloc_block(bb:ObjPtr<BB>){

    }
  
    
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat {


        FuncAllocStat { stack_size:todo!(), bb_stack_sizes: todo!(), spillings: todo!(), dstr:todo!()  }
    }
}
