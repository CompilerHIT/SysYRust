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

//

pub struct AllocatorInfo {
    pub to_color: (LinkedList<Reg>, Bitmap),      //待着色寄存器
    pub to_save: (LinkedList<Reg>, HashSet<i32>), //待拯救寄存器
    pub k_graph: (LinkedList<Reg>, Bitmap),       //悬点集合
    pub spill_cost: HashMap<Reg, f32>, //节点溢出代价 (用来启发寻找溢出代价最小的节点溢出)
    pub all_neighbors: HashMap<Reg, LinkedList<Reg>>, //所有邻居
    pub live_neighbors: HashMap<Reg, LinkedList<Reg>>, //还活着的邻居
    pub nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>>, //周围节点颜色数量
    pub availables: HashMap<Reg, RegUsedStat>, //节点可着色资源
    pub colors: HashMap<i32, i32>,     //着色情况
}
#[derive(PartialEq, Eq)]
pub enum ActionResult {
    Finish,
    Unfinish,
    Success,
    Fail,
}

pub struct Allocator {
    info: Option<AllocatorInfo>,
}
impl Allocator {
    pub fn new() -> Allocator {
        Allocator { info: None }
    }
    pub fn init(&mut self, func: &Func) {
        todo!()
    }

    pub fn color(&mut self) -> ActionResult {
        todo!()
    }
    pub fn simpilfy(&mut self) -> ActionResult {
        todo!()
    }
    pub fn spill(&mut self) -> ActionResult {
        todo!()
    }

    pub fn color_k_graph(&mut self) -> ActionResult {
        todo!()
    }
    pub fn is_k_graph_node(&mut self) -> bool {
        todo!()
    }

    pub fn merge(&mut self) -> ActionResult {
        todo!()
    }

    #[inline]
    pub fn rescue(&mut self) -> ActionResult {
        todo!()
    }

    #[inline]
    pub fn draw_dstr_spillings(&mut self) -> (HashMap<i32, i32>, HashSet<i32>) {
        let dstr = self.info.as_ref().unwrap().colors.to_owned();
        let spillings = self.info.as_ref().unwrap().to_save.1.to_owned();
        (dstr, spillings)
    }

    #[inline]
    pub fn color_one(&mut self, reg: Reg) {}
    #[inline]
    pub fn swap_color(&mut self, reg1: Reg, reg2: Reg) {}
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {
        self.init(func);
        while !(self.color() == ActionResult::Finish) {
            if self.simpilfy() == ActionResult::Success {
                continue;
            }
            self.spill();
        }
        self.color_k_graph();
        while self.merge() == ActionResult::Success {
            self.rescue();
        }

        let (dstr, spillings) = self.draw_dstr_spillings();
        let (func_stack_size, bb_sizes) = regalloc::countStackSize(func, &spillings);

        FuncAllocStat {
            dstr: dstr,
            spillings: spillings,
            stack_size: func_stack_size,
            bb_stack_sizes: bb_sizes,
        }
    }
}
