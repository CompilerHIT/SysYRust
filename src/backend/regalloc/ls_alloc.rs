use std::collections::{HashMap, HashSet, VecDeque};

use biheap::{bivec::order, BiHeap};

use crate::{
    backend::{
        instrs::{Func, LIRInst, BB},
        operand::Reg,
        regalloc::structs::RegUsedStat,
    },
    config,
    container::bitmap::Bitmap,
    utility::{ObjPtr, ScalarType},
};

use super::{structs::FuncAllocStat, *};

// 默认按照end进行排序
#[derive(PartialEq, Eq, Clone)]
struct RegInteval {
    reg: Reg,
    color: i32,
    start: usize,
    end: usize,
}
impl Ord for RegInteval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for RegInteval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.end.partial_cmp(&other.end)
    }
}

///深度优先遍历块建立线性序
pub fn build_linear_order(func: &Func) -> Vec<ObjPtr<LIRInst>> {
    let first_bb = func.get_first_block();
    let mut to_pass: VecDeque<ObjPtr<BB>> = VecDeque::new();
    let mut passed: HashSet<ObjPtr<BB>> = HashSet::new();
    let mut orders: Vec<ObjPtr<LIRInst>> = Vec::new();
    to_pass.push_back(first_bb);
    // 深度优先
    while !to_pass.is_empty() {
        let bb = to_pass.pop_back().unwrap();
        if passed.contains(&bb) {
            continue;
        }
        passed.insert(bb);
        orders.extend(bb.insts.iter());
        for out_bb in bb.out_edge.iter() {
            to_pass.push_back(*out_bb);
        }
    }
    orders
}

/// 时间复杂度O(NMlogN)
pub fn alloc(func: &Func) -> FuncAllocStat {
    config::record_event("start build live out for insts for ls alloc");
    let live_out_for_inst = func.build_live_out_for_insts();
    config::record_event("finish build live out for insts for ls alloc");
    // 建立指令的线性序
    config::record_event("start build linear order for insts");
    let orders = build_linear_order(func);
    config::record_event("finish build linear orders for insts");
    debug_assert!(orders.len() == func.num_insts());
    // 建立寄存器在伪拓扑中的生命周期,并按照生命起点进行排序
    let mut starts: HashMap<Reg, usize> = HashMap::new();
    let mut ends: HashMap<Reg, usize> = HashMap::new();
    let mut reg_starts: Vec<(Reg, usize)> = Vec::new();
    // 建立指令区间
    config::record_event("start build reg interval for ls alloc");
    // 建立starts
    for (index, inst) in orders.iter().enumerate() {
        for reg in inst.get_regs() {
            if !starts.contains_key(&reg) {
                starts.insert(reg, index);
            }
        }
        for reg in live_out_for_inst.get(inst).unwrap() {
            if !starts.contains_key(reg) {
                starts.insert(*reg, index);
            }
        }
    }
    //建立 ends
    for (index, inst) in orders.iter().enumerate().rev() {
        for reg in inst.get_regs() {
            if !ends.contains_key(&reg) {
                ends.insert(reg, index);
            }
        }
        for reg in live_out_for_inst.get(inst).unwrap() {
            if !ends.contains_key(reg) {
                ends.insert(*reg, index);
            }
        }
    }

    config::record_event("finish build reg interval for ls alloc");
    let regs: Vec<Reg> = func.draw_all_regs().iter().cloned().collect();
    let mut unavailables = Reg::get_all_specials_with_s0();
    unavailables.extend(Reg::get_all_tmps());
    alloc_with_start_end_unavailables(&starts, &ends, &regs, &unavailables)
}

