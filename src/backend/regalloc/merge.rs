use std::collections::{HashMap, HashSet};

use crate::{
    backend::{
        instrs::{Func, InstrsType, SingleOp},
        operand::Reg,
        regalloc::{perfect_alloc::check_if_full_neighbors, structs::RegUsedStat},
    },
    config, log_file,
    utility::{ObjPtr, ScalarType},
};

use super::{structs::FuncAllocStat, *};

#[inline]
///尝试合并两个虚拟寄存器
pub fn try_merge_vv(
    func: &mut Func,
    v_reg1: &Reg,
    v_reg2: &Reg,
    live_neigbhors: &mut HashMap<Reg, HashSet<Reg>>,
    availables: &mut HashMap<Reg, RegUsedStat>,
) -> (bool, Option<FuncAllocStat>) {
    // return (false, None);
    debug_assert!(!v_reg1.is_physic());
    debug_assert!(!v_reg2.is_physic());
    debug_assert!(v_reg1.get_type() == v_reg2.get_type());
    if live_neigbhors.get(v_reg1).unwrap().contains(v_reg2) {
        return (false, None);
    }
    debug_assert!(!live_neigbhors.get(v_reg1).unwrap().contains(v_reg2));
    debug_assert!(!live_neigbhors.get(v_reg2).unwrap().contains(v_reg1));
    // 它们共享约束
    let mut v_nbs = live_neigbhors.get(&v_reg1).unwrap().clone();
    v_nbs.extend(live_neigbhors.get(&v_reg2).unwrap().iter());

    let mut new_v_available = *availables.get(v_reg1).unwrap();
    new_v_available.merge(availables.get(v_reg2).unwrap());
    let new_v_reg = Reg::init(v_reg1.get_type());
    //判断是否合并直接可以进行
    if new_v_available.num_available_regs(v_reg1.get_type()) > v_nbs.len() {
        remove_node_from_intereference_graph(live_neigbhors, v_reg1);
        remove_node_from_intereference_graph(live_neigbhors, v_reg2);
        add_node_to_intereference_graph(live_neigbhors, &new_v_reg, v_nbs);
        availables.insert(new_v_reg, new_v_available);
        availables.remove(v_reg1);
        availables.remove(v_reg2);
        func.replace_reg(v_reg1, &new_v_reg);
        func.replace_reg(v_reg2, &new_v_reg);
        return (true, None);
    }
    //使用新寄存器尝试进行
    let mut new_live_neigbhors = live_neigbhors.clone();
    let mut new_availables = availables.clone();
    remove_node_from_intereference_graph(&mut new_live_neigbhors, &v_reg1);
    remove_node_from_intereference_graph(&mut new_live_neigbhors, v_reg2);
    add_node_to_intereference_graph(&mut new_live_neigbhors, &new_v_reg, v_nbs.clone());

    new_availables.insert(new_v_reg, new_v_available);
    new_availables.remove(v_reg1);
    new_availables.remove(v_reg2);
    if let Some(alloc_stat) = perfect_alloc::alloc_with_live_neighbors_and_availables(
        &mut new_live_neigbhors,
        &mut new_availables,
    ) {
        remove_node_from_intereference_graph(live_neigbhors, v_reg1);
        remove_node_from_intereference_graph(live_neigbhors, v_reg2);
        add_node_to_intereference_graph(live_neigbhors, &new_v_reg, v_nbs);
        func.replace_reg(v_reg1, &new_v_reg);
        func.replace_reg(v_reg2, &new_v_reg);
        availables.insert(new_v_reg, new_v_available);
        availables.remove(v_reg1);
        availables.remove(v_reg2);
        return (true, Some(alloc_stat));
    }
    (false, None)
}

