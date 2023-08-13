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

pub fn alloc(func: &Func) -> Option<FuncAllocStat> {
    let intereference_graph = regalloc::build_interference(func);
    let availables = regalloc::build_availables_with_interef_graph(&intereference_graph);
    alloc_with_interef_graph_and_constraints(intereference_graph, availables, &HashMap::new())
}

/// 尝试寻找完美分配方案,如果找到完美分配方案,则返回,否则返回None
/// 该分配方式依赖外部调用的calc live
pub fn alloc_with_constraints(
    func: &Func,
    constraints: &HashMap<Reg, HashSet<Reg>>,
) -> Option<FuncAllocStat> {
    let intereference_graph = regalloc::build_interference(func);
    let availables = regalloc::build_availables_with_interef_graph(&intereference_graph);
    //把interef graph转为活interef graph
    alloc_with_interef_graph_and_constraints(intereference_graph, availables, constraints)
}

pub fn alloc_with_interef_graph_and_constraints(
    intereference_graph: HashMap<Reg, HashSet<Reg>>,
    availables: HashMap<Reg, RegUsedStat>,
    constraints: &HashMap<Reg, HashSet<Reg>>,
) -> Option<FuncAllocStat> {
    let mut all_neighbors = intereference_graph;
    let mut availables = availables;
    // 根据constraints更新
    for (v_reg, constraint) in constraints.iter() {
        debug_assert!(!v_reg.is_physic());
        for p_reg in constraint.iter() {
            debug_assert!(p_reg.is_physic());
            if !all_neighbors.contains_key(p_reg) {
                all_neighbors.insert(*p_reg, HashSet::new());
            }
            // 所有寄存器都起码与特殊寄存器冲突,

            all_neighbors.get_mut(p_reg).unwrap().insert(*v_reg);
            all_neighbors.get_mut(v_reg).unwrap().insert(*p_reg);
            availables
                .get_mut(v_reg)
                .unwrap()
                .use_reg(p_reg.get_color());
        }
    }

    let mut live_neighbors = all_neighbors.clone();

    live_neighbors.retain(|k, _| !k.is_physic());
    for (_, nbs) in live_neighbors.iter_mut() {
        nbs.retain(|reg| !reg.is_physic());
    }

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
    准备最后根据总图,对排好序的寄存器进行分配
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
                if nb.is_physic() {
                    continue;
                }
                availables.get_mut(nb).unwrap().use_reg(color);
            }
        }
        colors
    };
    //应用kempe定义探索最优寄存器分配
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
                let nbs = live_neighbors.remove(to_color).unwrap();
                for nb in nbs.iter() {
                    live_neighbors.get_mut(nb).unwrap().remove(to_color);
                }
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
        let colors = final_color(&all_neighbors, availables, ordered_color_lst);

        let fat = FuncAllocStat {
            spillings: HashSet::new(),
            dstr: colors,
        };
        return Some(fat);
    }

    // 分配失败后,把剩余物理寄存器根据含义,补全为带物理寄存器的冲突图,试探弦图分配
    let FuncAllocStat {
        mut dstr,
        spillings,
    } = chordal_alloc::alloc_with_live_neighbors_and_availables(
        live_neighbors.clone(),
        availables.clone(),
    );
    let mut availables = availables;
    if spillings.len() == 0 {
        // 合并着色结果
        for (r, _) in live_neighbors.iter() {
            let color = dstr.get(&r.get_id()).unwrap();
            for nb in all_neighbors.get(r).unwrap() {
                if nb.is_physic() {
                    continue;
                }
                availables.get_mut(nb).unwrap().use_reg(*color);
            }
        }
        let colors = final_color(&all_neighbors, availables, ordered_color_lst);
        dstr.extend(colors);
        return Some(FuncAllocStat { spillings, dstr });
    }
    None
}
