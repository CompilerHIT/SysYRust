use std::collections::HashMap;

use crate::{ir::{module::Module, basicblock::BasicBlock, analysis::{downstream_tree::{ DownStreamTree, self}, dominator_tree::{calculate_dominator, self, DominatorTree}}, tools::func_process, instruction::Inst}, utility::{ObjPtr, ObjPool}};

use super::{global_value_numbering::{self, Congruence}, hoist::make_same_inst, delete_empty_block::replace_bb_with_bbs};

pub struct PreContext{
    index:i32,
}

impl PreContext{
    pub fn get_new_block(&mut self, bb_old: ObjPtr<BasicBlock>, pool: &mut ObjPool<BasicBlock>) ->ObjPtr<BasicBlock>{
        let name_old =bb_old.get_name().to_string();
        let name_new = name_old+"_pre_"+self.index.to_string().as_str();
        pool.new_basic_block(name_new)
    }
}

pub fn pre(module: &mut Module, opt_option: bool,pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
    let mut congruence_class = global_value_numbering::gvn(module,opt_option).unwrap();
    func_process(module, |_, func| {
        for congruence in congruence_class.get_all_congruence_mut(){
            pre_congruence(congruence, func.get_head(), pools);
        }
    });
}

pub fn pre_congruence(congruence: &mut Congruence,head:ObjPtr<BasicBlock>,pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
    let mut pre_context = PreContext{index:0};
    let mut downstream_tree = DownStreamTree::make_downstream_tree(head);
    let mut dominator_tree = calculate_dominator(head);
    for gp in &congruence.vec_class{
        let mut flag = false;
        loop{
            for i in 0..gp.len(){
                for j in 0..gp.len(){
                    if i==j{
                        continue;
                    }else{
                        if downstream_tree.is_upstream(gp[i].get_parent_bb(), gp[j].get_parent_bb()){
                            flag |=true;
                            if insert_inst_in_pre(gp.clone(), &mut pre_context, gp[j], &dominator_tree, pools.1, pools.0){
                                downstream_tree = DownStreamTree::make_downstream_tree(head);
                                dominator_tree = calculate_dominator(head);
                            }
                            break;//替换过指令，刷新，从头开始比较
                        }
                    }
    
                }
                if flag{//替换过指令，刷新，从头开始比较
                    break;
                }
            }
            if !flag{
                break;
            }
        }
        
    }
}

pub fn insert_inst_in_pre(gp: Vec<ObjPtr<Inst>>, pre_context: &mut PreContext,inst_old:ObjPtr<Inst>, dominator_tree: &DominatorTree, pool_inst: &mut ObjPool<Inst>, pool_block: &mut ObjPool<BasicBlock>)->bool{
    let bb = inst_old.get_parent_bb();
    let mut pres = bb.get_up_bb().clone();
    let mut vec_index = vec![];// 记录自己所支配前继的索引
    let mut vec_operands_phi = vec![];
    for i in 0..bb.get_up_bb().len(){
        if dominator_tree.is_dominate(&bb, &pres[i]){//前继是自己所支配的，相应插的phi参数为phi本身
            pres.remove(i);
            vec_index.push(i);
        }
    }
    let mut flag = false;
    for i in 0..pres.len(){
        if let Some(inst_temp) = bb_contains_inst(pres[i], gp.clone()){
            vec_operands_phi.push((i,inst_temp));
            continue;
        }
        if pres[i].get_next_bb().len() == 1 {// 只有当前块一个后继,不需要插块
            let inst_new = make_same_inst(inst_old, pool_inst);
            pres[i].get_tail_inst().as_mut().insert_before(inst_new);
            vec_operands_phi.push((i,inst_new));
        }else if pres[i].get_next_bb().len() >1{// 不只有当前块一个后继,需要插块
            flag = true;
            let newb = pre_context.get_new_block(pres[i], pool_block);
            let inst_jmp = pool_inst.make_jmp();
            newb.as_mut().push_back(inst_jmp);
            let inst_new = make_same_inst(inst_old, pool_inst);
            newb.get_tail_inst().as_mut().insert_before(inst_new);
            pres[i].as_mut().replace_next_bb(bb, newb);
            replace_bb_with_bbs(bb, pres[i], vec![newb]);
            vec_operands_phi.push((i,inst_new));
        }else {
            unreachable!("前继块没后继,upbb和nextbb管理出错")
        }
    }
    insert_phi(bb, vec_index,vec_operands_phi,pool_inst,inst_old);
    flag

}

pub fn bb_contains_inst(bb:ObjPtr<BasicBlock>,gp: Vec<ObjPtr<Inst>>)->Option<ObjPtr<Inst>>{
    let mut map_bb = HashMap::new();
    for i in gp{
        map_bb.insert(i.get_parent_bb(),i);
    }
    map_bb.get(&bb).copied()
}

pub fn insert_phi(bb:ObjPtr<BasicBlock>,vec_index:Vec<usize>,vec_operands_phi:Vec<(usize,ObjPtr<Inst>)>,pool: &mut ObjPool<Inst>,inst_old:ObjPtr<Inst>){//todo:输入参数为需要插phi的块和该块所支配前继的索引
    let inst_phi = pool.make_phi(inst_old.get_ir_type());
    let mut map_temp = HashMap::new();
    for i in vec_index{
        map_temp.insert(i, inst_phi);
    }
    for tuple in vec_operands_phi{
        map_temp.insert(tuple.0, tuple.1);
    }
    for i in 0..map_temp.len(){
        inst_phi.as_mut().add_operand(map_temp.get(&i).unwrap().clone())
    }
}