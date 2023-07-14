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
    frontend::ast::{Break, Continue},
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

        ///基础着色,
        for bb in func.blocks.iter() {
            // 对于bb,首先从 live in中找出已用颜色
            let mut reg_use_stat: RegUsedStat = RegUsedStat::new();
            let mut live_now: HashSet<Reg> = HashSet::new();
            let mut last_used: HashMap<i32, Reg> = HashMap::new();
            bb.live_in.iter().for_each(|reg| {
                live_now.insert(*reg);
                if reg.is_physic() {
                    reg_use_stat.use_reg(reg.get_color());
                    last_used.insert(reg.get_color(), *reg);
                } else if dstr.contains_key(&reg.get_id()) {
                    let color = *dstr.get(&reg.get_id()).unwrap();
                    reg_use_stat.use_reg(color);
                    last_used.insert(color, *reg);
                }
            });
            let tmp_set = HashSet::new();
            for (index, inst) in bb.insts.iter().enumerate() {
                // 首先找出在这里终结的寄存器,把它们从live now中取出
                let finished = ends_index_bb.get(&(index, *bb)).unwrap_or(&tmp_set);
                for reg in finished {
                    debug_assert!(live_now.contains(reg));
                    live_now.remove(reg);
                    if reg.is_physic() {
                        reg_use_stat.release_reg(reg.get_color());
                        last_used.insert(reg.get_color(), *reg);
                    } else if dstr.contains_key(&reg.get_id()) {
                        let color = dstr.get(&reg.get_id()).unwrap();
                        reg_use_stat.release_reg(*color);
                        last_used.insert(*color, *reg);
                    }
                }
                //然后对于新发现的def的寄存器,给它选择一个可用颜色,或者加上它的可用颜色情况
                for reg in inst.get_reg_def() {
                    live_now.insert(reg);
                    //如果已有颜色
                    if reg.is_physic() || dstr.contains_key(&reg.get_id()) {
                        let color = if reg.is_physic() {
                            reg.get_color()
                        } else {
                            *dstr.get(&reg.get_id()).unwrap()
                        };
                        if reg_use_stat.is_available_reg(color) {
                            reg_use_stat.use_reg(color);
                            last_used.insert(color, reg);
                        } else {
                            let last_used_reg = last_used.get(&color).unwrap();
                            if last_used_reg.is_physic() {
                                dstr.remove(&reg.get_id());
                                spillings.insert(reg.get_id());
                            } else {
                                //否则判断哪个spill代价大,则spill代价小的一个
                                let last_spill_cost = spill_costs.get(last_used_reg);
                                let cur_spill_cost = spill_costs.get(&reg);
                                if last_spill_cost > cur_spill_cost {
                                    dstr.remove(&reg.get_id());
                                    spillings.insert(reg.get_id());
                                } else {
                                    spillings.insert(last_used_reg.get_id());
                                    dstr.remove(&last_used_reg.get_id());
                                    last_used.insert(color, reg);
                                }
                            }
                        }
                        continue;
                    }
                    //如果已经spillings
                    if spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    //否则进行分配
                    let color = reg_use_stat.get_available_reg(reg.get_type());
                    if color.is_none() {
                        spillings.contains(&reg.get_id());
                        continue;
                    }
                    let color = color.unwrap();
                    log_file!("ls_alloc.txt", "color:{}({})", reg, color);
                    dstr.insert(reg.get_id(), color);
                    reg_use_stat.use_reg(color);
                    last_used.insert(color, reg);
                }
            }
        }

        // TODO,循环裂变着色

        let (stackSize, bb_stack_sizes) = regalloc::regalloc::countStackSize(func, &spillings);
        FuncAllocStat {
            stack_size: stackSize,
            bb_stack_sizes: bb_stack_sizes,
            spillings: spillings,
            dstr: dstr,
        }
    }
}
