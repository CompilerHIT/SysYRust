// 最大化执行速度

use crate::{
    backend::{
        instrs::{Func, BB},
        operand::Reg,
    },
    container::bitmap::{self, Bitmap},
    log_file, log_file_uln,
    utility::{ObjPool, ObjPtr, ScalarType},
};
use core::panic;
use std::{
    collections::{HashMap, HashSet, LinkedList, VecDeque},
    fmt::{self, format},
    hash::Hash,
};

use super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};

pub struct Allocator {
    pub to_color: LinkedList<Reg>, //待着色寄存器
    pub to_save: LinkedList<Reg>,  //待恢复寄存器
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            regs: LinkedList::new(),
            colors: HashMap::new(),
            costs_reg: HashMap::new(),
            availables: HashMap::new(),
            interference_graph: HashMap::new(),
            interference_regs: HashSet::new(),
            spillings: HashSet::new(),
            nums_neighbor_color: HashMap::new(),
            ends_index_bb: HashMap::new(),
        }
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {}
}
