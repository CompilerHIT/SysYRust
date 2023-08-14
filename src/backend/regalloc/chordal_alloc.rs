use std::collections::{HashMap, HashSet};

use biheap::BiHeap;

use crate::backend::{operand::Reg, regalloc::structs::FuncAllocStat};

use super::structs::RegUsedStat;

pub fn try_best_alloc_with_live_graph_and_availables(
    live_neighbors: &HashMap<Reg, HashSet<Reg>>,
    availables: &HashMap<Reg, RegUsedStat>,
) -> Option<FuncAllocStat> {
    let mut all_neighbors = live_neighbors.clone();
    let mut availables = availables.clone();
    let all_regs = Reg::get_all_regs();
    //所有物理寄存器构成一个团,在弦图分配中可以优先分配不影响可着色性,于是可以把点加入到团中
    //调整权重
    //参考mcs (最大势) 算法 ,势其实是可能邻接的物理/虚拟寄存器造成的影响
    //所以物理寄存器的邻接当作一种虚拟寄存器的邻接，于是给其加上一度,
    let mut mcss: HashMap<Reg, usize> = HashMap::new();
    let mut passed: HashSet<Reg> = HashSet::new();
    let mut orderd_tocolors: Vec<Reg> = Vec::new();
    for (reg, lnbs) in all_neighbors.iter_mut() {
        //加入新节点,进行着色
        let mut available = *availables.get(reg).unwrap();
        let rest_num = available.num_available_regs(reg.get_type());
        let mc = 32 - rest_num;
        mcss.insert(*reg, mc);
    }
    //ps,物理寄存器和浮点寄存器不会邻接,要分别考虑寄存器上限压力
    //记录待着色序列
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
    let mut to_color: BiHeap<ToColorItem> = BiHeap::new();
    while !to_color.is_empty() {
        let item = to_color.pop_max().unwrap();
        let (reg, mc) = (item.0, item.1);
        if passed.contains(&reg) {
            continue;
        }

        debug_assert!(mcss.get(&reg).unwrap() == &mc);
        passed.insert(reg);
        orderd_tocolors.push(reg);
        for nb in live_neighbors.get(&reg).unwrap() {
            if passed.contains(nb) {
                continue;
            }
            let new_mc = mcss.get(nb).unwrap_or(&0) + 1;
            to_color.push(ToColorItem(*nb, new_mc));
        }
    }
    //ordered colors的倒序便是完美消去序列 (/近似完美消去序列)
    let mut colors: HashMap<i32, i32> = HashMap::new();
    let spillings: HashSet<i32> = HashSet::new();
    for tocolor in orderd_tocolors.iter().rev() {
        let available_color = availables
            .get(tocolor)
            .unwrap()
            .get_available_reg(tocolor.get_type());
        if available_color.is_none() {
            return None;
        }
        debug_assert!(!tocolor.is_physic());
        let available_color = available_color.unwrap();
        colors.insert(tocolor.get_id(), available_color);
        for reg in live_neighbors.get(tocolor).unwrap() {
            availables.get_mut(reg).unwrap().use_reg(available_color);
        }
    }
    Some(FuncAllocStat {
        dstr: colors,
        spillings: spillings,
    })
}
