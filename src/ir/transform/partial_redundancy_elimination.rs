use std::collections::HashMap;

use crate::{
    ir::{
        analysis::{
            dominator_tree::{calculate_dominator, DominatorTree},
            downstream_tree::DownStreamTree,
        },
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
        module::Module,
        tools::{func_process, replace_inst},
        transform::{
            dead_code_eliminate::{dead_code_eliminate, global_eliminate},
            phi_optimizer::phi_run,
        },
    },
    utility::{ObjPool, ObjPtr},
};

use super::{
    delete_empty_block::{block_opt, replace_bb_with_bbs},
    global_value_numbering::{self, Congruence},
    gvn_hoist::{hoist_group, make_same_inst},
};

pub struct PreContext {
    index: i32,
}

impl PreContext {
    pub fn get_new_block(
        &mut self,
        bb_old: ObjPtr<BasicBlock>,
        pool: &mut ObjPool<BasicBlock>,
    ) -> ObjPtr<BasicBlock> {
        let name_old = bb_old.get_name().to_string();
        let name_new = name_old + "_PRE_" + self.index.to_string().as_str();
        self.index += 1;
        pool.new_basic_block(name_new)
    }
}

pub fn pre(
    module: &mut Module,
    opt_option: bool,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    block_opt(module, pools, opt_option);
    let mut vec_congruence_class = global_value_numbering::gvn(module, opt_option).unwrap();
    let mut index = 0;
    func_process(module, |_, func| {
        for congruence in vec_congruence_class[index].get_all_congruence_mut() {
            pre_congruence(congruence, func.get_head(), pools);
        }
        index += 1;
    });
    phi_run(module);
    dead_code_eliminate(module, opt_option);
    global_eliminate(module);
    // println!("pre finished");
}

pub fn pre_congruence(
    congruence: &mut Congruence,
    head: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut pre_context = PreContext { index: 0 };
    let mut dominator_tree = calculate_dominator(head);
    let mut downstream_tree = DownStreamTree::make_downstream_tree(head, &dominator_tree);
    for index_class in 0..congruence.vec_class.len() {
        loop {
            let mut changed = false;
            changed |= pre_group(
                &mut pre_context,
                head,
                &mut dominator_tree,
                &mut downstream_tree,
                congruence,
                index_class,
                pools,
            );
            changed |= hoist_group(congruence, index_class, &dominator_tree, pools.1);
            if !changed {
                break;
            }
        }
    }
}

