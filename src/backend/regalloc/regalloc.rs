use std::collections::{HashMap, HashSet, LinkedList, VecDeque};
use std::hash::Hash;
use std::num;
use std::path::Iter;

use crate::backend::func::Func;
use crate::backend::instrs::{InstrsType, BB};
use crate::backend::operand::Reg;
use crate::backend::regalloc::structs::FuncAllocStat;
use crate::container::bitmap::Bitmap;
use crate::log_file;
use crate::utility::{ObjPtr, ScalarType};

use super::structs::RegUsedStat;

// 该处理下，全局量被翻译到内存中，
// 以函数为寄存器分配的基本单位
pub trait Regalloc {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat;
}

pub fn countStackSize(
    func: &Func,
    spillings: &HashSet<i32>,
) -> (usize, HashMap<ObjPtr<BB>, usize>) {
    // 遍历所有块,找到每个块中的spillings大小,返回其中大小的最大值,
    let mut stackSize: usize = 0;
    let mut bb_stack_sizes: HashMap<ObjPtr<BB>, usize> = HashMap::new();
    let mut passed: HashSet<ObjPtr<BB>> = HashSet::new();
    let mut walk: VecDeque<ObjPtr<BB>> = VecDeque::new();
    walk.push_back(func.entry.unwrap().clone());
    passed.insert(func.entry.unwrap());
    // TOTEST
    while !walk.is_empty() {
        let cur = walk.pop_front().unwrap();
        let mut bbspillings: HashSet<i32> = HashSet::new();
        //println!("{}",cur.label);
        for reg in &cur.as_ref().live_in {
            if spillings.contains(&reg.get_id()) {
                bbspillings.insert(reg.get_id());
            }
        }
        let start = bbspillings.len() * 8;
        bb_stack_sizes.insert(cur, start);
        bbspillings.clear();
        // 统计spilling数量
        for inst in &cur.as_ref().insts {
            for reg in inst.as_ref().get_reg_def() {
                if spillings.contains(&reg.get_id()) {
                    bbspillings.insert(reg.get_id());
                }
            }
            for reg in inst.as_ref().get_reg_use() {
                if spillings.contains(&reg.get_id()) {
                    bbspillings.insert(reg.get_id());
                }
            }
        }
        if bbspillings.len() * 8 + start > stackSize {
            stackSize = bbspillings.len() * 8 + start;
        }
        // 扩展未扩展的节点
        for bb in &cur.as_ref().out_edge {
            if passed.contains(&bb) {
                continue;
            }
            passed.insert(bb.clone());
            walk.push_back(bb.clone());
        }
    }
    (spillings.len() * 8, bb_stack_sizes)
}

// 估计某个寄存器spill可能造成的冲突代价
// 它作为某个指令的def的时候冲突代价为2
// 作为某个指令的def以及use的时候冲突代价为2
// 只作为某个指令的use的时候冲突代价为1
pub fn estimate_spill_cost(func: &Func) -> HashMap<Reg, f32> {
    let mut out: HashMap<Reg, f32> = HashMap::new();
    // 选择一个合适的参数
    let (use_coe, def_coe, def_use_coe): (f32, f32, f32) = (1.0, 1.0, 1.0);
    let (use_cost, def_cost, def_use_cost) = (use_coe * 3.0, def_coe * 3.0, def_use_coe * 4.0);
    //
    for bb in func.blocks.iter() {
        let mut has_defined: HashSet<Reg> = HashSet::new();
        for inst in bb.insts.iter() {
            // FIXME,使用跟精确的统计方法，针对具体指令类型
            let mut in_use: HashSet<Reg> = HashSet::new();
            let mut in_def: HashSet<Reg> = HashSet::new();
            let mut regs: HashSet<Reg> = HashSet::with_capacity(3);
            for reg in inst.get_reg_use() {
                in_use.insert(reg);
                regs.insert(reg);
            }
            for reg in inst.get_reg_def() {
                in_def.insert(reg);
                regs.insert(reg);
            }
            for reg in regs {
                if in_use.contains(&reg) && in_def.contains(&reg) {
                    out.insert(reg, out.get(&reg).unwrap_or(&0.0) + def_use_cost);
                } else if in_use.contains(&reg) {
                    out.insert(reg, out.get(&reg).unwrap_or(&0.0) + use_cost);
                } else {
                    out.insert(reg, out.get(&reg).unwrap_or(&0.0) + def_cost);
                }
            }
        }
    }

    out
}

