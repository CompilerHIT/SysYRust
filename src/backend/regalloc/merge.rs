use std::collections::{HashMap, HashSet};

use crate::{
    backend::{
        instrs::{Func, InstrsType, SingleOp},
        operand::Reg,
        regalloc::structs::RegUsedStat,
    },
    config, log_file,
    utility::ObjPtr,
};

use super::{perfect_alloc::alloc_with_v_interference_graph_and_base_available, *};

///进行了寄存器合并的分配,在最后的最后进行
/// availables为可能各个地方允许使用的寄存器的并集
pub fn merge_reg_with_constraints(
    func: &mut Func,
    availables: &HashSet<Reg>,
    regs_used_but_not_saved: &HashMap<String, HashSet<Reg>>,
) -> bool {
    /*
    准备基础约束
     */
    let availables: HashSet<Reg> = availables.clone();
    let mut unavailables = Reg::get_all_regs();
    unavailables.retain(|reg| !availables.contains(reg));
    let unavailables = unavailables;
    /*
    准备每次合并的过程
     */

    let per_process = |func: &mut Func,
                       r1: &Reg,
                       r2: &Reg,
                       interef_graph: &mut HashMap<Reg, HashSet<Reg>>,
                       availables: &mut HashMap<Reg, RegUsedStat>,
                       constraints: &mut HashMap<Reg, HashSet<Reg>>|
     -> bool {
        if !constraints.contains_key(r1) || !constraints.contains_key(r2) {
            return false;
        }
        if interef_graph.get(r1).unwrap().contains(r2) {
            debug_assert!(interef_graph.get(r2).unwrap().contains(r1));
            return false;
        }
        debug_assert!(r1.get_type() == r2.get_type());
        debug_assert!(!interef_graph.get(r1).unwrap().contains(r2));
        debug_assert!(!interef_graph.get(r2).unwrap().contains(r1));
        Func::print_func(ObjPtr::new(func), "rm_inst.txt");
        log_file!("final_merge.txt", "try merge:{},{}", r1, r2);
        debug_assert!(
            availables.contains_key(r1) && availables.contains_key(r2),
            "{},{},{},{},{}",
            {
                // Func::print_func(ObjPtr::new(func), "rm_inst.txt");
                let ok = func.remove_self_mv();
                ok
            },
            r1,
            r2,
            availables.contains_key(r1),
            availables.contains_key(r2)
        );
        //尝试合并,
        let old_r1_constraints = constraints.remove(r1).unwrap();
        let old_r2_constraints = constraints.remove(r2).unwrap();
        //移除r1和r2的约束,然后从冲突图中移除原本的r1和r2
        let old_r1_inter = remove_node_from_intereference_graph(interef_graph, r1).unwrap();
        let old_r2_inter = remove_node_from_intereference_graph(interef_graph, r2).unwrap();
        let mut merge_inter = old_r1_inter.clone();
        merge_inter.extend(old_r2_inter.iter());
        merge_inter.extend(old_r1_constraints.iter());
        merge_inter.extend(old_r2_constraints.iter());
        merge_inter.retain(|reg| reg.is_physic());
        if merge_inter.len() == Reg::get_all_regs().len() {
            return false;
        }
        //最后尝试合并,从表中把r1,r2去掉,然后加入r3
        let new_v = Reg::init(r1.get_type());
        let mut new_v_links_to: HashSet<Reg> = old_r1_inter.clone();
        new_v_links_to.extend(old_r2_inter.iter());
        new_v_links_to.remove(r1);
        new_v_links_to.remove(r2);
        add_node_to_intereference_graph(interef_graph, &new_v, new_v_links_to.clone());
        let mut new_v_constraint: HashSet<Reg> = old_r1_constraints.clone();
        new_v_constraint.extend(old_r2_constraints.iter());
        constraints.insert(new_v, new_v_constraint.clone());
        //更新available
        let old_r1_available = availables.remove(r1).unwrap();
        let old_r2_available = availables.remove(r2).unwrap();

        let mut new_v_available = old_r1_available.clone();
        new_v_available.merge(&old_r2_available);

        availables.insert(new_v, new_v_available);

        //如果该移动边的两个顶点 其中有一个的邻居都是 小度点,则合并成功

        //尝试着色
        if let Some(_) = alloc_with_v_interference_graph_and_base_available(
            &interef_graph,
            &availables,
            &constraints,
        ) {
            log_file!("final_merge.txt", "merge:{},{}", r1, r2);
            //把func中所有寄存器通通替换
            func.replace_v_reg(r1, &new_v);
            func.replace_v_reg(r2, &new_v);
            return true;
        } else {
            constraints.remove(&new_v);
            constraints.insert(*r1, old_r1_constraints);
            constraints.insert(*r2, old_r2_constraints);
            remove_node_from_intereference_graph(interef_graph, &new_v);
            add_node_to_intereference_graph(interef_graph, r2, old_r2_inter);
            add_node_to_intereference_graph(interef_graph, r1, old_r1_inter);
            availables.insert(*r1, old_r1_available);
            availables.insert(*r2, old_r2_available);
            availables.remove(&new_v);
        }
        return false;
    };
    /*
    合并物理寄存器和虚拟寄存器
     */
    let per_process_between_v_and_p = |func: &mut Func,
                                       r1: &Reg,
                                       r2: &Reg,
                                       interef_graph: &mut HashMap<Reg, HashSet<Reg>>,
                                       availables: &mut HashMap<Reg, RegUsedStat>,
                                       constraints: &mut HashMap<Reg, HashSet<Reg>>|
     -> bool {
        //合并物理寄存器和虚拟寄存器
        if !r1.is_physic() && !r2.is_physic() {
            return false;
        }
        if r1.is_physic() && r2.is_physic() {
            return false;
        }
        //对于物理寄存器和虚拟寄存器的合并
        let (v_reg, p_reg) = if r1.is_physic() { (r1, r2) } else { (r2, r1) };
        if !availables
            .get(v_reg)
            .unwrap()
            .is_available_reg(p_reg.get_color())
        {
            return false;
        }
        //记录加unavailable序列
        let mut add_to_availables: Vec<(Reg, i32)> = Vec::new();
        let p_color = p_reg.get_color();
        for v_nb in interef_graph.get(v_reg).unwrap() {
            debug_assert!(!v_nb.is_physic());
            let available = availables.get_mut(v_nb).unwrap();
            if available.is_available_reg(p_color) {
                available.use_reg(p_color);
                add_to_availables.push((*v_nb, p_color));
            }
        }
        unimplemented!();

        false
    };

    /*
    p2v 并记录 去色动作序列
     */
    let to_recolors = Reg::get_all_recolorable_regs();

    let (_, p2v_actions) = func.p2v(&to_recolors);

    /*
    准备待合并列表
    并
    初始化对新虚拟寄存器的约束
    并根据约束分析可行性,根据可行性对 待合并列表进行排序
    */
    let mergables: HashSet<(Reg, Reg)> = analyse_mergable(func);
    let mut constraints: HashMap<Reg, HashSet<Reg>> =
        build_constraints(func, regs_used_but_not_saved);
    let all_virtual_regs: HashSet<Reg> = func.draw_all_virtual_regs();
    //对所有寄存器建立除了availables以外的约束
    for v_reg in all_virtual_regs.iter() {
        if !constraints.contains_key(v_reg) {
            constraints.insert(v_reg.clone(), HashSet::new());
        }
        constraints
            .get_mut(v_reg)
            .unwrap()
            .extend(unavailables.iter());
    }

    let merge_lst: HashSet<(Reg, Reg)> = mergables
        .iter()
        .map(|(reg, reg2)| {
            if reg.get_id() < reg2.get_id() {
                (*reg, *reg2)
            } else {
                (*reg2, *reg)
            }
        })
        .collect();
    let mut merge_lst: Vec<(Reg, Reg)> = merge_lst.iter().cloned().collect();
    merge_lst.sort_by_cached_key(|(r1, r2)| {
        let mut ct1 = constraints.get(r1).unwrap().clone();
        ct1.extend(constraints.get(r2).unwrap().iter());
        ct1.len()
    });

    /*
    建立活寄存器图和可用寄存器表
     */
    // func.calc_live_base();
    let mut interef_graph = regalloc::build_interference(func);
    //在总图上建立可用表
    let mut availables = regalloc::build_availables_with_interef_graph(&interef_graph);
    //把冲突图转化为活图,去掉图中的物理寄存器
    interef_graph.retain(|reg, _| !reg.is_physic());
    for (r, inter) in interef_graph.iter_mut() {
        debug_assert!(availables.contains_key(r));
        inter.retain(|reg| !reg.is_physic());
    }

    let mut if_merge = false;
    //统计所有的可着色对,然后按照约束和从小到大的顺序开始着色,如果失败,从表中移出
    for (r1, r2) in mergables.iter() {
        if r1.is_physic() || r2.is_physic() {
            // let ok = per_process_between_v_and_p(
            //     func,
            //     r1,
            //     r2,
            //     &mut interef_graph,
            //     &mut availables,
            //     &mut constraints,
            // );
            // if_merge |= ok;
            // todo!()
            continue;
        }

        debug_assert!(!r1.is_physic() && !r2.is_physic());
        let ok = per_process(
            func,
            r1,
            r2,
            &mut interef_graph,
            &mut availables,
            &mut constraints,
        );
        if_merge |= ok;
        if ok {
            config::record_merge_reg(&func.label, r1, r2);
        }
    }
    //如果合并成功
    if if_merge {
        assert!(func.remove_self_mv());
        loop {
            if let Some(alloc_stat) = alloc_with_v_interference_graph_and_base_available(
                &interef_graph,
                &availables,
                &constraints,
            ) {
                func.v2p(&alloc_stat.dstr);
                break;
            }
        }
    } else {
        Func::undo_p2v(&p2v_actions);
    }
    if_merge
}