///尝试合并两个物理寄存器
#[inline]
pub fn try_merge_vp(
    func: &mut Func,
    v_reg: &Reg,
    p_reg: &Reg,
    live_neigbhors: &mut HashMap<Reg, HashSet<Reg>>,
    availables: &mut HashMap<Reg, RegUsedStat>,
) -> (bool, Option<FuncAllocStat>) {
    // return (false, None);
    debug_assert!(!v_reg.is_physic());
    debug_assert!(p_reg.is_physic());
    debug_assert!(v_reg.get_type() == p_reg.get_type());
    if !availables
        .get(v_reg)
        .unwrap()
        .is_available_reg(p_reg.get_color())
    {
        return (false, None);
    }
    // 首先快速判断是否能够直接着色

    // // 或者如果该寄存器直接地变成该物理寄存器不会影响图的可着色性
    let mut if_ok = false;
    for nb in live_neigbhors.get(v_reg).unwrap() {
        let mut av = *availables.get(nb).unwrap();
        if !av.is_available_reg(p_reg.get_color()) {
            continue;
        }
        av.use_reg(p_reg.get_color());
        let na = av.num_available_regs(nb.get_type());
        let nln = live_neigbhors.get(nb).unwrap().len() - 1;
        if na <= nln {
            if_ok = true;
            break;
        }
    }
    if !if_ok {
        func.replace_reg(v_reg, p_reg);
        let nbs = remove_node_from_intereference_graph(live_neigbhors, v_reg);
        for nb in nbs.unwrap() {
            availables.get_mut(&nb).unwrap().use_reg(p_reg.get_color());
        }
        availables.remove(v_reg);
        return (true, None);
    }

    //最后是常规寄存器合并尝试
    let mut new_live_neigbhors = live_neigbhors.clone();
    let mut new_availables = availables.clone();
    let nbs = remove_node_from_intereference_graph(&mut new_live_neigbhors, v_reg).unwrap();
    for nb in nbs.iter() {
        new_availables
            .get_mut(nb)
            .unwrap()
            .use_reg(p_reg.get_color());
    }
    if let Some(func_alloc_stat) = perfect_alloc::alloc_with_live_neighbors_and_availables(
        &mut new_live_neigbhors,
        &mut new_availables,
    ) {
        func.replace_reg(v_reg, p_reg);
        let nbs = remove_node_from_intereference_graph(live_neigbhors, v_reg);
        for nb in nbs.unwrap() {
            availables.get_mut(&nb).unwrap().use_reg(p_reg.get_color());
        }
        availables.remove(v_reg);
        return (true, Some(func_alloc_stat));
    }

    (false, None)
}

///新寄存器合并
pub fn merge_reg_with_constraints(
    func: &mut Func,
    availables: &HashSet<Reg>,
    regs_used_but_not_saved: &HashMap<String, HashSet<Reg>>,
) -> bool {
    let merge_action_path = "merge_actions.txt";
    log_file!(merge_action_path, "func:{}", func.label);

    debug_assert!(!func.remove_unuse_def());
    Func::print_func(ObjPtr::new(&func), "before_p2v_for_merge.txt");
    let availables: HashSet<Reg> = availables.clone();
    let mut unavailables_reg_use_stat = RegUsedStat::init_unavailable();
    for reg in availables.iter() {
        unavailables_reg_use_stat.release_reg(reg.get_color());
    }
    //首先p2v
    let (_, p2v_actions) = func.p2v(&Reg::get_all_recolorable_regs());
    Func::print_func(ObjPtr::new(&func), "before_merge.txt");

    //p2v后处理
    func.calc_live_base();
    let mut all_neighbors = regalloc::build_interference(func);
    let mut availables = regalloc::build_availables_with_interef_graph(&all_neighbors);
    // 建立约束,加入约束
    // 对于所有得虚拟寄存器都应该加入extend约束
    let constraints = build_constraints(func, regs_used_but_not_saved);
    perfect_alloc::add_constraint_to_interference_graph(
        &mut all_neighbors,
        &mut availables,
        &constraints,
    );
    let mut live_neighbors = perfect_alloc::build_live_neighbors_from_all_neigbhors(&all_neighbors);
    for (r, _) in live_neighbors.iter() {
        availables
            .get_mut(r)
            .unwrap()
            .merge(&unavailables_reg_use_stat);
    }

    debug_assert!(live_neighbors.len() == func.draw_all_virtual_regs().len());
    // 分析合并机会
    let mut mergables = analyse_mergable(func);
    // 按照着色对可着色性的减少程度来排序可着色性
    mergables.sort_by_cached_key(|(r1, r2)| {
        let mut ct1 = if r1.is_physic() {
            HashSet::new()
        } else {
            live_neighbors.get(r1).unwrap().clone()
        };
        if !r2.is_physic() {
            debug_assert!(live_neighbors.contains_key(r2), "{}", {
                Func::print_func(ObjPtr::new(func), "sort_merge.txt");
                r2
            });
            ct1.extend(live_neighbors.get(r2).unwrap());
        };
        ct1.len()
    });
    let mut if_merge = false;
    let mut pre_alloc_stat: Option<FuncAllocStat> = None;
    for (r1, r2) in mergables.iter() {
        if !r1.is_physic() && !live_neighbors.contains_key(r1) {
            continue;
        }
        if !r2.is_physic() && !live_neighbors.contains_key(r2) {
            continue;
        }

        log_file!(merge_action_path, "try_merge {},{}", r1, r2);

        let result = if r1.is_physic() {
            try_merge_vp(func, r2, r1, &mut live_neighbors, &mut availables)
        } else if r2.is_physic() {
            try_merge_vp(func, r1, r2, &mut live_neighbors, &mut availables)
        } else {
            try_merge_vv(func, r1, r2, &mut live_neighbors, &mut availables)
        };
        if_merge |= result.0;

        if result.0 {
            log_file!(
                merge_action_path,
                "merge {},{} at:{}",
                r1,
                r2,
                config::get_passed_secs()
            );
            debug_assert!(!live_neighbors.contains_key(r1));
            debug_assert!(!live_neighbors.contains_key(r2));
            config::record_merge_reg(&func.label, r1, r2);
            if let Some(fas) = result.1 {
                pre_alloc_stat = Some(fas);
            } else {
                pre_alloc_stat = None;
            }
        }
    }
    Func::print_func(ObjPtr::new(&func), "after_merge.txt");
    if if_merge {
        log_file!(
            merge_action_path,
            "finall alloc for merge at {}",
            config::get_passed_secs()
        );
        assert!(func.remove_self_mv());
        assert!(!func.remove_unuse_def());
        if let Some(alloc_stat) = pre_alloc_stat {
            func.v2p(&alloc_stat.dstr);
        } else {
            // unreachable!();
            Func::print_func(ObjPtr::new(&func), "merge.txt");
            loop {
                let alloc_stat = perfect_alloc::alloc_with_live_neighbors_and_availables(
                    &mut live_neighbors,
                    &mut availables,
                );
                if alloc_stat.is_none() {
                    continue;
                }
                let alloc_stat = alloc_stat.unwrap();
                if alloc_stat.spillings.len() != 0 {
                    unreachable!();
                    continue;
                }
                func.v2p(&alloc_stat.dstr);
                break;
            }
        }
    } else {
        Func::undo_p2v(&p2v_actions);
    }
    debug_assert!(func.draw_all_virtual_regs().len() == 0);
    func.remove_self_mv();
    func.calc_live_base();
    assert!(!func.remove_unuse_def());
    if_merge
}