// 获取冲突表
pub fn build_interference(
    func: &Func,
    ends_index_bb: &HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
) -> HashMap<Reg, HashSet<Reg>> {
    // todo,修改逻辑，以能够处理多定义的情况
    let mut interference_graph: HashMap<Reg, HashSet<Reg>> = HashMap::new();
    let tmp_set = HashSet::new();
    let process =
        |cur_bb: ObjPtr<BB>, interef_graph: &mut HashMap<Reg, HashSet<Reg>>, kind: ScalarType| {
            let mut livenow: HashSet<Reg> = HashSet::new();
            // 冲突分析
            cur_bb.live_in.iter().for_each(|reg| {
                if reg.get_type() != kind {
                    return;
                }
                if let None = interef_graph.get(reg) {
                    interef_graph.insert(*reg, HashSet::new());
                }
                for live in livenow.iter() {
                    if live == reg {
                        continue;
                    }
                    interef_graph.get_mut(live).unwrap().insert(*reg);
                    interef_graph.get_mut(reg).unwrap().insert(*live);
                }
                livenow.insert(*reg);
            });
            for (index, inst) in cur_bb.insts.iter().enumerate() {
                // 先与reg use冲突,然后消去终结的,然后与reg def冲突,并加上新的reg def
                let finishes = ends_index_bb
                    .get(&(index as i32, cur_bb))
                    .unwrap_or(&tmp_set);
                for finish in finishes {
                    livenow.remove(finish);
                }
                for reg in inst.get_reg_def() {
                    if reg.get_type() != kind {
                        continue;
                    }
                    if let None = interef_graph.get(&reg) {
                        interef_graph.insert(reg, HashSet::new());
                    }
                    for live in livenow.iter() {
                        log_file!("tmp2.txt", "pre {} {}.", live, reg);
                        if *live == reg {
                            continue;
                        }
                        log_file!("tmp2.txt", "suf {} {}.", live, reg);
                        interef_graph.get_mut(live).unwrap().insert(reg);
                        interef_graph.get_mut(&reg).unwrap().insert(*live);
                    }
                    if finishes.contains(&reg) {
                        continue;
                    } //fixme,修复增加这里的bug
                    livenow.insert(reg);
                }
            }
        };
    // 遍历所有块，分析冲突关系
    for cur_bb in func.blocks.iter() {
        let cur_bb = *cur_bb;
        // 把不同类型寄存器统计加入表中
        //分别处理浮点寄存器的情况和通用寄存器的情况
        process(cur_bb, &mut interference_graph, ScalarType::Float);
        process(cur_bb, &mut interference_graph, ScalarType::Int);
        // 加入还没有处理过的bb
    }
    interference_graph
}

