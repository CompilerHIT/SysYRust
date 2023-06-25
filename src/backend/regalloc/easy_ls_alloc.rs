// 或者可以认为是没有启发的线性扫描寄存器分配

use std::{collections::{HashMap, HashSet}, fs};

use crate::{backend::{regalloc::{regalloc::Regalloc, self, structs::RegUsedStat}, instrs::BB, operand::Reg, block}, utility::{ObjPtr, ScalarType}, frontend::ast::Continue, log_file};

use super::structs::FuncAllocStat;

pub struct Allocator {
    
}

impl Allocator {
    pub fn new()->Allocator {
        Allocator {  }
    }
}
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> FuncAllocStat {
        let mut dstr:HashMap<i32,i32>=HashMap::new();
        let mut spillings:HashSet<i32>=HashSet::new();
        let (stackSize,bb_stack_sizes)=regalloc::regalloc::countStackSize(func, &spillings);
        FuncAllocStat { stack_size: stackSize, bb_stack_sizes: bb_stack_sizes, spillings: spillings, dstr: dstr }
    }
}