use std::collections::{HashMap, HashSet};

use biheap::BiHeap;

use crate::backend::{instrs::Func, operand::Reg, regalloc::structs::FuncAllocStat};

use super::{
    perfect_alloc::fill_live_neighbors_to_all_neighbors_with_availables, structs::RegUsedStat, *,
};

// 弦图分配,依赖外部使用的calc live
pub fn alloc(func: &Func) -> FuncAllocStat {
    let intereference_graph = regalloc::build_interference(func);
    let mut availables = regalloc::build_availables_with_interef_graph(&intereference_graph);
    let ordered_to_color = build_color_order(&intereference_graph);
    let mut colors: HashMap<i32, i32> = HashMap::new();
    let mut spillings: HashSet<i32> = HashSet::new();
    for reg in ordered_to_color {
        debug_assert!(!reg.is_physic());
        let available = availables.get(&reg).unwrap();
        let available = available.get_available_reg(reg.get_type());
        if available.is_none() {
            spillings.insert(reg.get_id());
            continue;
        }
        let available = available.unwrap();
        colors.insert(reg.get_id(), available);
        for nb in intereference_graph.get(&reg).unwrap() {
            availables.get_mut(nb).unwrap().use_reg(available);
        }
    }
    // 根据冲突图获取完美作色序列
    FuncAllocStat {
        spillings,
        dstr: colors,
    }
}

// 根据live_interef_graph以及available进行分配
pub fn alloc_with_live_neighbors_and_availables(
    live_neighbors: HashMap<Reg, HashSet<Reg>>,
    availables: HashMap<Reg, RegUsedStat>,
) -> FuncAllocStat {
    let mut intereference_graph: HashMap<Reg, HashSet<Reg>> = live_neighbors;
    let mut availables = availables;
    //还原成完整的图
    fill_live_neighbors_to_all_neighbors_with_availables(&mut intereference_graph, &availables);
    let ordered_to_color = build_color_order(&intereference_graph);
    let mut colors: HashMap<i32, i32> = HashMap::new();
    let mut spillings: HashSet<i32> = HashSet::new();
    for reg in ordered_to_color {
        debug_assert!(!reg.is_physic());
        let available = availables.get(&reg).unwrap();
        let available = available.get_available_reg(reg.get_type());
        if available.is_none() {
            spillings.insert(reg.get_id());
            continue;
        }
        let available = available.unwrap();
        colors.insert(reg.get_id(), available);
        for nb in intereference_graph.get(&reg).unwrap() {
            if nb.is_physic() {
                continue;
            }
            availables.get_mut(nb).unwrap().use_reg(available);
        }
    }
    // 根据冲突图获取完美作色序列
    FuncAllocStat {
        spillings,
        dstr: colors,
    }
}

// 验证某个着色序列是否是完美消去序列

// 建立完美消去序列
pub fn build_color_order(intereference_graph: &HashMap<Reg, HashSet<Reg>>) -> Vec<Reg> {
    // 从物理寄存器出发,
    let mut intereference_graph = intereference_graph.clone();
    let mut ordered_to_color = Vec::new();
    #[derive(PartialEq, Eq)]
    struct ToColorItem(Reg, usize);
    impl PartialOrd for ToColorItem {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            self.1.partial_cmp(&other.1)
        }
    }
    impl Ord for ToColorItem {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.1.cmp(&other.1)
        }
    }
    let mut reg_weight: HashMap<Reg, usize> = HashMap::new();
    for (reg, _) in intereference_graph.iter() {
        reg_weight.insert(*reg, 0);
    }
    for reg in Reg::get_all_regs() {
        if !intereference_graph.contains_key(&reg) {
            continue;
        }

        let nbs = intereference_graph.remove(&reg).unwrap();
        for nb in nbs.iter() {
            let new_weight = reg_weight.get(nb).unwrap() + 1;
            reg_weight.insert(*nb, new_weight);
            intereference_graph.get_mut(nb).unwrap().remove(&reg);
        }
    }

    let mut rest_regs: BiHeap<ToColorItem> = BiHeap::new();
    for (reg, _) in intereference_graph.iter() {
        rest_regs.push(ToColorItem(*reg, *reg_weight.get(reg).unwrap()));
    }
    // 寻找势力最大的节点进行着色
    while !rest_regs.is_empty() {
        let max = rest_regs.pop_max().unwrap();
        let reg = max.0;
        let weight = max.1;
        if reg_weight.get(&reg).unwrap() != &weight {
            continue;
        }
        ordered_to_color.push(reg);
        let nbs = intereference_graph.remove(&reg).unwrap();
        for nb in nbs.iter() {
            let new_weight = reg_weight.get(nb).unwrap() + 1;
            reg_weight.insert(*nb, new_weight);
            intereference_graph.get_mut(nb).unwrap().remove(&reg);
            rest_regs.push(ToColorItem(*nb, new_weight));
        }
    }
    ordered_to_color
}
