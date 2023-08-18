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

/// 该allco依赖于外部的calc_live，与外部使用的calc_live类型对应
pub fn alloc(func: &Func) -> Option<FuncAllocStat> {
    let intereference_graph = regalloc::build_interference(func);
    let availables = regalloc::build_availables_with_interef_graph(&intereference_graph);
    alloc_with_interef_graph_and_availables_and_constraints(
        intereference_graph,
        availables,
        &HashMap::new(),
    )
}

/// 尝试寻找完美分配方案,如果找到完美分配方案,则返回,否则返回None
/// 该分配方式依赖外部调用的calc live
#[inline]
pub fn alloc_with_constraints(
    func: &Func,
    constraints: &HashMap<Reg, HashSet<Reg>>,
) -> Option<FuncAllocStat> {
    let intereference_graph = regalloc::build_interference(func);
    let availables = regalloc::build_availables_with_interef_graph(&intereference_graph);
    //把interef graph转为活interef graph
    alloc_with_interef_graph_and_availables_and_constraints(
        intereference_graph,
        availables,
        constraints,
    )
}

/// 把约束加入到全图中
/// 目前的实现中加入的约束只是物理寄存器和虚拟寄存器的约束
#[inline]
pub fn add_constraint_to_interference_graph(
    all_neighbors: &mut HashMap<Reg, HashSet<Reg>>,
    availables: &mut HashMap<Reg, RegUsedStat>,
    constraints: &HashMap<Reg, HashSet<Reg>>,
) {
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
}

///建立活邻接图
#[inline]
pub fn build_live_neighbors_from_all_neigbhors(
    all_neighbors: &HashMap<Reg, HashSet<Reg>>,
) -> HashMap<Reg, HashSet<Reg>> {
    let mut live_neighbors: HashMap<Reg, HashSet<Reg>> = all_neighbors.clone();
    live_neighbors.retain(|reg, _| !reg.is_physic());
    for (_, lns) in live_neighbors.iter_mut() {
        lns.retain(|reg| !reg.is_physic());
    }
    check_if_full_neighbors(&live_neighbors);
    live_neighbors
}

///补全活邻居图为全图
#[inline]
pub fn fill_live_neighbors_to_all_neighbors_with_availables(
    live_neighbors: &mut HashMap<Reg, HashSet<Reg>>,
    availables: &HashMap<Reg, RegUsedStat>,
) {
    let mut p_nbs: HashMap<Reg, HashSet<Reg>> = HashMap::new();
    for (reg, _) in live_neighbors.iter_mut() {
        let available = availables.get(reg).unwrap();
        for p_reg in Reg::get_all_regs() {
            if !available.is_available_reg(p_reg.get_color()) {
                if !p_nbs.contains_key(&p_reg) {
                    p_nbs.insert(p_reg, HashSet::new());
                }
                p_nbs.get_mut(&p_reg).unwrap().insert(*reg);
            }
        }
    }
    for (p, pnbs) in p_nbs.iter() {
        for v_nb in pnbs.iter() {
            live_neighbors.get_mut(v_nb).unwrap().insert(*p);
        }
    }
    live_neighbors.extend(p_nbs);
    check_if_full_neighbors(&live_neighbors);
}

///根据活邻居图构建全图
#[inline]
pub fn build_live_neighbors_to_all_neighbors_with_availables(
    live_neighbors: &HashMap<Reg, HashSet<Reg>>,
    availables: &HashMap<Reg, RegUsedStat>,
) -> HashMap<Reg, HashSet<Reg>> {
    let mut live_neighbors = live_neighbors.clone();
    let mut p_nbs: HashMap<Reg, HashSet<Reg>> = HashMap::new();
    for (reg, nbs) in live_neighbors.iter_mut() {
        let available = availables.get(reg).unwrap();
        for p_reg in Reg::get_all_regs() {
            if available.is_available_reg(reg.get_color()) {
                if !p_nbs.contains_key(&p_reg) {
                    p_nbs.insert(p_reg, HashSet::new());
                }
                p_nbs.get_mut(&p_reg).unwrap().insert(*reg);
            }
        }
    }
    for (p, pnbs) in p_nbs.iter() {
        for v_nb in pnbs.iter() {
            live_neighbors.get_mut(v_nb).unwrap().insert(*p);
        }
    }
    live_neighbors.extend(p_nbs);
    check_if_full_neighbors(&live_neighbors);
    live_neighbors
}