pub fn build_interference_into_lst(
    func: &Func,
    ends_index_bb: &HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
) -> HashMap<Reg, LinkedList<Reg>> {
    let mut interference_graph: HashMap<Reg, LinkedList<Reg>> = HashMap::new();
    let mut if_passed: HashSet<(i32, i32)> = HashSet::new();
    let tmp_set = HashSet::new();
    let mut process = |cur_bb: ObjPtr<BB>,
                       interef_graph: &mut HashMap<Reg, LinkedList<Reg>>,
                       kind: ScalarType| {
        let mut livenow: HashSet<Reg> = HashSet::new();
        // 冲突分析
        cur_bb.live_in.iter().for_each(|reg| {
            if reg.get_type() != kind {
                return;
            }
            if let None = interef_graph.get(reg) {
                interef_graph.insert(*reg, LinkedList::new());
            }
            for live in livenow.iter() {
                // log_file!("tmp2.txt", "pre {} {}.", live, reg);
                if live == reg {
                    continue;
                }
                // log_file!("tmp2.txt", "suf {} {}.", live, reg);
                // TODO:检查避免重复加入是否成功
                if if_passed.contains(&(live.bit_code(), reg.bit_code()))
                    || if_passed.contains(&(reg.bit_code(), live.bit_code()))
                {
                    continue;
                }
                if_passed.insert((live.bit_code(), reg.bit_code()));
                if_passed.insert((reg.bit_code(), live.bit_code()));
                interef_graph.get_mut(live).unwrap().push_back(*reg);
                interef_graph.get_mut(reg).unwrap().push_back(*live);
            }
            livenow.insert(*reg);
        });
        for (index, inst) in cur_bb.insts.iter().enumerate() {
            // 先与reg use冲突,然后消去终结的,然后与reg def冲突,并加上新的reg def
            let finishes = ends_index_bb
                .get(&(index as i32, cur_bb))
                .unwrap_or(&tmp_set);
            for finish in finishes {
                livenow.remove(finish);
            }
            for reg in inst.get_reg_def() {
                if reg.get_type() != kind {
                    continue;
                }
                if let None = interef_graph.get(&reg) {
                    interef_graph.insert(reg, LinkedList::new());
                }
                for live in livenow.iter() {
                    // log_file!("tmp2.txt", "pre {} {}.", live, reg);
                    if *live == reg {
                        continue;
                    }
                    // log_file!("tmp2.txt", "suf {} {}.", live, reg);
                    // TOTO :检查避免重复加入是否成功
                    if if_passed.contains(&(live.bit_code(), reg.bit_code()))
                        || if_passed.contains(&(reg.bit_code(), live.bit_code()))
                    {
                        continue;
                    }
                    if_passed.insert((live.bit_code(), reg.bit_code()));
                    if_passed.insert((reg.bit_code(), live.bit_code()));
                    interef_graph.get_mut(live).unwrap().push_back(reg);
                    interef_graph.get_mut(&reg).unwrap().push_back(*live);
                }
                if finishes.contains(&reg) {
                    continue;
                } //fixme,修复增加这里的bug
                livenow.insert(reg);
            }
        }
    };
    // 遍历所有块，分析冲突关系
    for cur_bb in func.blocks.iter() {
        let cur_bb = *cur_bb;
        // 把不同类型寄存器统计加入表中
        //分别处理浮点寄存器的情况和通用寄存器的情况
        process(cur_bb, &mut interference_graph, ScalarType::Float);
        process(cur_bb, &mut interference_graph, ScalarType::Int);
        // 加入还没有处理过的bb
    }
    interference_graph
}

// 获取可分配表
pub fn build_availables(
    func: &Func,
    ends_index_bb: &HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
) -> HashMap<Reg, RegUsedStat> {
    let mut availables: HashMap<Reg, RegUsedStat> = HashMap::new();
    let process =
        |cur_bb: ObjPtr<BB>, availables: &mut HashMap<Reg, RegUsedStat>, kind: ScalarType| {
            let mut livenow: HashSet<Reg> = HashSet::new();
            // 冲突分析
            cur_bb.live_in.iter().for_each(|reg| {
                if reg.get_type() != kind {
                    return;
                }
                if let None = availables.get(reg) {
                    availables.insert(*reg, RegUsedStat::new());
                }
                if reg.is_physic() {
                    livenow.iter().for_each(|live| {
                        availables.get_mut(live).unwrap().use_reg(reg.get_color());
                    });
                }
                for live in livenow.iter() {
                    if live.is_physic() {
                        availables.get_mut(reg).unwrap().use_reg(live.get_color());
                    }
                }
                livenow.insert(*reg);
            });
            for (index, inst) in cur_bb.insts.iter().enumerate() {
                // 先与reg use冲突,然后消去终结的,然后与reg def冲突,并加上新的reg def
                if let Some(finishes) = ends_index_bb.get(&(index as i32, cur_bb)) {
                    for finish in finishes {
                        livenow.remove(finish);
                    }
                }
                for reg in inst.get_reg_def() {
                    if reg.get_type() != kind {
                        continue;
                    }
                    if let None = availables.get(&reg) {
                        availables.insert(reg, RegUsedStat::new());
                    }
                    if reg.is_physic() {
                        livenow.iter().for_each(|live| {
                            availables.get_mut(&live).unwrap().use_reg(reg.get_color());
                        });
                    }
                    for live in livenow.iter() {
                        if *live == reg {
                            continue;
                        }
                        if live.is_physic() {
                            availables.get_mut(&reg).unwrap().use_reg(live.get_color());
                        }
                    }

                    livenow.insert(reg);
                }
            }
        };
    // 遍历所有块，分析冲突关系
    for cur_bb in func.blocks.iter() {
        let cur_bb = *cur_bb;
        // 把不同类型寄存器统计加入表中
        //分别处理浮点寄存器的情况和通用寄存器的情况
        process(cur_bb, &mut availables, ScalarType::Float);
        process(cur_bb, &mut availables, ScalarType::Int);
        // 加入还没有处理过的bb
    }
    return availables;
}

