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
    pub regs: LinkedList<Reg>,                 //所有虚拟寄存器的列表
    pub colors: HashMap<i32, i32>,             // 保存着色结果
    pub costs_reg: HashMap<Reg, f32>,          //记录虚拟寄存器的使用次数(作为代价)
    pub availables: HashMap<Reg, RegUsedStat>, // 保存每个点的可用寄存器集合
    pub nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>>,
    pub ends_index_bb: HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
    pub interference_regs: HashSet<Reg>,
    pub interference_graph: HashMap<Reg, LinkedList<Reg>>, //记录冲突
    pub spillings: HashSet<i32>,                           //记录溢出寄存器
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
