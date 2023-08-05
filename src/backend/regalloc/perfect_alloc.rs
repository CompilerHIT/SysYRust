use std::collections::{HashMap, HashSet, LinkedList};

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
        let mut colors: HashMap<i32, i32> = HashMap::new();
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
        let fat = FuncAllocStat {
            stack_size: 0,
            bb_stack_sizes: HashMap::new(),
            spillings: HashSet::new(),
            dstr: colors,
        };
        return Some(fat);
    } else {
        log_file!("unbest_alloc_for_pp.txt", "{:?}", to_colors);
    }
    None
}