pub fn pre_group(
    pre_context: &mut PreContext,
    head: ObjPtr<BasicBlock>,
    dominator_tree: &mut DominatorTree,
    downstream_tree: &mut DownStreamTree,
    congruence: &mut Congruence,
    index_class: usize,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> bool {
    let mut changed = false;
    loop {
        let mut flag = false;
        if congruence.vec_class[index_class].len() > 0 {
            match congruence.vec_class[index_class][0].get_kind() {
                InstKind::Gep => {
                    break;
                }
                _ => {}
            }
        }
        for i in 0..congruence.vec_class[index_class].len() {
            for j in i+1..congruence.vec_class[index_class].len() {
                if downstream_tree.is_upstream(
                    congruence.vec_class[index_class][j].get_parent_bb(),
                    congruence.vec_class[index_class][i].get_parent_bb(),
                ) {// 其中一个指令所在块是另一个块的上游，且不互为上下游(不在同一个循环体中)
                    let down = congruence.vec_class[index_class][i].get_parent_bb();
                    let pres = down.get_up_bb();
                    if pres.len() == 1 {
                        if pres[0].get_next_bb().len() > 1 {
                            continue;
                        }
                    }
                    flag |= true;
                    changed |= true;
                    if insert_inst_in_pre(
                        index_class,
                        congruence,
                        congruence.vec_class[index_class].clone(),
                        pre_context,
                        congruence.vec_class[index_class][i],
                        &dominator_tree,
                        pools.1,
                        pools.0,
                    ) {
                        // println!("计算新树");
                        *dominator_tree = calculate_dominator(head);
                        *downstream_tree = DownStreamTree::make_downstream_tree(head,dominator_tree);
                        // println!("计算新支配树");
                        // *dominator_tree = calculate_dominator(head);
                        // println!("计算完成");
                    }
                    break; //替换过指令，刷新，从头开始比较
                }
            }
            if flag {
                //替换过指令，刷新，从头开始比较
                break;
            }
        }
        if !flag {
            // println!("end loop");
            break;
        }
    }
    changed
}
// todo:循环中的pre需要另外处理

pub fn insert_inst_in_pre(
    index: usize,
    congruence: &mut Congruence,
    gp: Vec<ObjPtr<Inst>>,
    pre_context: &mut PreContext,
    inst_old: ObjPtr<Inst>,
    dominator_tree: &DominatorTree,
    pool_inst: &mut ObjPool<Inst>,
    pool_block: &mut ObjPool<BasicBlock>,
) -> bool {
    let bb = inst_old.get_parent_bb();
    let mut pres = bb.get_up_bb().clone();
    let mut vec_index = vec![]; // 记录自己所支配前继的索引
    let mut vec_operands_phi = vec![];
    let mut flag = false;
    //todo:只有一个前继的情况需要特殊处理
    //只有一个前继且被自己所支配的情况不予考虑
    //只有一个前继且前继有多个后继,不处理
    //只有一个前继且前继只有当前块一个后继，直接把指令塞上去
    if pres.len() == 1 {
        if pres[0].get_next_bb().len() == 1 {
            let inst_new = make_same_inst(inst_old, pool_inst);
            pres[0].get_tail_inst().as_mut().insert_before(inst_new);
            replace_inst(inst_old, inst_new);
            congruence.remove_inst(inst_old); // congruence删除旧指令
            congruence.add_inst(inst_new, index);
            return flag;
        }
        unreachable!()
    }
    congruence.remove_inst(inst_old); // congruence删除旧指令
    let mut index_temp = 0;
    for i in 0..bb.get_up_bb().len() {
        if dominator_tree.is_dominate(&bb, &pres[index_temp]) {
            //前继是自己所支配的，相应插的phi参数为phi本身
            pres.remove(index_temp);
            vec_index.push(i);
            continue;
        }
        index_temp += 1;
    }
    for i in 0..pres.len() {
        if let Some(inst_temp) = bb_contains_inst(pres[i], gp.clone()) {
            vec_operands_phi.push((i, inst_temp));
            continue;
        }
        if pres[i].get_next_bb().len() == 1 {
            // 只有当前块一个后继,不需要插块
            let inst_new = make_same_inst(inst_old, pool_inst);
            pres[i].get_tail_inst().as_mut().insert_before(inst_new);
            congruence.add_inst(inst_new, index); //congruence加入新指令
            vec_operands_phi.push((i, inst_new));
        } else if pres[i].get_next_bb().len() > 1 {
            // 不只有当前块一个后继,需要插块
            flag = true;
            let newb = pre_context.get_new_block(pres[i], pool_block);
            println!("插块bb:{:?}", newb.get_name());
            let inst_jmp = pool_inst.make_jmp();
            newb.as_mut().push_back(inst_jmp);
            let inst_new = make_same_inst(inst_old, pool_inst);
            newb.get_tail_inst().as_mut().insert_before(inst_new);
            congruence.add_inst(inst_new, index); //congruence加入新指令
            pres[i].as_mut().replace_next_bb(bb, newb);
            replace_bb_with_bbs(bb, pres[i], vec![newb]);
            newb.as_mut().set_next_bb(vec![bb]);
            newb.as_mut().set_up_bb(vec![pres[i]]);
            vec_operands_phi.push((i, inst_new));
        } else {
            unreachable!("前继块没后继,upbb和nextbb管理出错")
        }
    }
    insert_phi(vec_index, vec_operands_phi, pool_inst, inst_old);
    flag
}

pub fn bb_contains_inst(bb: ObjPtr<BasicBlock>, gp: Vec<ObjPtr<Inst>>) -> Option<ObjPtr<Inst>> {
    let mut map_bb = HashMap::new();
    for i in gp {
        map_bb.insert(i.get_parent_bb(), i);
    }
    map_bb.get(&bb).copied()
}

pub fn insert_phi(
    vec_index: Vec<usize>,
    vec_operands_phi: Vec<(usize, ObjPtr<Inst>)>,
    pool: &mut ObjPool<Inst>,
    inst_old: ObjPtr<Inst>,
) {
    //todo:输入参数为需要插phi的块和该块所支配前继的索引
    let inst_phi = pool.make_phi(inst_old.get_ir_type());
    let mut map_temp = HashMap::new();
    for i in vec_index {
        map_temp.insert(i, inst_phi);
    }
    for tuple in vec_operands_phi {
        map_temp.insert(tuple.0, tuple.1);
    }
    for i in 0..map_temp.len() {
        inst_phi
            .as_mut()
            .add_operand(map_temp.get(&i).unwrap().clone())
    }
    inst_old.get_parent_bb().as_mut().push_front(inst_phi);
    // println!("插phi bb:{:?}", inst_phi.get_parent_bb().get_name());
    replace_inst(inst_old, inst_phi);
    // println!("phi ops:{:?}", inst_phi.get_operands().len());
    // println!("bb up :{:?}", inst_phi.get_parent_bb().get_up_bb().len());
}
