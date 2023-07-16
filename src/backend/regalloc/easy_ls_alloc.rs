// 或者可以认为是没有启发的线性扫描寄存器分配

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
};

use biheap::BiHeap;

use crate::{
    backend::{
        block,
        instrs::{Func, BB},
        operand::Reg,
        regalloc::{self, regalloc::Regalloc, structs::RegUsedStat},
    },
    frontend::ast::{Break, Continue},
    log, log_file,
    utility::{ObjPtr, ScalarType},
};

use super::structs::FuncAllocStat;

const easy_ls_path: &str = "easyls.txt";

struct BlocksVec(Vec<ObjPtr<BB>>);

pub struct Allocator {}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {}
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> FuncAllocStat {
        func.calc_live();
        log_file!(easy_ls_path, "func:{}", func.label);
        let mut colors: HashMap<i32, i32> = HashMap::new();
        let mut spillings: HashSet<i32> = HashSet::new();
        let ends_index_bb = regalloc::regalloc::build_ends_index_bb(func);
        ends_index_bb.iter().for_each(|((index, bb), ends)| {
            let path = "ends.txt";
            log_file!(
                path,
                "func:{},block:{},index:{}",
                func.label,
                bb.label,
                index
            );
            log_file!(path, "{:?}", ends);
        });
        let spill_costs = regalloc::regalloc::estimate_spill_cost(func);

        let mut blocks: Vec<ObjPtr<BB>> = func.blocks.iter().cloned().collect();
        // blocks.sort_by_key(|bb| bb.live_in.len());
        ///基础着色,
        Allocator::alloc_one_opt(
            &blocks,
            &ends_index_bb,
            &mut spillings,
            &mut colors,
            &spill_costs,
        );

        loop {
            spillings.clear();
            blocks.reverse();
            let mut if_opt = Allocator::alloc_one_fixed(
                &blocks,
                &ends_index_bb,
                &mut spillings,
                &mut colors,
                &spill_costs,
            );
            if if_opt {
                continue;
            }
            blocks.reverse();
            if_opt = Allocator::alloc_one_fixed(
                &blocks,
                &ends_index_bb,
                &mut spillings,
                &mut colors,
                &spill_costs,
            );
            if if_opt {
                continue;
            }
            break;
        }
        // TODO,循环裂变着色
        let (stackSize, bb_stack_sizes) = regalloc::regalloc::countStackSize(func, &spillings);
        FuncAllocStat {
            stack_size: stackSize,
            bb_stack_sizes: bb_stack_sizes,
            spillings: spillings,
            dstr: colors,
        }
    }
}

impl Allocator {
    ///每次产生新的colored的时候就返回true
    pub fn alloc_one_fixed(
        blocks: &Vec<ObjPtr<BB>>,
        ends_index_bb: &HashMap<(usize, ObjPtr<BB>), HashSet<Reg>>,
        spillings: &mut HashSet<i32>,
        colors: &mut HashMap<i32, i32>,
        spill_costs: &HashMap<Reg, f32>,
    ) -> bool {
        ///基础着色
        let mut fixed: HashSet<i32> = HashSet::new();
        for (reg, _) in colors.iter() {
            fixed.insert(*reg);
        }
        for bb in blocks.iter() {
            // 对于bb,首先从 live in中找出已用颜色
            let mut reg_use_stat: RegUsedStat = RegUsedStat::new();
            let mut live_now: HashSet<Reg> = HashSet::new();
            let mut last_used: HashMap<i32, Reg> = HashMap::new();
            log_file!(easy_ls_path, "{},live in:{:?}", bb.label, bb.live_in);
            bb.live_in.iter().for_each(|reg| {
                if reg.is_physic() || colors.contains_key(&reg.get_id()) {
                    Allocator::process_one_reg_fixed(
                        reg,
                        &mut live_now,
                        &mut last_used,
                        &mut reg_use_stat,
                        colors,
                        &spill_costs,
                        spillings,
                        &mut fixed,
                    );
                } else if !spillings.contains(&reg.get_id()) {
                    Allocator::color_one(reg, &mut reg_use_stat, spillings, colors, &mut last_used);
                }
                live_now.insert(*reg);
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
                        last_used.remove(&reg.get_color());
                    } else if colors.contains_key(&reg.get_id()) {
                        let color = colors.get(&reg.get_id()).unwrap();
                        reg_use_stat.release_reg(*color);
                        last_used.remove(color);
                    }
                }
                //然后对于新发现的def的寄存器,给它选择一个可用颜色,或者加上它的可用颜色情况
                for reg in inst.get_reg_def() {
                    if reg.is_physic() || colors.contains_key(&reg.get_id()) {
                        Allocator::process_one_reg_fixed(
                            &reg,
                            &mut live_now,
                            &mut last_used,
                            &mut reg_use_stat,
                            colors,
                            &spill_costs,
                            spillings,
                            &mut fixed,
                        );
                    } else if !spillings.contains(&reg.get_id()) {
                        Allocator::color_one(
                            &reg,
                            &mut reg_use_stat,
                            spillings,
                            colors,
                            &mut last_used,
                        );
                    }
                    live_now.insert(reg);
                }
            }
        }