///根据regs used but not saved建立冲突图
fn build_constraints(
    func: &Func,
    regs_used_but_not_saved: &HashMap<String, HashSet<Reg>>,
) -> HashMap<Reg, HashSet<Reg>> {
    func.calc_live_base();
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

//分析虚拟寄存器的合并机会
pub fn analyse_mergable(func: &Func) -> HashSet<(Reg, Reg)> {
    let mut mergables: HashSet<(Reg, Reg)> = HashSet::new();
    //分析可以合并的虚拟寄存器
    func.calc_live_base();
    for bb in func.blocks.iter() {
        Func::analyse_inst_with_live_now_backorder(
            *bb,
            &mut |inst, live_now| match inst.get_type() {
                InstrsType::OpReg(SingleOp::Mv) => {
                    let reg_use = inst.get_lhs().drop_reg();
                    let reg_def = inst.get_def_reg().unwrap();
                    if live_now.contains(&reg_use) {
                        return;
                    }
                    if reg_use.get_type() != reg_def.get_type() {
                        unreachable!();
                    }
                    mergables.insert((reg_use, reg_def));
                    mergables.insert((reg_def, reg_use));
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
        return Some(rm);
    } else {
        return Some(HashSet::new());
    };
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
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_let() {
        // let mut mm: HashSet<i32>;
        // assert!(mm.len() == 0);
    }
}
