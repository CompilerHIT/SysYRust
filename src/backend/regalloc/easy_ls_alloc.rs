// 或者可以认为是没有启发的线性扫描寄存器分配

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::{
    backend::{
        block,
        instrs::BB,
        operand::Reg,
        regalloc::{self, regalloc::Regalloc, structs::RegUsedStat},
    },
    frontend::ast::Continue,
    log_file,
    utility::{ObjPtr, ScalarType},
};

use super::structs::FuncAllocStat;

pub struct Allocator {}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {}
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> FuncAllocStat {
        let mut dstr: HashMap<i32, i32> = HashMap::new();
        let mut spillings: HashSet<i32> = HashSet::new();
        let ends_index_bb = regalloc::regalloc::build_ends_index_bb(func);
        let interference_graph = regalloc::regalloc::build_interference(func, &ends_index_bb);
        let availables = regalloc::regalloc::build_availables(func, &interference_graph);
        // 获取周围的颜色,

        let (stackSize, bb_stack_sizes) = regalloc::regalloc::countStackSize(func, &spillings);
        FuncAllocStat {
            stack_size: stackSize,
            bb_stack_sizes: bb_stack_sizes,
            spillings: spillings,
            dstr: dstr,
        }
    }
}