pub fn build_nums_neighbor_color(
    func: &Func,
    ends_index_bb: &HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
) -> HashMap<Reg, HashMap<i32, i32>> {
    let mut nums_neighbor_color = HashMap::new();
    let process_one = |livenow: &HashSet<Reg>,
                       nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
                       reg: Reg,
                       kind: ScalarType| {
        log_file!("tmp.txt", "{reg}");
        if reg.get_type() != kind {
            return;
        }
        if reg.is_physic() {
            // 跟live now内部有冲突
            let color = reg.get_color();
            for live in livenow.iter() {
                let nnc = nums_neighbor_color.get_mut(live);
                if nnc.is_none() {
                    panic!("gg{}", live);
                }
                let nnc = nnc.unwrap();
                nnc.insert(color, nnc.get(&color).unwrap_or(&0) + 1);
            }
        }
        if !nums_neighbor_color.contains_key(&reg) {
            nums_neighbor_color.insert(reg, HashMap::with_capacity(32));
        }
        let nnc = nums_neighbor_color.get_mut(&reg).unwrap();
        for live in livenow.iter() {
            if live.is_physic() {
                let live_color = live.get_color();
                nnc.insert(live_color, nnc.get(&live_color).unwrap_or(&0) + 1);
            }
        }
    };

    let process = |cur_bb: ObjPtr<BB>,
                   nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
                   kind: ScalarType| {
        let mut livenow: HashSet<Reg> = HashSet::new();
        let tmp_set = HashSet::new();
        log_file!("tmp.txt", "{},kind:{:?}", cur_bb.label, kind);
        // 冲突分析
        cur_bb.live_in.iter().for_each(|reg| {
            process_one(&mut livenow, nums_neighbor_color, *reg, kind);
            if reg.get_type() == kind {
                livenow.insert(*reg);
            }
        });
        for (index, inst) in cur_bb.insts.iter().enumerate() {
            // 先与reg use冲突,然后消去终结的,然后与reg def冲突,并加上新的reg def
            let finishes = ends_index_bb
                .get(&(index as i32, cur_bb))
                .unwrap_or(&tmp_set);
            for finish in finishes {
                livenow.remove(finish);
            }
            for reg in inst.get_reg_def() {
                process_one(&mut livenow, nums_neighbor_color, reg, kind);
                if finishes.contains(&reg) || reg.get_type() != kind {
                    continue;
                }
                livenow.insert(reg);
            }
        }
    };
    // 遍历所有块，分析冲突关系
    for cur_bb in func.blocks.iter() {
        let cur_bb = *cur_bb;
        // 把不同类型寄存器统计加入表中
        //分别处理浮点寄存器的情况和通用寄存器的情况
        process(cur_bb, &mut nums_neighbor_color, ScalarType::Float);
        process(cur_bb, &mut nums_neighbor_color, ScalarType::Int);
        // 加入还没有处理过的bb
    }
    nums_neighbor_color
}

