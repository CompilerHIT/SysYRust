mod color;
mod draw;
mod dump;
mod get;
mod init;
mod jud;
mod k_graph;
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
        // panic!("gg");
        log_file!("opt2.txt", "func:{}", func.label);
        // self.dump_all_neighbors();
        self.dump_live_neighbors();
        // self.dump_last_colors();

        self.dump_tocolor();

        loop {
            let mut stat = self.color();
            if stat == ActionResult::Success {
                continue;
            }
            // self.check_k_graph();
            if stat == ActionResult::Finish {
                if self.simpilfy() != ActionResult::Finish {
                    continue;
                }
                if self.spill() != ActionResult::Finish {
                    continue;
                }
                if self.check_k_graph() != ActionResult::Success {
                    continue;
                }
                break;
            }
            stat = self.simpilfy();
            if stat == ActionResult::Success {
                continue;
            }
            stat = self.spill();
        }

        // loop {
        //     while self.color() != ActionResult::Finish {}
        //     self.dump_colors();
        //     self.dump_spillings();
        //     while self.simpilfy() != ActionResult::Finish {}
        //     self.dump_colors();
        //     self.dump_spillings();
        //     while self.spill() != ActionResult::Finish {}
        //     self.dump_colors();
        //     self.dump_spillings();
        //     if self.check_k_graph() == ActionResult::Success {
        //         break;
        //     }
        //     continue;
        // }

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
