pub mod opt_gc_alloc2;

extern crate biheap;
use biheap::core::BiHeap;
use core::{panic, time};
use std::collections::{hash_map::Iter, HashMap, HashSet, LinkedList};

use super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};
use crate::{
    backend::{
        instrs::{Func, BB},
        operand::Reg,
    },
    container::bitmap::{self, Bitmap},
};

#[derive(PartialEq, Clone, Copy)]
pub struct OperItem {
    reg: Reg,
    cost: f32, //对于color过程,该cost是邻接度(小优先),对于rescue过程,是spillcost的值(大优先,先拯救spill代价大的东西),
               // 对于spill过程来说,该cost是spillcost的值(小优先),
               //因为数据会发生改变,所以最好每轮更新一下数据
}
impl OperItem {
    pub fn new(reg: &Reg, cost: &f32) -> OperItem {
        OperItem {
            reg: *reg,
            cost: *cost,
        }
    }
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
    pub k_graph: (BiHeap<OperItem>, Bitmap), //悬点集合,用来悬图优化,(临时悬点,可以用来切换颜色)
    pub to_simplify: BiHeap<OperItem>,       //准备化简保留的寄存器
    pub to_spill: BiHeap<OperItem>,          //待spill寄存器
    pub to_color: BiHeap<OperItem>,          //待着色寄存器
    pub colored: BiHeap<OperItem>,           //已着色寄存器
    pub last_colors: HashSet<Reg>,           //真正的弦点,永恒悬点
    pub spill_cost: HashMap<Reg, f32>,       //节点溢出代价 (用来启发寻找溢出代价最小的节点溢出)
    pub all_neighbors: HashMap<Reg, LinkedList<Reg>>, //所有邻居,在恢复节点的时候考虑,该表初始化后就不改变
    pub all_live_neighbors: HashMap<Reg, LinkedList<Reg>>, //还活着的邻居,在着色的时候动态考虑
    pub all_live_neigbhors_bitmap: HashMap<Reg, Bitmap>, //记录还活着的邻居 TODO,
    pub nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>>, //周围节点颜色数量
    pub availables: HashMap<Reg, RegUsedStat>,        //节点可着色资源
    pub colors: HashMap<i32, i32>,                    //着色情况
    pub spillings: HashSet<i32>,                      //溢出情况
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

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> FuncAllocStat {
        self.init(func);

        // loop {
        //     loop {
        //         let mut stat = self.color();
        //         if stat == ActionResult::Fail {
        //             stat = self.simpilfy();
        //             if stat == ActionResult::Success {
        //                 continue;
        //             }
        //             stat = self.spill();
        //         } else if stat == ActionResult::Finish {
        //             break;
        //         }
        //     }
        //     let mut stat = ActionResult::Finish;
        //     loop {
        //         stat = self.simpilfy();
        //         if stat == ActionResult::Finish {
        //             break;
        //         }
        //         if stat == ActionResult::Success {
        //             break;
        //         }
        //     }
        //     if stat == ActionResult::Success {
        //         continue;
        //     }
        //     loop {
        //         stat = self.spill();
        //         if stat == ActionResult::Finish || stat == ActionResult::Success {
        //             break;
        //         }
        //     }
        //     if stat == ActionResult::Success {
        //         continue;
        //     }
        //     let mut stat = self.check_k_graph();
        //     if stat == ActionResult::Success {
        //         break;
        //     } else {
        //         continue;
        //     }
        // }
        self.color_k_graph();
        // while self.merge() == ActionResult::Success {
        //     self.rescue();
        // }
        self.color_last();
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