// alloc with unavailables
fn alloc_with_start_end_unavailables(
    starts: &HashMap<Reg, usize>,
    ends: &HashMap<Reg, usize>,
    regs: &Vec<Reg>,
    unavailables: &HashSet<Reg>,
) -> FuncAllocStat {
    let mut regs: Vec<Reg> = regs
        .iter()
        .filter(|reg| !unavailables.contains(&reg))
        .cloned()
        .collect();
    config::record_event("start sort reg intervals by start for ls alloc");
    regs.sort_by_cached_key(|reg| starts.get(reg).unwrap());
    config::record_event("finish sort reg intervals by start for ls alloc");
    let mut colors: HashMap<i32, i32> = HashMap::new();
    let mut spillings: HashSet<i32> = HashSet::new();
    let mut iwindows: BiHeap<RegInteval> = BiHeap::new();
    let mut fwindows: BiHeap<RegInteval> = BiHeap::new();
    let mut reg_use_stat = RegUsedStat::init_unspecial_regs_without_s0();
    for reg in unavailables.iter() {
        reg_use_stat.use_reg(reg.get_color());
    }
    // 开始的时候需要根据live in 修改reg_use_stat能够使用的寄存器
    config::record_event("start windows slice for ls alloc");
    for reg in regs.iter() {
        let cur_index = starts.get(reg).unwrap();
        let end_index = ends.get(reg).unwrap();
        // println!("{}:{}-{}", reg, cur_index, end_index);
        // 判断当前是否有终结日期小于cur_index的寄存器,如果有,则可以释放
        let windows = match reg.get_type() {
            ScalarType::Int => &mut iwindows,
            ScalarType::Float => &mut fwindows,
            _ => unreachable!(),
        };
        while !windows.is_empty() {
            let min = windows.peek_min().unwrap().end;
            if &min < cur_index {
                let RegInteval {
                    reg,
                    color,
                    start,
                    end,
                } = windows.pop_min().unwrap();
                if !reg.is_physic() {
                    colors.insert(reg.get_id(), color);
                }
                reg_use_stat.release_reg(color);
            } else {
                break;
            }
        }

        if reg.is_physic() {
            // 否则把里面有该颜色的寄存器给释放
            if reg_use_stat.is_available_reg(reg.get_color()) {
                reg_use_stat.use_reg(reg.get_color());
                windows.push(RegInteval {
                    reg: *reg,
                    color: reg.get_color(),
                    start: *cur_index,
                    end: *end_index,
                });
            } else {
                let mut new_windows: BiHeap<RegInteval> = BiHeap::new();
                for rt in windows.iter() {
                    if rt.color == reg.get_color() {
                        spillings.insert(rt.reg.get_id());
                        continue;
                    }
                    new_windows.push(rt.clone());
                }
                *windows = new_windows;
                windows.push(RegInteval {
                    reg: *reg,
                    color: reg.get_color(),
                    start: *cur_index,
                    end: *end_index,
                });
            }
            continue;
        }

        // 加入新的寄存器,寻找可以着色的颜色
        let color = reg_use_stat.get_available_reg(reg.get_type());
        if let Some(color) = color {
            reg_use_stat.use_reg(color);
            windows.push(RegInteval {
                reg: *reg,
                color,
                start: *cur_index,
                end: *end_index,
            });
        } else {
            // 竞争,找到当前块最大的与之颜色冲突的颜色,
            let mut new_windows: BiHeap<RegInteval> = BiHeap::new();
            while !windows.is_empty() {
                let max = windows.pop_max().unwrap();
                if max.reg.is_physic() {
                    new_windows.push(max);
                    continue;
                }
                if max.end > *end_index {
                    new_windows.push(RegInteval {
                        reg: *reg,
                        color: max.color,
                        start: *cur_index,
                        end: *end_index,
                    });
                    spillings.insert(max.reg.get_id());
                } else {
                    new_windows.push(max);
                    spillings.insert(reg.get_id());
                }
                break;
            }
            for ri in windows.iter() {
                new_windows.push(ri.clone());
            }
            *windows = new_windows;
        }
    }
    for rt in fwindows.iter() {
        if !rt.reg.is_physic() {
            colors.insert(rt.reg.get_id(), rt.color);
        }
    }
    for rt in iwindows.iter() {
        if !rt.reg.is_physic() {
            colors.insert(rt.reg.get_id(), rt.color);
        }
    }
    config::record_event("finish windows slice for ls alloc");
    FuncAllocStat {
        spillings,
        dstr: colors,
    }
}
