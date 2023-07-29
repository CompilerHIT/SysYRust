mod color;
mod draw;
mod dump;
mod get;
mod init;
mod jud;
mod k_graph;
pub mod params;
mod rescue;
mod simplify;
mod spill;
pub mod structs;
extern crate biheap;
use biheap::core::BiHeap;

use crate::backend::regalloc::regalloc;
use crate::backend::regalloc::regalloc::Regalloc;
use crate::backend::regalloc::structs::{FuncAllocStat, RegUsedStat};
use crate::log_file;
use crate::{
    backend::{instrs::Func, operand::Reg},
    container::bitmap::Bitmap,
};
use std::collections::{HashMap, HashSet, LinkedList};
use structs::{ActionResult, AllocatorInfo, OperItem};

pub struct Allocator {
    info: Option<AllocatorInfo>,
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> FuncAllocStat {
        self.init(func);

        loop {
            let mut stat = self.color();
            while stat == ActionResult::Success || stat == ActionResult::Fail {
                if stat == ActionResult::Fail {
                    if self.simpilfy() == ActionResult::Success {
                        continue;
                    } else {
                        break;
                    }
                }
                stat = self.color();
            }
            while self.simpilfy() != ActionResult::Finish {
                continue;
            }
            while self.spill() != ActionResult::Finish {
                continue;
            }
            if self.check_k_graph() != ActionResult::Success {
                continue;
            }
            if self.color() != ActionResult::Finish {
                continue;
            }
            break;
        }
        self.color_k_graph();
        self.color_last();
        let (dstr, spillings) = self.draw_dstr_spillings();
        let (func_stack_size, bb_sizes) = regalloc::count_stack_size(func, &spillings);

        // 检查分配结果
        let p = "tmp4.txt";
        let regs = func.draw_all_virtual_regs();
        //
        log_file!(p, "func:{}", func.label);
        // log_file!(p, "{:?}", regs);
        for reg in regs {
            if dstr.contains_key(&reg.get_id()) || spillings.contains(&reg.get_id()) {
                continue;
            }
            log_file!(p, "{},", reg);
        }

        FuncAllocStat {
            dstr: dstr,
            spillings: spillings,
            stack_size: func_stack_size,
            bb_stack_sizes: bb_sizes,
        }
    }
}

//为它实现带约束分配trait
impl Allocator {
    pub fn alloc_with_constraints(
        &mut self,
        func: &Func,
        constraints: &HashMap<Reg, HashSet<Reg>>,
    ) -> FuncAllocStat {
        self.init_with_constraints(func, constraints);
        //TODO,加入约束
        loop {
            let mut stat = self.color();
            while stat == ActionResult::Success || stat == ActionResult::Fail {
                if stat == ActionResult::Fail {
                    if self.simpilfy() == ActionResult::Success {
                        continue;
                    } else {
                        break;
                    }
                }
                stat = self.color();
            }
            while self.simpilfy() != ActionResult::Finish {
                continue;
            }
            while self.spill() != ActionResult::Finish {
                continue;
            }
            if self.check_k_graph() != ActionResult::Success {
                continue;
            }
            if self.color() != ActionResult::Finish {
                continue;
            }
            break;
        }
        self.color_k_graph();
        self.color_last();
        let (dstr, spillings) = self.draw_dstr_spillings();
        let (func_stack_size, bb_sizes) = regalloc::count_stack_size(func, &spillings);

        // 检查分配结果
        let p = "tmp4.txt";
        let regs = func.draw_all_virtual_regs();
        //
        log_file!(p, "func:{}", func.label);
        // log_file!(p, "{:?}", regs);
        for reg in regs {
            if dstr.contains_key(&reg.get_id()) || spillings.contains(&reg.get_id()) {
                continue;
            }
            log_file!(p, "{},", reg);
        }

        FuncAllocStat {
            dstr: dstr,
            spillings: spillings,
            stack_size: func_stack_size,
            bb_stack_sizes: bb_sizes,
        }
    }
}