///检查是否是完整的图 ,两个节点间有双向边
#[inline]
pub fn check_if_full_neighbors(all_neighbors: &HashMap<Reg, HashSet<Reg>>) {
    debug_assert!(|| -> bool {
        for (r, nbs) in all_neighbors.iter() {
            for nb in nbs.iter() {
                if !all_neighbors.get(nb).unwrap().contains(r) {
                    return false;
                }
            }
            debug_assert!(!nbs.contains(r));
        }
        true
    }());
}

///化简活邻接图,获取最后消去序列
/// 最后消去序列中的元素满足,如果去掉最后消去序列中的节点得到的诱导子图能够完美分配
/// 则原本的图一定能够完美分配
/// 分配方式为:先给出化简后的活邻居图的一种完美着色方式,然后再按照最后消去序列中节点顺序从前往后进行着色,则一定能够完美分配
#[inline]
pub fn simplify_live_graph_and_build_last_tocolors(
    live_intereference_graph: &mut HashMap<Reg, HashSet<Reg>>,
    availables: &HashMap<Reg, RegUsedStat>,
) -> LinkedList<Reg> {
    check_if_full_neighbors(&live_intereference_graph);
    let mut last_to_colors: LinkedList<Reg> = LinkedList::new();
    //新节点加入方式push front
    //最后着色方式pop front
    let mut to_rm: Vec<Reg> = Vec::with_capacity(live_intereference_graph.len());
    loop {
        for (reg, nbs) in live_intereference_graph.iter() {
            let available = availables.get(reg).unwrap();
            let na = available.num_available_regs(reg.get_type());
            let nln = nbs.len();
            if na > nln {
                to_rm.push(*reg);
            }
        }
        if to_rm.len() == 0 {
            break;
        }
        if to_rm.len() == live_intereference_graph.len() {
            // 对最后一次移除做特殊处理以加速
            for reg in to_rm {
                last_to_colors.push_front(reg);
            }
            live_intereference_graph.clear();
            break;
        }

        for reg in to_rm.iter() {
            last_to_colors.push_front(*reg);
            let nbs = live_intereference_graph.remove(&reg).unwrap();
            for nb in nbs.iter() {
                live_intereference_graph.get_mut(nb).unwrap().remove(&reg);
            }
        }
        to_rm.clear();
    }
    last_to_colors
}

///寻找不影响总体可着色性的个体可着色方案并返回
pub fn simplify_live_graph_and_build_pre_colors(
    live_intereference_graph: &mut HashMap<Reg, HashSet<Reg>>,
    availables: &HashMap<Reg, RegUsedStat>,
) -> HashMap<Reg, i32> {
    let mut pre_colors = HashMap::new();
    let mut new_pre_color: Vec<Reg> = Vec::with_capacity(live_intereference_graph.len());
    loop {
        for (r, nbs) in live_intereference_graph.iter() {
            // 以一个使用了所有寄存器的情况为基底
            let mut reg_use_stat = RegUsedStat::init_unavailable();
            for nb in nbs.iter() {
                reg_use_stat.inter(availables.get(nb).unwrap());
            }
            let self_available = *availables.get(r).unwrap();
            // 在非特殊寄存器中寻找自己允许以及该reg_use_stat不允许的一个例子
            let mut choices = Reg::get_all_regs();
            choices.retain(|reg| self_available.is_available_reg(reg.get_color()));
            choices.retain(|reg| !reg_use_stat.is_available_reg(reg.get_color()));
            choices.retain(|reg| reg.get_type() == r.get_type());
            if choices.len() == 0 {
                continue;
            }
            let color = {
                let mut color = None;
                for r in choices {
                    color = Some(r.get_color());
                }
                color.unwrap()
            };
            pre_colors.insert(*r, color);
            new_pre_color.push(*r);
        }
        if new_pre_color.len() == 0 {
            break;
        }

        if new_pre_color.len() == live_intereference_graph.len() {
            live_intereference_graph.clear();
            break;
        }

        for r in new_pre_color.iter() {
            let nbs = live_intereference_graph.remove(r).unwrap();
            for nb in nbs.iter() {
                live_intereference_graph.get_mut(nb).unwrap().remove(r);
            }
        }
        new_pre_color.clear();
    }
    pre_colors
}