// 获取 （下标,块)->失效寄存器集合  表
pub fn build_ends_index_bb(func: &Func) -> HashMap<(i32, ObjPtr<BB>), HashSet<Reg>> {
    // TODO 更换新的build ends
    // let a = build_ends_index_bb_old(func);
    let b = build_ends_index_bb_new(func);
    // log_file!("ends.txt", "func:{}", func.label.to_owned());
    // log_file!("ends.txt", "old:");
    // a.iter().for_each(|((index, bb), sets)| {
    //     sets.iter().for_each(|reg| {
    //         log_file!("ends.txt", "{},{},{}", index, bb.label, reg);
    //     });
    // });
    // log_file!("ends.txt", "new:");
    // b.iter().for_each(|((index, bb), sets)| {
    //     sets.iter().for_each(|reg| {
    //         log_file!("ends.txt", "{},{},{}", index, bb.label, reg);
    //     });
    // });
    // log_file!("ends.txt");
    return b;
    // return a;
}
// todo,进行替换尝试，更精确地分析
fn build_ends_index_bb_old(func: &Func) -> HashMap<(i32, ObjPtr<BB>), HashSet<Reg>> {
    // 获取reg
    let mut out: HashMap<(i32, ObjPtr<BB>), HashSet<Reg>> = HashMap::new();
    let mut passed_regs: HashSet<Reg> = HashSet::new();
    for bb in func.blocks.iter() {
        for (index, inst) in bb.insts.iter().enumerate().rev() {
            if let None = out.get(&(index as i32, *bb)) {
                out.insert((index as i32, *bb), HashSet::new());
            }
            for reg in inst.get_regs() {
                if bb.live_out.contains(&reg) {
                    continue;
                }
                if passed_regs.contains(&reg) {
                    continue;
                }
                passed_regs.insert(reg);
                out.get_mut(&(index as i32, *bb)).unwrap().insert(reg);
            }
        }
    }
    out
}

fn build_ends_index_bb_new(func: &Func) -> HashMap<(i32, ObjPtr<BB>), HashSet<Reg>> {
    // 获取reg
    let mut out: HashMap<(i32, ObjPtr<BB>), HashSet<Reg>> = HashMap::new();
    for bb in func.blocks.iter() {
        let mut livenow: HashSet<Reg> = HashSet::new();
        bb.live_out.iter().for_each(|reg| {
            livenow.insert(*reg);
        });
        for (index, inst) in bb.insts.iter().enumerate().rev() {
            if let None = out.get(&(index as i32, *bb)) {
                out.insert((index as i32, *bb), HashSet::new());
            }
            for reg in inst.get_regs() {
                if livenow.contains(&reg) {
                    continue;
                }
                out.get_mut(&(index as i32, *bb)).unwrap().insert(reg);
                livenow.insert(reg);
            }
            for reg in inst.get_reg_def() {
                livenow.remove(&reg);
            }
        }
    }
    out
}

//
pub fn merge_alloc_total(
    func: &Func,
    dstr: &mut HashMap<i32, i32>,
    spillings: &mut HashSet<i32>,
    ends_index_bb: &HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
    nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
    availables: &mut HashMap<Reg, RegUsedStat>,
    spill_cost: &HashMap<Reg, f32>,
    interference_graph: &HashMap<Reg, HashSet<Reg>>,
) -> bool {
    //
    let mut color_regs: HashMap<i32, Bitmap> = HashMap::new();
    let mut map: HashMap<(Reg, i32), i32> = HashMap::new();

    false //合并结束不可能存在新的合并了
}

