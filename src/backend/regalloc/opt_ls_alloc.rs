// 优化线性寄存器分配
/*
已实现优化:

待实现优化:
1.寄存器合并


 */

use biheap::bivec::order;
use biheap::BiHeap;

use super::regalloc;
use super::structs::BlockAllocStat;
use crate::algorithm::graphalgo;
use crate::algorithm::graphalgo::Graph;
use crate::backend::block::{self, BB};
use crate::backend::func::Func;
use crate::backend::instrs::LIRInst;
use crate::backend::operand::Reg;
use crate::backend::regalloc::regalloc::Regalloc;
use crate::backend::regalloc::structs::{FuncAllocStat, RegUsedStat};
use crate::frontend::ast::Continue;
use crate::utility::ObjPtr;
use crate::utility::ScalarType;
use crate::{log_file, log_file_uln};
use core::panic;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::{write, Display};
use std::hash::Hash;
use std::slice::Windows;

const optls_path: &str = "optls.txt";

// 摆烂的深度优先指令编码简单实现的线性扫描寄存器分配
pub struct Allocator {}

#[derive(Eq, PartialEq)]
struct RegInterval {
    pub reg: Reg,
    pub available: RegUsedStat,
    pub die: i32,
}
impl Display for RegInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.reg, self.die, self.available)
    }
}

impl RegInterval {
    fn new(reg: Reg, die: i32, reg_use_stat: RegUsedStat) -> RegInterval {
        RegInterval {
            reg: reg,
            die: die,
            available: reg_use_stat,
        }
    }
}

