use std::collections::{HashMap, HashSet};

use crate::{
    backend::{
        instrs::{Func, InstrsType, SingleOp},
        operand::Reg,
        regalloc::structs::RegUsedStat,
    },
    frontend::preprocess,
};

use super::{perfect_alloc::alloc_with_v_interference_graph_and_base_available, *};

///进行了寄存器合并的分配,在最后的最后进行
pub fn alloc_with_merge(func: &mut Func, reg_used_but_not_saveds: &HashMap<String, HashSet<Reg>>) {
    //首先对寄存器除了特殊寄存器以外的寄存器使用进行p2v
    //然后统计合并机会
    //然后重新分配,从小度开始合并
    //直到合无可合则结束合并
    //availables 为能够使用的寄存器
    let availables: HashSet<Reg> = { todo!() };
    let mut unavailables = Reg::get_all_regs();
    unavailables.retain(|reg| !availables.contains(reg));
    let regs_to_decolor = Reg::get_all_recolorable_regs();
    let per_process = |func: &mut Func,
                       r1: &Reg,
                       r2: &Reg,
                       interef_graph: &mut HashMap<Reg, HashSet<Reg>>,
                       availables: &mut HashMap<Reg, RegUsedStat>,
                       constraints: &mut HashMap<Reg, HashSet<Reg>>|
     -> bool {
        //尝试合并,直到某次合并成功
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
        if merge_inter.len() == 0 {
            return false;
        }
        //最后尝试合并,从表中把r1,r2去掉,然后加入r3
        let new_v = Reg::init(r1.get_type());
        let mut new_v_links_to: HashSet<Reg> = old_r1_inter.clone();
        new_v_links_to.extend(old_r2_inter.iter());
        add_node_to_intereference_graph(interef_graph, &new_v, new_v_links_to);
        let mut new_v_constraint: HashSet<Reg> = old_r1_constraints.clone();
        new_v_constraint.extend(old_r2_constraints.iter());
        constraints.insert(new_v, new_v_constraint);
        //更新available
        let old_r1_available = availables.remove(r1).unwrap();
        let old_r2_available = availables.remove(r2).unwrap();

        let mut new_v_available = old_r1_available.clone();
        new_v_available.merge(&old_r2_available);
        availables.insert(new_v, new_v_available);

        //尝试着色
        if let Some(_) = alloc_with_v_interference_graph_and_base_available(
            &interef_graph,
            &availables,
            &constraints,
        ) {
            //把func中所有寄存器通通替换
            func.replace_v_reg(r1, &new_v);
            func.replace_v_reg(r2, &new_v);
            return true;
        } else {
            constraints.remove(&new_v);
            remove_node_from_intereference_graph(interef_graph, &new_v);
            add_node_to_intereference_graph(interef_graph, r2, old_r2_inter);
            add_node_to_intereference_graph(interef_graph, r1, old_r1_inter);
            constraints.insert(*r1, old_r1_constraints);
            constraints.insert(*r2, old_r2_constraints);
            availables.insert(*r1, old_r1_available);
            availables.insert(*r2, old_r2_available);
            availables.remove(&new_v);
        }
        return false;
    };
    func.p2v_pre_handle_call(regs_to_decolor.clone());
    //分析有合并机会的寄存器对,
    let mergables: HashSet<(Reg, Reg)> = analyse_mergable(func);
    //分析约束,统计所有能够进行合并的可能,并按照合并可能进行合并,之后约束只会增加不会减少
    let mut constraints = build_constraints(&func, reg_used_but_not_saveds);
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
    let mut interef_graph = regalloc::build_interference(func);
    let mut availables = regalloc::build_availables_with_interef_graph(&interef_graph);
    //统计所有的可着色对,然后按照约束和从小到大的顺序开始着色,如果失败,从表中移出
    for (r1, r2) in mergables.iter() {
        per_process(
            func,
            r1,
            r2,
            &mut interef_graph,
            &mut availables,
            &mut constraints,
        );
    }
    //带寄存器合并的分配方式结束后,开始执行减少
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
                    if reg_use.is_physic() || reg_def.is_physic() {
                        return;
                    }
                    if reg_use.get_type() != reg_def.get_type() {
                        unreachable!();
                        return;
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

pub fn build_constraints(
    func: &Func,
    reg_used_but_not_saveds: &HashMap<String, HashSet<Reg>>,
) -> HashMap<Reg, HashSet<Reg>> {
    todo!()
}

//从冲突图中移除某个节点与其他节点关系
pub fn remove_node_from_intereference_graph(
    intereference_graph: &mut HashMap<Reg, HashSet<Reg>>,
    to_rm: &Reg,
) -> Option<HashSet<Reg>> {
    let rm = intereference_graph.remove(to_rm);
    if let Some(rm) = rm {
        for r in rm.iter() {
            intereference_graph.get_mut(&r).unwrap().remove(to_rm);
        }
        return Some(rm);
    }
    None
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