// 通用寄存器合并
pub fn merge_alloc(
    func: &Func,
    dstr: &mut HashMap<i32, i32>,
    spillings: &mut HashSet<i32>,
    ends_index_bb: &HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
    nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
    availables: &mut HashMap<Reg, RegUsedStat>,
    spill_cost: &HashMap<Reg, f32>,
    interference_graph: &HashMap<Reg, HashSet<Reg>>,
) -> bool {
    // 合并条件,如果一个mv x55 x66指令， 后面 x66指令不再使用了,
    // 则x55(color1),x66(color2)可以进行合并，

    let tmp_set = HashSet::new();
    // 计算价值函数,统计一个指令的价值
    let mut base_count_merge_val = |reg: &Reg,
                                    dstr: &HashMap<i32, i32>,
                                    other_color: i32,
                                    spillings: &HashSet<i32>,
                                    nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
                                    interference_graph: &HashMap<Reg, HashSet<Reg>>|
     -> f32 {
        let out = interference_graph.get(reg).unwrap_or(&tmp_set).len();
        out as f32
    };

    let mut only_virtual_merge_val = |reg: &Reg,
                                      dstr: &HashMap<i32, i32>,
                                      other_color: i32,
                                      spillings: &HashSet<i32>,
                                      nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
                                      interference_graph: &HashMap<Reg, HashSet<Reg>>|
     -> f32 {
        let mut out = 0;
        interference_graph
            .get(&reg)
            .unwrap_or(&tmp_set)
            .iter()
            .for_each(|reg| {
                if reg.is_physic() {
                    return;
                }
                out += 1;
            });
        out as f32
    };
    let mut real_affect_merge_val0 = |reg: &Reg,
                                      dstr: &HashMap<i32, i32>,
                                      other_color: i32,
                                      spillings: &HashSet<i32>,
                                      nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
                                      interference_graph: &HashMap<Reg, HashSet<Reg>>|
     -> f32 {
        let n = interference_graph.get(reg).unwrap_or(&tmp_set).len() as f32;
        let self_color = dstr.get(&reg.get_id()).unwrap();
        let mut out = 1.0;
        if spillings.contains(&reg.get_id()) {
            out += spill_cost.get(reg).unwrap();
        }
        interference_graph
            .get(reg)
            .unwrap_or(&tmp_set)
            .iter()
            .for_each(|reg| {
                if reg.is_physic() {
                    return;
                }

                let nums_neighbor_color = nums_neighbor_color.get(reg).unwrap();
                if spillings.contains(&reg.get_id()) {
                    let num = nums_neighbor_color.get(&other_color).unwrap_or(&0);
                    if *num == 0 {
                        out -= 1 as f32 * spill_cost.get(reg).unwrap();
                    } else {
                        // out -= 1 as f32
                        //     / (*num as f32 + 1.0)
                        //     / (*num as f32 + 1.0)
                        //     / (*num as f32 + 1.0)
                        //     / (*num as f32 + 1.0)
                        //     * spill_cost.get(reg).unwrap();
                    }
                    let num = nums_neighbor_color.get(&self_color).unwrap_or(&0);
                    if *num == 1 {
                        out += 1 as f32 * spill_cost.get(reg).unwrap();
                    } else if *num > 1 {
                        // out += 1 as f32
                        //     / (*num as f32)
                        //     / (*num as f32)
                        //     / (*num as f32)
                        //     / (*num as f32)
                        //     * spill_cost.get(reg).unwrap();
                    }
                } else {
                }
            });
        out
    };

    // 统计每个寄存器辅助减少的指令数
    let mut vals: HashMap<Reg, i32> = HashMap::new();
    let mut m: LinkedList<i32> = LinkedList::new();
    let mut if_merge = false;
    let mut merge = |bb: ObjPtr<BB>,
                     dstr: &mut HashMap<i32, i32>,
                     spillings: &mut HashSet<i32>,
                     ends_index_bb: &HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
                     nums_neighbor_color: &mut HashMap<Reg, HashMap<i32, i32>>,
                     availables: &mut HashMap<Reg, RegUsedStat>,
                     interference_graph: &HashMap<Reg, HashSet<Reg>>| {
        // 首先定位到可能出现merge的指令，比如mv
        for (index, inst) in bb.insts.iter().enumerate() {
            if inst.get_type() != InstrsType::OpReg(crate::backend::instrs::SingleOp::Mv) {
                continue;
            }
            let dst_reg = *inst.get_reg_def().get(0).unwrap();
            let src_reg = *inst.get_reg_use().get(0).unwrap();
            if dst_reg == src_reg {
                continue;
            }
            // 不处理特殊寄存器的合并
            if dst_reg.is_special() || src_reg.is_special() {
                // TODO,处理特殊寄存器的合并
                continue;
            }
            //不处理物理寄存器的寄存器合并
            if dst_reg.is_physic() && src_reg.is_physic() {
                //TODO,暂时不考虑物理寄存器的合并，分配器不应该修改函数或者指令的内容
                continue;
            }
            //不处理两个spilling的寄存器合并且 // 不处理两个相同颜色的寄存器合并
            if dst_reg.is_virtual() && src_reg.is_virtual() {
                if spillings.contains(&dst_reg.get_id()) && spillings.contains(&src_reg.get_id()) {
                    continue;
                }
                if dstr.contains_key(&src_reg.get_id())
                    && dstr.contains_key(&dst_reg.get_id())
                    && dstr.get(&src_reg.get_id()).unwrap() == dstr.get(&dst_reg.get_id()).unwrap()
                {
                    // 不处理同色寄存器合并
                    continue;
                }
            }

            if interference_graph
                .get(&dst_reg)
                .unwrap_or(&tmp_set)
                .contains(&src_reg)
            {
                continue;
            }

            let available_src = availables.get(&src_reg).unwrap();
            let available_dst = availables.get(&dst_reg).unwrap();
            let mut tomerge: Option<Reg> = None;
            let mut merge_color: Option<i32> = None;

            if dst_reg.is_physic() {
                if available_src.is_available_reg(dst_reg.get_color()) {
                    if spillings.contains(&src_reg.get_id()) {
                        tomerge = Some(src_reg);
                        merge_color = Some(dst_reg.get_color());
                    } else {
                        // TODO,分析这里的情况是否需要合并
                    }
                }
            } else if src_reg.is_physic() {
                if available_dst.is_available_reg(src_reg.get_color()) {
                    if spillings.contains(&dst_reg.get_id()) {
                        tomerge = Some(dst_reg);
                        merge_color = Some(src_reg.get_color());
                    } else {
                        // TODO,分析这里的情况是否需要合并
                    }
                }
            } else {
                // 必然其中一个有颜色或者两个有颜色
                let count_merge_val = base_count_merge_val;
                let mut base_inter: f32 = 0.0;
                if dstr.contains_key(&src_reg.get_id())
                    && available_dst.is_available_reg(*dstr.get(&src_reg.get_id()).unwrap())
                {
                    let src_color = dstr.get(&src_reg.get_id()).unwrap();
                    let num_inter_dst = count_merge_val(
                        &dst_reg,
                        &dstr,
                        *src_color,
                        spillings,
                        nums_neighbor_color,
                        interference_graph,
                    );
                    if num_inter_dst > base_inter {
                        base_inter = num_inter_dst;
                        tomerge = Some(dst_reg);
                        merge_color = Some(*dstr.get(&src_reg.get_id()).unwrap());
                    }
                }
                if dstr.contains_key(&dst_reg.get_id())
                    && available_src.is_available_reg(*dstr.get(&dst_reg.get_id()).unwrap())
                {
                    let dst_color = dstr.get(&dst_reg.get_id()).unwrap();
                    let num_inter_src = count_merge_val(
                        &src_reg,
                        &dstr,
                        *dst_color,
                        spillings,
                        nums_neighbor_color,
                        interference_graph,
                    );
                    if num_inter_src > base_inter {
                        tomerge = Some(src_reg);
                        merge_color = Some(*dstr.get(&dst_reg.get_id()).unwrap());
                    }
                }
            }

            // TODO,选择好合并对象之后进行合并
            if let Some(reg) = tomerge {
                if_merge = true;
                spillings.remove(&reg.get_id());
                // log_file!("color_spill.txt","inst:{:?},src:{},dst:{}",inst.get_type(),src_reg,dst_reg);
                // log_file!("color_spill.txt","availables:\nsrc:{}\ndst{}",available_src,available_dst);
                // log_file!("color_spill.txt","merge:{}({}) index:{},bb:{}",reg,merge_color.unwrap(),index,bb.label.clone());
                let merge_color = merge_color.unwrap();
                let neighbors = interference_graph.get(&reg).unwrap();
                if dstr.contains_key(&reg.get_id()) {
                    let old_color = *dstr.get(&reg.get_id()).unwrap();
                    let old_color = &old_color;
                    for neighbor in neighbors {
                        let nums_neighbor_color = nums_neighbor_color.get_mut(neighbor).unwrap();
                        let new_num = nums_neighbor_color.get(old_color).unwrap_or(&1) - 1;
                        nums_neighbor_color.insert(*old_color, new_num);
                        if new_num == 0 {
                            availables
                                .get_mut(neighbor)
                                .unwrap()
                                .release_reg(*old_color);
                            if spillings.contains(&neighbor.get_id()) {
                                spillings.remove(&neighbor.get_id());
                                dstr.insert(neighbor.get_id(), *old_color);
                            }
                        }
                    }
                }
                dstr.insert(reg.get_id(), merge_color);
                for neighbor in neighbors {
                    let nums_neighbor_color = nums_neighbor_color.get_mut(neighbor).unwrap();
                    nums_neighbor_color.insert(
                        merge_color,
                        nums_neighbor_color.get(&merge_color).unwrap_or(&0) + 1,
                    );
                    availables.get_mut(neighbor).unwrap().use_reg(merge_color);
                }
            }
        }
    };
    // 根据冲突结果进行寄存器合并
    for block in func.blocks.iter() {
        merge(
            *block,
            dstr,
            spillings,
            ends_index_bb,
            nums_neighbor_color,
            availables,
            interference_graph,
        );
    }
    if_merge
}

