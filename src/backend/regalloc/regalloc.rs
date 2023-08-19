use std::collections::{HashMap, HashSet, LinkedList, VecDeque};

use crate::backend::func::Func;
use crate::backend::instrs::{InstrsType, BB};
use crate::backend::operand::Reg;
use crate::backend::regalloc::structs::FuncAllocStat;
use crate::log_file;
use crate::utility::{ObjPtr, ScalarType};
// use crate::{log, log_file};

use super::structs::RegUsedStat;

// 该处理下，全局量被翻译到内存中，
// 以函数为寄存器分配的基本单位
pub trait Regalloc {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat;
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
        let coe: usize = 10;
        let coe = coe.pow(bb.depth as u32) as f32;
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
                    out.insert(reg, out.get(&reg).unwrap_or(&0.0) + def_use_cost * coe);
                } else if in_use.contains(&reg) {
                    out.insert(reg, out.get(&reg).unwrap_or(&0.0) + use_cost * coe);
                } else {
                    out.insert(reg, out.get(&reg).unwrap_or(&0.0) + def_cost * coe);
                }
            }
        }
    }

    out
}

/// 获取冲突表
/// 依赖于 外部调用 calc live计算的 live in,out
pub fn build_interference(func: &Func) -> HashMap<Reg, HashSet<Reg>> {
    // todo,修改逻辑，以能够处理多定义的情况
    let mut interference_graph: HashMap<Reg, HashSet<Reg>> = HashMap::new();
    // let tmp_set = HashSet::new();
    let process =
        |cur_bb: ObjPtr<BB>, interef_graph: &mut HashMap<Reg, HashSet<Reg>>, kind: ScalarType| {
            let mut livenow: HashSet<Reg> = HashSet::new();
            // 冲突分析
            cur_bb.live_out.iter().for_each(|reg| {
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
            for inst in cur_bb.insts.iter().rev() {
                // 先与reg use冲突,然后消去终结的,然后与reg def冲突,并加上新的reg def
                // let finishes = ends_index_bb
                //     .get(&(index as i32, cur_bb))
                //     .unwrap_or(&tmp_set);
                // for finish in finishes {
                //     livenow.remove(finish);
                // }

                for reg in inst.get_reg_def() {
                    livenow.remove(&reg);
                }
                for reg in inst.get_reg_use() {
                    if reg.get_type() != kind {
                        continue;
                    }
                    if let None = interef_graph.get(&reg) {
                        interef_graph.insert(reg, HashSet::new());
                    }
                    for live in livenow.iter() {
                        // log_file!("tmp2.txt", "pre {} {}.", live, reg);
                        if *live == reg {
                            continue;
                        }
                        // log_file!("tmp2.txt", "suf {} {}.", live, reg);
                        interef_graph.get_mut(live).unwrap().insert(reg);
                        interef_graph.get_mut(&reg).unwrap().insert(*live);
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
        process(cur_bb, &mut interference_graph, ScalarType::Float);
        process(cur_bb, &mut interference_graph, ScalarType::Int);
        // 加入还没有处理过的bb
    }
    interference_graph
}

pub fn build_interference_into_lst(func: &Func) -> HashMap<Reg, LinkedList<Reg>> {
    let mut interference_graph_lst: HashMap<Reg, LinkedList<Reg>> = HashMap::new();
    let igg = build_interference(func);
    for (reg, neighbors) in igg.iter() {
        let mut lst = LinkedList::new();
        neighbors.iter().for_each(|reg| lst.push_back(*reg));
        interference_graph_lst.insert(*reg, lst);
    }
    interference_graph_lst
}

// 获取可分配表
pub fn build_availables_with_interef_graph(
    intereference_graph: &HashMap<Reg, HashSet<Reg>>,
) -> HashMap<Reg, RegUsedStat> {
    let mut availables: HashMap<Reg, RegUsedStat> = HashMap::new();
    for (reg, neighbors) in intereference_graph.iter() {
        let mut available = RegUsedStat::init_unspecial_regs();
        for neighbor in neighbors {
            if neighbor.is_physic() {
                available.use_reg(neighbor.get_color());
            }
        }
        availables.insert(*reg, available);
    }
    return availables;
}

pub fn build_nums_neighbor_color(
    _func: &Func,
    interference_graph: &HashMap<Reg, HashSet<Reg>>,
) -> HashMap<Reg, HashMap<i32, i32>> {
    let mut nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>> = HashMap::new();
    for (reg, neighbors) in interference_graph.iter() {
        let mut num_neighbor_color = HashMap::new();
        // 遍历reg周围的颜色
        for neighbor in neighbors.iter() {
            if neighbor.is_physic() {
                let color = neighbor.get_color();
                num_neighbor_color.insert(color, *num_neighbor_color.get(&color).unwrap_or(&0) + 1);
            }
        }
        nums_neighbor_color.insert(*reg, num_neighbor_color);
    }
    nums_neighbor_color
}

///  获取 （下标,块)->失效寄存器集合  表
/// 注意！！！！ 该函数依赖于func的cal live的结果，内部并不会调用func的cal live
pub fn build_ends_index_bb(func: &Func) -> HashMap<(usize, ObjPtr<BB>), HashSet<Reg>> {
    let mut out: HashMap<(usize, ObjPtr<BB>), HashSet<Reg>> = HashMap::new();
    for bb in func.blocks.iter() {
        let mut livenow: HashSet<Reg> = HashSet::new();
        bb.live_out.iter().for_each(|reg| {
            livenow.insert(*reg);
        });
        for (index, inst) in bb.insts.iter().enumerate().rev() {
            if let None = out.get(&(index, *bb)) {
                out.insert((index, *bb), HashSet::new());
            }
            for reg in inst.get_reg_use() {
                if livenow.contains(&reg) {
                    continue;
                }
                out.get_mut(&(index, *bb)).unwrap().insert(reg);
                livenow.insert(reg);
            }
            for reg in inst.get_reg_def() {
                if livenow.contains(&reg) {
                    out.get_mut(&(index, *bb)).unwrap().remove(&reg);
                }
                livenow.remove(&reg);
            }
        }
    }
    out
}

/// 通用寄存器分配结果检查,判断是否仍然存在冲突情况,若存在,返回冲突的寄存器集合以及所在的指令编号，块标识符)
/// (old_reg,cur_reg,inst index,block label)
/// 调用该函数前外部应该对要分析的func调用某种calc live (比如 calc for handle alloc)
pub fn check_alloc(
    func: &Func,
    dstr: &HashMap<i32, i32>,
    spillings: &HashSet<i32>,
) -> Vec<(Reg, Reg, i32, String)> {
    let mut out: Vec<(Reg, Reg, i32, String)> = Vec::new();
    let ends_index_bb = build_ends_index_bb(func);
    let tmp_set = HashSet::new();
    let mut check_alloc_one = |reg: &Reg,
                               index: i32,
                               bb: ObjPtr<BB>,
                               reg_use_stat: &mut RegUsedStat,
                               livenow: &mut HashMap<i32, HashSet<Reg>>|
     -> bool {
        if spillings.contains(&reg.get_id()) {
            return true;
        }
        if reg.is_physic() {
            reg_use_stat.use_reg(reg.get_color());
            livenow.get_mut(&reg.get_color()).unwrap().insert(*reg);
            return true;
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
            // panic!();
            let interef_regs = livenow.get(color).unwrap();
            if interef_regs.contains(&reg) {
                return true;
            }
            for interef_reg in interef_regs.iter() {
                out.push((*interef_reg, *reg, index, bb.label.clone()));
            }
            return false;
        }
        reg_use_stat.use_reg(*color);
        livenow.get_mut(color).unwrap().insert(*reg);
        return true;
    };
    for bb in func.blocks.iter() {
        let mut reg_use_stat = RegUsedStat::new();
        let mut livenow: HashMap<i32, HashSet<Reg>> = HashMap::new();
        for i in 0..=63 {
            livenow.insert(i, HashSet::new());
        }

        let mut if_finish = false;
        bb.live_in.iter().for_each(|reg| {
            if if_finish {
                return;
            }
            if !check_alloc_one(reg, -1, *bb, &mut reg_use_stat, &mut livenow) {
                if_finish = true;
            }
        });
        if if_finish {
            return out;
        }
        for (index, inst) in bb.insts.iter().enumerate() {
            // 先處理生命周期結束的寄存器
            let end_regs = ends_index_bb.get(&(index, *bb)).unwrap_or(&tmp_set);

            for reg in end_regs {
                if spillings.contains(&reg.get_id()) {
                    continue;
                }
                // println!("{}", reg.get_id());
                let color = if reg.is_physic() {
                    reg.get_color()
                } else {
                    *dstr.get(&reg.get_id()).unwrap()
                };
                // if color.is_none() {return  out;}   //FIXME
                livenow.get_mut(&color).unwrap().remove(&reg);
                reg_use_stat.release_reg(color);
            }

            for reg in inst.get_reg_def() {
                check_alloc_one(&reg, index as i32, *bb, &mut reg_use_stat, &mut livenow);
                // if bb.live_out.contains(&reg) {
                //     continue;
                // }
                if spillings.contains(&reg.get_id()) {
                    continue;
                }
                if end_regs.contains(&reg) {
                    let color = if reg.is_physic() {
                        reg.get_color()
                    } else {
                        *dstr.get(&reg.get_id()).unwrap()
                    };
                    livenow.get_mut(&color).unwrap().remove(&reg);
                    if livenow.get(&color).unwrap().len() == 0 {
                        reg_use_stat.release_reg(color);
                    } else {
                        unreachable!();
                    }
                }
            }
        }
    }
    out
}

///检查寄存器分配结果是否正确,(依赖外部的calc live)
pub fn check_alloc_v2(func: &Func, dstr: &HashMap<i32, i32>, _spillings: &HashSet<i32>) {
    for bb in func.blocks.iter() {
        let mut livenow: HashSet<Reg> = HashSet::new();
        let mut live_color: HashSet<i32> = HashSet::new();
        let mut last_use: HashMap<i32, Reg> = HashMap::new();
        bb.live_out.iter().for_each(|reg| {
            if livenow.contains(reg) {
                unreachable!()
            }
            if reg.is_physic() {
                let color = reg.get_color();
                if last_use.contains_key(&color) && live_color.contains(&color) {
                    panic!(
                        "inter:{} ,{} :({})",
                        last_use.get(&color).unwrap(),
                        reg,
                        color
                    );
                }
                last_use.insert(color, *reg);
                live_color.insert(color);
            } else if dstr.contains_key(&reg.get_id()) {
                let color = *dstr.get(&reg.get_id()).unwrap();
                if last_use.contains_key(&color) && live_color.contains(&color) {
                    panic!(
                        "inter:{} ,{} :({})",
                        last_use.get(&color).unwrap(),
                        reg,
                        color
                    );
                }
                live_color.insert(color);
                last_use.insert(color, *reg);
            }
            livenow.insert(*reg);
        });
        for inst in bb.insts.iter().rev() {
            for reg in inst.get_reg_def() {
                livenow.remove(&reg);
                let color = if reg.is_physic() {
                    Some(reg.get_color())
                } else if dstr.contains_key(&reg.get_id()) {
                    Some(*dstr.get(&reg.get_id()).unwrap())
                } else {
                    None
                };
                if color.is_some() {
                    live_color.remove(&color.unwrap());
                }
            }
            for reg in inst.get_reg_use() {
                if livenow.contains(&reg) {
                    continue;
                }
                if reg.is_physic() {
                    let color = reg.get_color();
                    if last_use.contains_key(&color) && live_color.contains(&color) {
                        panic!(
                            "inter:{} ,{} :({})",
                            last_use.get(&color).unwrap(),
                            reg,
                            color
                        );
                    }
                    last_use.insert(color, reg);
                    live_color.insert(color);
                } else if dstr.contains_key(&reg.get_id()) {
                    let color = *dstr.get(&reg.get_id()).unwrap();
                    if last_use.contains_key(&color) && live_color.contains(&color) {
                        panic!(
                            "inter:{} ,{} :({})",
                            last_use.get(&color).unwrap(),
                            reg,
                            color
                        );
                    }
                    live_color.insert(color);
                    last_use.insert(color, reg);
                }
                livenow.insert(reg);
            }
        }
    }
}

// 对分配结果的评估
pub fn eval_alloc(func: &Func, _dstr: &HashMap<i32, i32>, spillings: &HashSet<i32>) -> i32 {
    //
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
    fcost as i32
}