impl PartialOrd for RegInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.die.cmp(&other.die))
    }
}
impl Ord for RegInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        // Rust中BinaryHeap的默认实现是大根堆,我们需要的正是大根堆
        self.partial_cmp(other).unwrap()
    }
    fn max(self, other: Self) -> Self {
        let o = self.cmp(&other);
        match o {
            Ordering::Greater => self,
            Ordering::Equal => self,
            Ordering::Less => other,
        }
    }
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {}
    }
    //获取某个顺序的块结构
    pub fn order_blocks(blocks: &Vec<ObjPtr<BB>>) -> Vec<ObjPtr<BB>> {
        //优先传入live in 比较大的
        let mut ordered_blocks: Vec<ObjPtr<BB>> = blocks.iter().cloned().collect();
        ordered_blocks.sort_by_key(|bb| bb.live_in.len());
        ordered_blocks.reverse();
        ordered_blocks
    }

    //对于某个块内进行分配
    fn alloc_block(
        bb: ObjPtr<BB>,
        colors: &mut HashMap<i32, i32>,
        spillings: &mut HashSet<i32>,
        spill_costs: &HashMap<Reg, f32>,
    ) {
        let mut iwindows: BiHeap<RegInterval> = BiHeap::new();
        let mut fwindows: BiHeap<RegInterval> = BiHeap::new();
        let mut ends: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut color_owners: HashMap<i32, Reg> = HashMap::new();

        let mut reg_use_stat = RegUsedStat::new();
        let get_color =
            |reg: &Reg, colors: &HashMap<i32, i32>, spillings: &HashSet<i32>| -> Option<i32> {
                if reg.is_physic() {
                    Some(reg.get_color())
                } else if colors.contains_key(&reg.get_id()) {
                    Some(*colors.get(&reg.get_id()).unwrap())
                } else {
                    None
                }
            };
        let show_windows = |windows: &BiHeap<RegInterval>| {
            log_file_uln!(optls_path, "now windows:{{");
            windows
                .iter()
                .for_each(|r| log_file_uln!(optls_path, "{},", r));
            log_file!(optls_path, "}}");
        };
        ///limit num作为 允许存活的虚拟 寄存器数量
        let add_reg_to_window =
            |reg_interval: RegInterval,
             reg_use_stat: &RegUsedStat,
             iwindows: &mut BiHeap<RegInterval>,
             fwindows: &mut BiHeap<RegInterval>,
             spillings: &mut HashSet<i32>,
             spilling_costs: &HashMap<Reg, f32>| {
                debug_assert!(!spillings.contains(&reg_interval.reg.get_id()));
                let mut reg_interval = reg_interval;
                let kind = reg_interval.reg.get_type();
                let limit_num = reg_use_stat.num_available_regs(kind);
                let windows = &mut match kind {
                    ScalarType::Float => fwindows,
                    ScalarType::Int => iwindows,
                    _ => unreachable!(),
                };
                if windows.len() < limit_num as usize {
                    windows.push(reg_interval);
                    return;
                }
                // unimplemented!("先替换,再轮询");
                // let color = None;
                if windows.len() == 0 {
                    ///说明 limit_num 此时也为0
                    spillings.insert(reg_interval.reg.get_id());
                    return;
                }
                if windows.len() >= limit_num as usize {
                    let max = windows.peek_max().unwrap();
                    let cost_top = spilling_costs.get(&max.reg).unwrap();
                    let cost_cur = spilling_costs.get(&reg_interval.reg).unwrap();
                    if max.die > reg_interval.die
                        || (max.die == reg_interval.die && cost_top < cost_cur)
                    {
                        let reg = windows.pop_max().unwrap().reg;
                        spillings.insert(reg.get_id());
                        windows.push(reg_interval);
                    } else {
                        spillings.insert(reg_interval.reg.get_id());
                    }
                }
                while windows.len() > limit_num {
                    let reg = windows.pop_max().unwrap().reg;
                    spillings.insert(reg.get_id());
                }
            };
        let mut process_color =
            |color: i32,
             die: i32,
             owner: &Reg,
             color_owners: &mut HashMap<i32, Reg>,
             colors: &mut HashMap<i32, i32>,
             spillings: &mut HashSet<i32>,
             reg_use_stat: &mut RegUsedStat,
             ends: &mut HashMap<i32, Vec<i32>>,
             iwindows: &mut BiHeap<RegInterval>,
             fwindows: &mut BiHeap<RegInterval>| {
                log_file!(optls_path, "process color:{}(die at:{})", color, die);
                if color_owners.contains_key(&color) {
                    if !owner.is_physic() {
                        colors.remove(&owner.get_id());
                        spillings.insert(owner.get_id());
                        return;
                    } else {
                        let reg = color_owners.get(&color).unwrap();
                        colors.remove(&reg.get_id());
                        spillings.insert(reg.get_id());
                        color_owners.remove(&color);
                        reg_use_stat.release_reg(color);
                    }
                }
                reg_use_stat.use_reg(color);
                color_owners.insert(color, *owner);
                if !ends.contains_key(&die) {
                    ends.insert(die, vec![color]);
                } else {
                    ends.get_mut(&die).unwrap().push(color);
                }
                let windows = if color <= 31 { iwindows } else { fwindows };
                let mut new_windows = BiHeap::new();
                while !windows.is_empty() {
                    let mut interval = windows.pop_max().unwrap();
                    interval.available.use_reg(color);
                    new_windows.push(interval);
                }
                *windows = new_windows;
            };
        let mut livein_comsume =
            |reg: &Reg,
             reg_use_stat: &mut RegUsedStat,
             iwindows: &mut BiHeap<RegInterval>,
             fwindows: &mut BiHeap<RegInterval>,
             color_owners: &mut HashMap<i32, Reg>,
             colors: &mut HashMap<i32, i32>,
             spillings: &mut HashSet<i32>,
             spill_costs: &HashMap<Reg, f32>| {
                if spillings.contains(&reg.get_id()) {
                    return;
                }
                let reg = *reg;
                let born_key = (reg, -1);
                let (reg, die) = *bb.reg_intervals.get(&born_key).unwrap();
                //给该寄存器分配颜色 (或者该寄存器有颜色)
                let color = get_color(&reg, &colors, &spillings);
                if let Some(color) = color {
                    process_color(
                        color,
                        die,
                        &reg,
                        color_owners,
                        colors,
                        spillings,
                        reg_use_stat,
                        &mut ends,
                        iwindows,
                        fwindows,
                    );
                } else {
                    add_reg_to_window(
                        RegInterval::new(reg, die, *reg_use_stat),
                        &reg_use_stat,
                        iwindows,
                        fwindows,
                        spillings,
                        spill_costs,
                    );
                }
            };

        bb.live_in.iter().for_each(|reg| {
            livein_comsume(
                reg,
                &mut reg_use_stat,
                &mut iwindows,
                &mut fwindows,
                &mut color_owners,
                colors,
                spillings,
                spill_costs,
            );
        });

        let refresh_windows =
            |index: usize, windows: &mut BiHeap<RegInterval>, spillings: &mut HashSet<i32>| {
                while windows.len() > index {
                    let max = windows.pop_max().unwrap().reg;
                    spillings.insert(max.get_id());
                }
            };

        let color_and_release = |index: usize,
                                 windows: &mut BiHeap<RegInterval>,
                                 colors: &mut HashMap<i32, i32>,
                                 spillings: &mut HashSet<i32>| {
            let mut colors_used = RegUsedStat::new();

            while windows.len() > 0 && windows.peek_min().unwrap().die <= index as i32 {
                let min = windows.pop_min().unwrap();
                let mut available = min.available;
                available.merge(&colors_used);
                let reg = min.reg;
                // if reg.get_id() == 71 || reg.get_id() == 66 {
                //     log_file!(optls_path, "{}", colors_used);
                //     log_file!(optls_path, "{}", min.die);
                //     log_file!(optls_path, "{}", windows.len());
                //     windows
                //         .iter()
                //         .for_each(|ri| log_file!(optls_path, "{}", ri.reg));
                //     // log_file!(optls_path, "{:?}", );
                // }
                let color = available.get_available_reg(reg.get_type());
                if color.is_none() {
                    spillings.insert(reg.get_id());
                    continue;
                }
                debug_assert!(color.is_some());
                let color = color.unwrap();
                log_file!(optls_path, "color {} with {}", reg, color);
                colors.insert(reg.get_id(), color);
                colors_used.use_reg(color);
            }
            //对于剩下的寄存器,使用colors_used中的内容对其available进行更新
            let mut new_windows = BiHeap::new();
            while !windows.is_empty() {
                let mut ri = windows.pop_max().unwrap();
                ri.available.merge(&colors_used);
                if !ri.available.is_available(ri.reg.get_type()) {
                    spillings.insert(ri.reg.get_id());
                    continue;
                }
                new_windows.push(ri);
            }
            *windows = new_windows;
        };

        let mut inst_comsume = |inst: ObjPtr<LIRInst>,
                                index: usize,
                                bb: ObjPtr<BB>,
                                reg_use_stat: &mut RegUsedStat,
                                iwindows: &mut BiHeap<RegInterval>,
                                fwindows: &mut BiHeap<RegInterval>,
                                colors: &mut HashMap<i32, i32>,
                                spillings: &mut HashSet<i32>,
                                spill_costs: &HashMap<Reg, f32>| {
            //首先释放能够释放的颜色
            if let Some(colors_released) = ends.get(&(index as i32)) {
                for color in colors_released {
                    log_file!(optls_path, "release color:{}", color);
                    reg_use_stat.release_reg(*color);
                    color_owners.remove(color);
                    log_file!(optls_path, "after release:{}", reg_use_stat);
                }
            }
            refresh_windows(reg_use_stat.num_available_iregs(), iwindows, spillings);
            refresh_windows(reg_use_stat.num_available_fregs(), fwindows, spillings);
            //释放并着色
            color_and_release(index, iwindows, colors, spillings);
            color_and_release(index, fwindows, colors, spillings);
            //对于def中的寄存器
            for reg in inst.get_reg_def() {
                if spillings.contains(&reg.get_id()) {
                    continue;
                }
                let color = get_color(&reg, &colors, &spillings);
                let (reg, die) = *bb.reg_intervals.get(&(reg, index as i32)).unwrap();
                if let Some(color) = color {
                    process_color(
                        color,
                        die,
                        &reg,
                        &mut color_owners,
                        colors,
                        spillings,
                        reg_use_stat,
                        &mut ends,
                        iwindows,
                        fwindows,
                    );
                } else {
                    let new_interval = RegInterval {
                        reg: reg,
                        die: die,
                        available: *reg_use_stat,
                    };
                    log_file!(optls_path, "add {} at {}", new_interval, index);
                    add_reg_to_window(
                        new_interval,
                        &reg_use_stat,
                        iwindows,
                        fwindows,
                        spillings,
                        spill_costs,
                    );
                    if reg.get_type() == ScalarType::Int {
                        show_windows(&iwindows);
                    } else {
                        show_windows(&fwindows);
                    }
                }
            }
        };

        for (index, inst) in bb.insts.iter().enumerate() {
            inst_comsume(
                *inst,
                index,
                bb,
                &mut reg_use_stat,
                &mut iwindows,
                &mut fwindows,
                colors,
                spillings,
                spill_costs,
            );
        }

        //对于 live out内的东西 (因为reg 一定在其他block的 live in里,所以就不会管)
        refresh_windows(reg_use_stat.num_available_iregs(), &mut iwindows, spillings);
        refresh_windows(reg_use_stat.num_available_fregs(), &mut fwindows, spillings);
        color_and_release(bb.insts.len(), &mut iwindows, colors, spillings);
        color_and_release(bb.insts.len(), &mut fwindows, colors, spillings);
        // unimplemented!("live out do");
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat {
        func.calc_live_for_alloc_reg();
        func.build_reg_intervals();
        let mut colors: HashMap<i32, i32> = HashMap::new();
        let mut spillings: HashSet<i32> = HashSet::new();
        let spill_costs = regalloc::estimate_spill_cost(func);
        let blocks = Allocator::order_blocks(&func.blocks);
        debug_assert!(blocks.len() == func.blocks.len());
        //打印intervals
        func.blocks.iter().for_each(|bb| {
            log_file!(
                "intervals.txt",
                "{},{}:\n{:?}",
                func.label,
                bb.label,
                bb.reg_intervals
            );
        });

        for bb in blocks.iter() {
            Allocator::alloc_block(*bb, &mut colors, &mut spillings, &spill_costs);
        }
        spillings.clear();
        for bb in blocks.iter() {
            Allocator::alloc_block(*bb, &mut colors, &mut spillings, &spill_costs);
        }

        //TODO,寄存器分裂
        let (stack_size, bb_stack_sizes) = regalloc::countStackSize(func, &spillings);
        FuncAllocStat {
            stack_size: stack_size,
            bb_stack_sizes: bb_stack_sizes,
            spillings: spillings,
            dstr: colors,
        }
    }
}