// 判断

// 通用寄存器分配结果检查,判断是否仍然存在冲突情况,若存在,返回冲突的寄存器集合以及所在的指令编号，块标识符)
// (old_reg,cur_reg,inst index,block label)
pub fn check_alloc(
    func: &Func,
    dstr: &HashMap<i32, i32>,
    spillings: &HashSet<i32>,
) -> Vec<(i32, i32, i32, String)> {
    let mut out: Vec<(i32, i32, i32, String)> = Vec::new();
    let ends_index_bb = build_ends_index_bb(func);
    let tmp_set = HashSet::new();
    let mut check_alloc_one =
        |reg: &Reg,
         index: i32,
         bb: ObjPtr<BB>,
         reg_use_stat: &mut RegUsedStat,
         livenow: &mut HashMap<i32, HashSet<i32>>| {
            if spillings.contains(&reg.get_id()) {
                return;
            }
            if reg.is_physic() {
                if reg.get_type() == ScalarType::Float {
                    //fixme
                } else if reg.get_type() == ScalarType::Int {
                    reg_use_stat.use_ireg(reg.get_id())
                }
                return;
            }
            // println!("g?{}", reg.get_id());
            let color = dstr.get(&reg.get_id());
            // fix me
            // if color.is_none() {
            //     out.push((reg.get_id(),-1,index,bb.label.clone()));
            //     return;
            // }

            let color = color.unwrap();
            //
            if !reg_use_stat.is_available_reg(*color) {
                let interef_regs = livenow.get(color).unwrap();
                if interef_regs.contains(&reg.get_id()) {
                    return;
                }
                for interef_reg in interef_regs.iter() {
                    out.push((*interef_reg, reg.get_id(), index, bb.label.clone()));
                }
            }
            reg_use_stat.use_reg(*color);
            livenow.get_mut(color).unwrap().insert(reg.get_id());
        };
    for bb in func.blocks.iter() {
        let mut reg_use_stat = RegUsedStat::new();
        let mut livenow: HashMap<i32, HashSet<i32>> = HashMap::new();
        for i in 0..=63 {
            livenow.insert(i, HashSet::new());
        }

        bb.live_in
            .iter()
            .for_each(|reg| check_alloc_one(reg, -1, *bb, &mut reg_use_stat, &mut livenow));
        for (index, inst) in bb.insts.iter().enumerate() {
            // 先處理生命周期結束的寄存器
            let end_regs = ends_index_bb.get(&(index as i32, *bb)).unwrap_or(&tmp_set);
            for reg in end_regs {
                if spillings.contains(&reg.get_id()) {
                    continue;
                }
                if !reg.is_virtual() {
                    continue;
                }
                // println!("{}", reg.get_id());
                let color = dstr.get(&reg.get_id());
                // if color.is_none() {return  out;}   //FIXME
                let color = color.unwrap();
                livenow.get_mut(color).unwrap().remove(&reg.get_id());
                if livenow.get(color).unwrap().is_empty() {
                    reg_use_stat.release_reg(*color);
                }
            }

            for reg in inst.get_reg_def() {
                check_alloc_one(&reg, index as i32, *bb, &mut reg_use_stat, &mut livenow);
                if end_regs.contains(&reg) {
                    if spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    if !reg.is_virtual() {
                        continue;
                    }
                    let color = dstr.get(&reg.get_id());
                    // if color.is_none() {return  out;}   //FIXME
                    let color = color.unwrap();
                    livenow.get_mut(color).unwrap().remove(&reg.get_id());
                    if livenow.get(color).unwrap().is_empty() {
                        reg_use_stat.release_reg(*color);
                    }
                }
            }
        }
    }
    out
}

// 对分配结果的评估
pub fn eval_alloc(func: &Func, dstr: &HashMap<i32, i32>, spillings: &HashSet<i32>) -> i32 {
    //
    let mut cost: i32 = 0;
    let mut fcost: f32 = 0.0;
    // TODO
    let counts = estimate_spill_cost(func);
    counts.iter().for_each(|(reg, v)| {
        if reg.is_virtual() {
            if spillings.contains(&reg.get_id()) {
                fcost += v;
                // cost+=v;
            }
        }
    });
    cost = fcost as i32;
    cost
}
