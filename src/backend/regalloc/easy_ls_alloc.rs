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
        let spill_costs = regalloc::regalloc::estimate_spill_cost(func);
        let interference_graph = regalloc::regalloc::build_interference(func);
        let availables = regalloc::regalloc::build_availables(func, &interference_graph);
        // 获取周围的颜色,
        for bb in func.blocks.iter() {
            // 对于bb,首先从 live in中找出已用颜色
            let mut reg_use_stat: RegUsedStat = RegUsedStat::new();
            let mut live_now: HashSet<Reg> = HashSet::new();
            bb.live_in.iter().for_each(|reg| {
                if reg.is_physic() {
                    reg_use_stat.use_reg(reg.get_color());
                } else if dstr.contains_key(&reg.get_id()) {
                    reg_use_stat.use_reg(*dstr.get(&reg.get_id()).unwrap());
                }
            });
            let tmp_set = HashSet::new();
            for (index, inst) in bb.insts.iter().enumerate() {
                // 首先找出在这里终结的寄存器,把它们从live now中取出
                let finished = ends_index_bb.get(&(index, *bb)).unwrap_or(&tmp_set);

                //然后对于新发现的def的寄存器,给它选择一个可用颜色,或者加上它的可用颜色情况
            }
        }
        let (stackSize, bb_stack_sizes) = regalloc::regalloc::countStackSize(func, &spillings);
        FuncAllocStat {
            stack_size: stackSize,
            bb_stack_sizes: bb_stack_sizes,
            spillings: spillings,
            dstr: dstr,
        }
    }
}