///对最后作色序列进行着色
/// 注意,这里传入的live_neigbhors应该是最初的live_neigbhors
#[inline]
pub fn color_last_tocolors(
    last_tocolors: &mut LinkedList<Reg>,
    live_neighbors: &HashMap<Reg, HashSet<Reg>>,
    availables: &mut HashMap<Reg, RegUsedStat>,
) -> HashMap<i32, i32> {
    let mut colors: HashMap<i32, i32> = HashMap::new();
    // 此处的图应该是原本的简图
    while !last_tocolors.is_empty() {
        let reg = last_tocolors.pop_front().unwrap();
        let available = availables.get(&reg).unwrap();
        let color = available.get_available_reg(reg.get_type()).unwrap();
        colors.insert(reg.get_id(), color);
        for nb in live_neighbors.get(&reg).unwrap().iter() {
            availables.get_mut(nb).unwrap().use_reg(color);
        }
    }
    colors
}

#[inline]
pub fn alloc_with_interef_graph_and_availables_and_constraints(
    intereference_graph: HashMap<Reg, HashSet<Reg>>,
    availables: HashMap<Reg, RegUsedStat>,
    constraints: &HashMap<Reg, HashSet<Reg>>,
) -> Option<FuncAllocStat> {
    let mut all_neighbors = intereference_graph;
    let mut availables = availables;
    // 根据constraints更新
    add_constraint_to_interference_graph(&mut all_neighbors, &mut availables, constraints);
    let mut live_neighbors = build_live_neighbors_from_all_neigbhors(&all_neighbors);
    alloc_with_live_neighbors_and_availables(&mut live_neighbors, &mut availables)
}

#[inline]
pub fn alloc_with_live_neighbors_and_availables(
    live_neighbors: &mut HashMap<Reg, HashSet<Reg>>,
    availables: &mut HashMap<Reg, RegUsedStat>,
) -> Option<FuncAllocStat> {
    let base_live_neighbors = live_neighbors.clone();
    let mut last_to_colors = LinkedList::new();
    let mut pre_colors: HashMap<Reg, i32> = HashMap::new();
    // 迭代化简图,直到化简结束
    loop {
        let tmp_last_tocolors =
            simplify_live_graph_and_build_last_tocolors(live_neighbors, &availables);
        for reg in tmp_last_tocolors.iter().rev() {
            last_to_colors.push_front(*reg);
        }
        if live_neighbors.len() == 0 {
            break;
        }
        let tmp_pre_colors = simplify_live_graph_and_build_pre_colors(live_neighbors, &availables);
        pre_colors.extend(tmp_pre_colors.iter());
        if live_neighbors.len() == 0 {
            break;
        }
        if tmp_last_tocolors.len() == 0 && tmp_pre_colors.len() == 0 {
            break;
        }
    }

    // 加入pre color的影响,需要考虑pre color 对剩余未着色寄存器的影响
    for (r, color) in pre_colors.iter() {
        let nbs = base_live_neighbors.get(r).unwrap();
        for nb in nbs.iter() {
            availables.get_mut(nb).unwrap().use_reg(*color);
        }
    }

    if live_neighbors.len() == 0 {
        let mut colors = color_last_tocolors(&mut last_to_colors, &base_live_neighbors, availables);
        for (r, color) in pre_colors.iter() {
            colors.insert(r.get_id(), *color);
        }
        let fat = FuncAllocStat {
            spillings: HashSet::new(),
            dstr: colors,
        };
        return Some(fat);
    }

    // 如果不能够通过化简直接完成分配,则使用chordal_alloc进行分配 (利用弦图性质)
    let FuncAllocStat {
        mut dstr,
        spillings,
    } = chordal_alloc::alloc_with_live_neighbors_and_availables(
        live_neighbors.clone(),
        availables.clone(),
    );
    let mut availables = availables;
    if spillings.len() == 0 {
        // 加入chordal着色的结果
        for (r, _) in live_neighbors.iter() {
            let color = dstr.get(&r.get_id()).unwrap();
            for nb in base_live_neighbors.get(r).unwrap() {
                availables.get_mut(nb).unwrap().use_reg(*color);
            }
        }
        let colors =
            color_last_tocolors(&mut last_to_colors, &base_live_neighbors, &mut availables);
        dstr.extend(colors);

        // dstr还要加上pre_colors
        for (r, color) in pre_colors {
            dstr.insert(r.get_id(), color);
        }

        return Some(FuncAllocStat { spillings, dstr });
    }
    None
}
