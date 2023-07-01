mod color;
mod draw;
mod get;
mod init;
mod jud;
mod k_graph;
mod opt_gc_alloc2;
mod rescue;
mod simplify;
mod spill;
pub mod structs;

extern crate biheap;
use biheap::core::BiHeap;

use crate::backend::regalloc::regalloc;
use crate::backend::regalloc::regalloc::Regalloc;
use crate::backend::regalloc::structs::{FuncAllocStat, RegUsedStat};
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
        panic!("gg");
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