        if colors.len() > fixed.len() {
            return true;
        }
        return false;
    }

    // 不断每次采用opt方式进行选择,直到spill cost减少为止
    pub fn alloc_one_opt(
        blocks: &Vec<ObjPtr<BB>>,
        ends_index_bb: &HashMap<(usize, ObjPtr<BB>), HashSet<Reg>>,
        spillings: &mut HashSet<i32>,
        colors: &mut HashMap<i32, i32>,
        spill_costs: &HashMap<Reg, f32>,
    ) {
        ///基础着色
        for bb in blocks.iter() {
            // 对于bb,首先从 live in中找出已用颜色
            let mut reg_use_stat: RegUsedStat = RegUsedStat::new();
            let mut live_now: HashSet<Reg> = HashSet::new();
            let mut last_used: HashMap<i32, Reg> = HashMap::new();
            log_file!(easy_ls_path, "{},live in:{:?}", bb.label, bb.live_in);
            log_file!(
                easy_ls_path,
                "insts:{:?}",
                bb.as_ref()
                    .insts
                    .iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<String>>()
            );
            bb.live_in.iter().for_each(|reg| {
                if reg.is_physic() || colors.contains_key(&reg.get_id()) {
                    Allocator::process_one_reg_opt(
                        reg,
                        &mut live_now,
                        &mut last_used,
                        &mut reg_use_stat,
                        colors,
                        &spill_costs,
                        spillings,
                    );
                } else if !spillings.contains(&reg.get_id()) {
                    Allocator::color_one(reg, &mut reg_use_stat, spillings, colors, &mut last_used);
                }
                live_now.insert(*reg);
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
                        last_used.remove(&reg.get_color());
                    } else if colors.contains_key(&reg.get_id()) {
                        let color = colors.get(&reg.get_id()).unwrap();
                        reg_use_stat.release_reg(*color);
                        last_used.remove(color);
                    }
                }
                //然后对于新发现的def的寄存器,给它选择一个可用颜色,或者加上它的可用颜色情况
                for reg in inst.get_reg_def() {
                    if reg.is_physic() || colors.contains_key(&reg.get_id()) {
                        Allocator::process_one_reg_opt(
                            &reg,
                            &mut live_now,
                            &mut last_used,
                            &mut reg_use_stat,
                            colors,
                            &spill_costs,
                            spillings,
                        );
                    } else if !spillings.contains(&reg.get_id()) {
                        Allocator::color_one(
                            &reg,
                            &mut reg_use_stat,
                            spillings,
                            colors,
                            &mut last_used,
                        );
                    }
                    live_now.insert(reg);
                }
            }
        }
    }

    ///处理遇到的有色寄存器的逻辑, (opt类型,会选择一个spill cost大的删除)
    pub fn process_one_reg_opt(
        reg: &Reg,
        live_now: &mut HashSet<Reg>,
        last_used: &mut HashMap<i32, Reg>,
        reg_use_stat: &mut RegUsedStat,
        colors: &mut HashMap<i32, i32>,
        spill_costs: &HashMap<Reg, f32>,
        spillings: &mut HashSet<i32>,
    ) {
        debug_assert!(reg.is_physic() || colors.contains_key(&reg.get_id()));
        if live_now.contains(reg) {
            return;
        }
        let reg = *reg;
        let color = if reg.is_physic() {
            reg.get_color()
        } else {
            *colors.get(&reg.get_id()).unwrap()
        };
        if !last_used.contains_key(&color) {
            // debug_assert!(reg_use_stat.is_available_reg(color), "color:{color}");    //比如zero 就不是available的
            log_file!(easy_ls_path, "init color:{color}");
            reg_use_stat.use_reg(color);
            last_used.insert(color, reg);
        } else if !reg.is_physic() {
            let last_used_reg = last_used.get(&color).unwrap();
            if last_used_reg.is_physic() {
                log_file!(easy_ls_path, "spill:{reg}");
                colors.remove(&reg.get_id());
                spillings.insert(reg.get_id());
            } else {
                //否则判断哪个spill代价大,则spill代价小的一个
                let last_spill_cost = spill_costs.get(last_used_reg);
                let cur_spill_cost = spill_costs.get(&reg);
                if last_spill_cost > cur_spill_cost {
                    log_file!(easy_ls_path, "spill:{reg}");
                    colors.remove(&reg.get_id());
                    spillings.insert(reg.get_id());
                } else {
                    log_file!(easy_ls_path, "spill:{last_used_reg}");
                    spillings.insert(last_used_reg.get_id());
                    colors.remove(&last_used_reg.get_id());
                    last_used.insert(color, reg);
                }
            }
        } else if reg.is_physic() {
            log_file!(easy_ls_path, "{reg}({color}),{last_used:?}");
            let last_use_reg = *last_used.get(&color).unwrap();
            debug_assert!(!last_use_reg.is_physic());
            colors.remove(&last_use_reg.get_id());
            log_file!(easy_ls_path, "spill:{}", last_use_reg.get_id());
            spillings.insert(last_use_reg.get_id());
            last_used.insert(color, reg);
        } else {
            unreachable!();
        }
    }

    ///处理遇到的有色寄存器的逻辑, (fixed类型,强调保持原有的着色)
    pub fn process_one_reg_fixed(
        reg: &Reg,
        live_now: &mut HashSet<Reg>,
        last_used: &mut HashMap<i32, Reg>,
        reg_use_stat: &mut RegUsedStat,
        colors: &mut HashMap<i32, i32>,
        spill_costs: &HashMap<Reg, f32>,
        spillings: &mut HashSet<i32>,
        fixed: &mut HashSet<i32>,
    ) {
        debug_assert!(reg.is_physic() || colors.contains_key(&reg.get_id()));
        if live_now.contains(reg) {
            return;
        }
        let reg = *reg;
        let color = if reg.is_physic() {
            reg.get_color()
        } else {
            *colors.get(&reg.get_id()).unwrap()
        };
        if !last_used.contains_key(&color) {
            // debug_assert!(reg_use_stat.is_available_reg(color), "color:{color}");    //比如zero 就不是available的
            log_file!(easy_ls_path, "init color:{color}");
            reg_use_stat.use_reg(color);
            last_used.insert(color, reg);
        } else if !reg.is_physic() {
            let last_used_reg = last_used.get(&color).unwrap();
            debug_assert!(
                !(fixed.contains(&last_used_reg.get_id()) && fixed.contains(&reg.get_id()))
            );
            if last_used_reg.is_physic() {
                log_file!(easy_ls_path, "spill:{reg}");
                colors.remove(&reg.get_id());
                spillings.insert(reg.get_id());
            } else if fixed.contains(&last_used_reg.get_id()) {
                colors.remove(&reg.get_id());
                spillings.insert(reg.get_id());
            } else if fixed.contains(&reg.get_id()) {
                colors.remove(&last_used_reg.get_id());
                spillings.insert(last_used_reg.get_id());
            } else {
                //否则判断哪个spill代价大,则spill代价小的一个
                let last_spill_cost = spill_costs.get(last_used_reg);
                let cur_spill_cost = spill_costs.get(&reg);
                if last_spill_cost > cur_spill_cost {
                    log_file!(easy_ls_path, "spill:{reg}");
                    colors.remove(&reg.get_id());
                    spillings.insert(reg.get_id());
                } else {
                    log_file!(easy_ls_path, "spill:{last_used_reg}");
                    spillings.insert(last_used_reg.get_id());
                    colors.remove(&last_used_reg.get_id());
                    last_used.insert(color, reg);
                }
            }
        } else if reg.is_physic() {
            log_file!(easy_ls_path, "{reg}({color}),{last_used:?}");
            let last_use_reg = *last_used.get(&color).unwrap();
            debug_assert!(!last_use_reg.is_physic());
            colors.remove(&last_use_reg.get_id());
            spillings.insert(last_use_reg.get_id());
            last_used.insert(color, reg);
        } else {
            unreachable!();
        }
    }

    pub fn color_one(
        reg: &Reg,
        reg_use_stat: &mut RegUsedStat,
        spillings: &mut HashSet<i32>,
        colors: &mut HashMap<i32, i32>,
        last_used: &mut HashMap<i32, Reg>,
    ) -> bool {
        if reg.get_id() == 70 {
            let a = 2;
        }
        debug_assert!(!colors.contains_key(&reg.get_id()));
        debug_assert!(!spillings.contains(&reg.get_id()));
        let color = reg_use_stat.get_available_reg(reg.get_type());
        if color.is_none() {
            spillings.insert(reg.get_id());
            return false;
        } else {
            let color = color.unwrap();
            log_file!(easy_ls_path, "color:{}({})", reg, color);
            colors.insert(reg.get_id(), color);
            reg_use_stat.use_reg(color);
            last_used.insert(color, *reg);
            return true;
        }
    }
}
