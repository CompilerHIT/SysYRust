use std::collections::{HashMap, HashSet, LinkedList};

use biheap::bivec::order;

use crate::{
    backend::{instrs::Func, operand::Reg},
    log_file,
};

use super::{
    structs::{FuncAllocStat, RegUsedStat},
    *,
};

/// 尝试寻找完美分配方案,如果找到完美分配方案,则返回,否则返回None
/// 该分配方式依赖外部调用的calc live
pub fn alloc(func: &Func, constraints: &HashMap<Reg, HashSet<Reg>>) -> Option<FuncAllocStat> {
    let mut intereference_graph = regalloc::build_interference(func);
    // intereference_graph.retain(|reg, _| !reg.is_physic());
    let availables = regalloc::build_availables_with_interef_graph(&intereference_graph);
    //把interef graph转为活interef graph
    intereference_graph.retain(|reg, _| !reg.is_physic());
    for (_, inter) in intereference_graph.iter_mut() {
        inter.retain(|reg| !reg.is_physic());
    }
    alloc_with_v_interference_graph_and_base_available(
        &intereference_graph,
        &availables,
        constraints,
    )
}

pub fn alloc_with_v_interference_graph_and_base_available(
    all_live_neighbors: &HashMap<Reg, HashSet<Reg>>,
    availables: &HashMap<Reg, RegUsedStat>,
    constraints: &HashMap<Reg, HashSet<Reg>>,
) -> Option<FuncAllocStat> {
    let all_live_neighbors = all_live_neighbors;
    let mut availables = availables.clone();
    for (reg, constraints) in constraints.iter() {
        debug_assert!(!reg.is_physic());
        if !availables.contains_key(reg) {
            availables.insert(*reg, RegUsedStat::init_unspecial_regs());
        }
        for p_reg in constraints.iter() {
            // debug_assert!(availables.contains_key(reg), "availables:{}", reg);
            availables.get_mut(reg).unwrap().use_reg(p_reg.get_color());
        }
    }
    let mut live_neighbors = all_live_neighbors.clone();

    //检查是否有物理寄存器
    debug_assert!(|| -> bool {
        for (r, nbs) in live_neighbors.iter() {
            if r.is_physic() {
                return false;
            }
            for nb in nbs.iter() {
                if nb.is_physic() {
                    return false;
                }
            }
        }
        return true;
    }());

    /*
    准备最后根据总图进行寄存器分配的方法
     */
    let final_color = |all_neighbors: &HashMap<Reg, HashSet<Reg>>,
                       availables: HashMap<Reg, RegUsedStat>,
                       orderd_tocolors: LinkedList<Reg>|
     -> HashMap<i32, i32> {
        let mut colors: HashMap<i32, i32> = HashMap::new();
        let all_live_neighbors = all_neighbors;
        let mut ordered_color_lst = orderd_tocolors;
        let mut availables = availables;
        while !ordered_color_lst.is_empty() {
            let reg = ordered_color_lst.pop_front().unwrap();
            let color = availables.get(&reg).unwrap();
            let color = color.get_available_reg(reg.get_type()).unwrap();
            colors.insert(reg.get_id(), color);
            for nb in all_live_neighbors.get(&reg).unwrap() {
                availables.get_mut(nb).unwrap().use_reg(color);
            }
        }
        colors
    };

    let mut to_colors: Vec<Reg> = live_neighbors.iter().map(|(key, _)| *key).collect();
    let mut ordered_color_lst: LinkedList<Reg> = LinkedList::new();
    let availables = availables;
    loop {
        let mut finish_flag = true;
        let mut new_to_colors: Vec<Reg> = Vec::new();
        for to_color in to_colors.iter() {
            debug_assert!(availables.contains_key(to_color), "{}", {
                to_color.to_string(true)
            });
            let available = availables
                .get(to_color)
                .unwrap()
                .num_available_regs(to_color.get_type());
            let num_live_neighbors = live_neighbors.get(to_color).unwrap().len();
            if available > num_live_neighbors {
                //加入着色队列
                finish_flag = false;
                ordered_color_lst.push_front(*to_color);
                let nbs = live_neighbors.get(to_color).unwrap().clone();
                for nb in nbs.iter() {
                    live_neighbors.get_mut(nb).unwrap().remove(to_color);
                }
                live_neighbors.remove(to_color);
            } else {
                new_to_colors.push(*to_color);
            }
        }
        to_colors = new_to_colors;
        if finish_flag {
            break;
        }
    }

    if to_colors.len() == 0 {
        let colors = final_color(all_live_neighbors, availables, ordered_color_lst);
        // let mut colors: HashMap<i32, i32> = HashMap::new();
        // let mut availables = availables;
        // while !ordered_color_lst.is_empty() {
        //     let reg = ordered_color_lst.pop_front().unwrap();
        //     let color = availables.get(&reg).unwrap();
        //     let color = color.get_available_reg(reg.get_type()).unwrap();
        //     colors.insert(reg.get_id(), color);
        //     for nb in all_live_neighbors.get(&reg).unwrap() {
        //         availables.get_mut(nb).unwrap().use_reg(color);
        //     }
        // }

        let fat = FuncAllocStat {
            stack_size: 0,
            bb_stack_sizes: HashMap::new(),
            spillings: HashSet::new(),
            dstr: colors,
        };
        return Some(fat);
    }

    // return None;
    //发现一阶段方案不足以完美分配,对于剩下的活寄存器,进一步搜索试探完美分配
    log_file!("unbest_alloc_for_pp.txt", "{:?}", to_colors);
    let mut pre_colors: HashMap<i32, i32> = HashMap::new();
    let mut availables = availables;
    loop {
        let mut finish_flag = true;
        let mut new_to_colors: Vec<Reg> = Vec::new();

        for to_color in to_colors.iter() {
            let mut nb_availables = RegUsedStat::init_unavailable();
            //现在要记录所有邻居都使用到的寄存器
            for nb in live_neighbors.get(to_color).unwrap() {
                let available = availables.get(nb).unwrap();
                nb_availables.inter(available);
            }

            //从中去掉特殊颜色
            let special = RegUsedStat::init_unspecial_regs_without_s0();
            nb_availables.inter(&special);

            //然后在nb中找一个可用的颜色来着色
            if !nb_availables.is_available(to_color.get_type()) {
                new_to_colors.push(*to_color);
                continue;
            }
            //着色,加入表中
            finish_flag = false;
            let available_color = nb_availables
                .get_available_reg(to_color.get_type())
                .unwrap();
            pre_colors.insert(to_color.get_id(), available_color);
            let nbs = live_neighbors.remove(to_color).unwrap();
            for nb in nbs.iter() {
                availables.get_mut(nb).unwrap().use_reg(available_color);
            }
        }
        to_colors = new_to_colors;
        let mut new_to_colors = Vec::new();

        for to_color in to_colors.iter() {
            debug_assert!(availables.contains_key(to_color), "{}", {
                to_color.to_string(true)
            });
            let available = availables
                .get(to_color)
                .unwrap()
                .num_available_regs(to_color.get_type());
            let num_live_neighbors = live_neighbors.get(to_color).unwrap().len();
            if available > num_live_neighbors {
                //加入着色队列
                finish_flag = false;
                ordered_color_lst.push_front(*to_color);
                let nbs = live_neighbors.get(to_color).unwrap().clone();
                for nb in nbs.iter() {
                    live_neighbors.get_mut(nb).unwrap().remove(to_color);
                }
                live_neighbors.remove(to_color);
            } else {
                new_to_colors.push(*to_color);
            }
        }
        to_colors = new_to_colors;
        if finish_flag {
            break;
        }
    }

    if to_colors.len() == 0 {
        let mut colors: HashMap<i32, i32> =
            final_color(all_live_neighbors, availables, ordered_color_lst);
        colors.extend(pre_colors);
        let fat = FuncAllocStat {
            stack_size: 0,
            bb_stack_sizes: HashMap::new(),
            spillings: HashSet::new(),
            dstr: colors,
        };
        return Some(fat);
    }

    //如果二阶段方案不足以完美分配,试探三阶段完美分配  (建立数学模型+多线程发射)

    None
}
