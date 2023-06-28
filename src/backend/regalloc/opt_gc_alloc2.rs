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
use biheap::core::BiHeap;
use core::panic;
use std::{
    collections::{HashMap, HashSet, LinkedList, VecDeque},
    fmt::{self, format},
};

use super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};

#[derive(PartialEq)]
pub struct OperItem {
    reg: Reg,
    cost: f32, //对于color过程,该cost是邻接度(小优先),对于rescue过程,是spillcost的值(大优先,但是会遍历所有),
               // 对于spill过程来说,该cost是spillcost的值(小优先),对于simplify来说(colored中选择节点来simplify,cost的值为邻接度,大度优先)
}
impl Eq for OperItem {}

impl PartialOrd for OperItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OperItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.cost < other.cost {
            std::cmp::Ordering::Less
        } else if (self.cost - other.cost).abs() < 10E-10 {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    }
}
//

pub struct AllocatorInfo {
    pub to_color: (BiHeap<OperItem>, Bitmap), //待着色寄存器
    pub to_save: (LinkedList<OperItem>, HashSet<i32>), //待拯救寄存器
    pub k_graph: (LinkedList<OperItem>, Bitmap), //悬点集合,用来悬图优化
    pub colored: (LinkedList<OperItem>, Bitmap), //已着色节点
    pub spill_cost: HashMap<Reg, f32>,        //节点溢出代价 (用来启发寻找溢出代价最小的节点溢出)
    pub all_neighbors: HashMap<Reg, LinkedList<Reg>>, //所有邻居,在恢复节点的时候考虑,该表初始化后就不改变
    pub live_neighbors: HashMap<Reg, LinkedList<Reg>>, //还活着的邻居,在着色的时候动态考虑
    pub nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>>, //周围节点颜色数量
    pub availables: HashMap<Reg, RegUsedStat>,        //节点可着色资源
    pub colors: HashMap<i32, i32>,                    //着色情况
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
                self.rescue(); //化简完先试图拯救下被spill的寄存器
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