///根据regs used but not saved 以及 live interval建立冲突
///live interval依赖于外部调用的calc live
fn build_constraints(
    func: &Func,
    regs_used_but_not_saved: &HashMap<String, HashSet<Reg>>,
) -> HashMap<Reg, HashSet<Reg>> {
    let mut constraints = HashMap::new();
    for bb in func.blocks.iter() {
        Func::analyse_inst_with_live_now_backorder(*bb, &mut |inst, live_now| {
            if inst.get_type() != InstrsType::Call {
                return;
            }
            let func = inst.get_func_name().unwrap();
            let constraint: HashSet<Reg> =
                regs_used_but_not_saved.get(func.as_str()).unwrap().clone();

            let live_now = live_now.clone();
            for r in live_now.iter().filter(|reg| !reg.is_physic()) {
                if !constraints.contains_key(r) {
                    constraints.insert(*r, constraint.clone());
                } else {
                    constraints.get_mut(r).unwrap().extend(constraint.iter());
                }
            }
        });
    }
    return constraints;
}

//分析寄存器的合并机会,依赖外部调用得calc live
pub fn analyse_mergable(func: &Func) -> Vec<(Reg, Reg)> {
    let mut mergables: Vec<(Reg, Reg)> = Vec::new();
    //分析可以合并的虚拟寄存器
    for bb in func.blocks.iter() {
        Func::analyse_inst_with_live_now_backorder(
            *bb,
            &mut |inst, live_now| match inst.get_type() {
                InstrsType::OpReg(SingleOp::Mv) => {
                    // println!("{},{},{}", func.label, bb.label, inst.as_ref().to_string());
                    let reg_use = inst.get_lhs().drop_reg();
                    let reg_def = inst.get_def_reg().unwrap();
                    if live_now.contains(&reg_use) {
                        return;
                    }
                    if reg_use.is_physic() && reg_def.is_physic() {
                        return;
                    }
                    if reg_use.get_type() != reg_def.get_type() {
                        unreachable!();
                    }
                    mergables.push((reg_use, reg_def));
                }
                _ => (),
            },
        );
    }
    return mergables;
}

//从冲突图中移除某个节点与其他节点关系
pub fn remove_node_from_intereference_graph(
    intereference_graph: &mut HashMap<Reg, HashSet<Reg>>,
    to_rm: &Reg,
) -> Option<HashSet<Reg>> {
    let rm = intereference_graph.remove(to_rm);
    let fms = if let Some(rm) = rm {
        for r in rm.iter() {
            intereference_graph.get_mut(&r).unwrap().remove(to_rm);
        }
        rm
    } else {
        HashSet::new()
    };
    check_if_full_neighbors(&intereference_graph);
    Some(fms)
}

//加回某个节点与其他节点的关系
pub fn add_node_to_intereference_graph(
    intereference_graph: &mut HashMap<Reg, HashSet<Reg>>,
    to_add: &Reg,
    links: HashSet<Reg>,
) {
    let out = intereference_graph.insert(*to_add, links.clone());
    debug_assert!(out.is_none());
    for r in links.iter() {
        if !intereference_graph.contains_key(r) {
            intereference_graph.insert(*r, HashSet::new());
        }
        intereference_graph.get_mut(r).unwrap().insert(*to_add);
    }
    check_if_full_neighbors(&intereference_graph);
}
