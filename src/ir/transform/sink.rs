use crate::{ir::{module::Module, tools::{func_process, bfs_inst_process, bfs_bb_proceess, inst_process_in_bb, replace_inst}, analysis::dominator_tree::{calculate_dominator, self, DominatorTree}, instruction::Inst, basicblock::BasicBlock}, utility::{ObjPtr, ObjPool}};

use super::gvn_hoist::make_same_inst;

pub fn sink(module: &mut Module,pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
    func_process(module, |_, func| {
        let dominator_tree = calculate_dominator(func.get_head());
        bfs_bb_proceess(func.get_head(), |bb| {
            inst_process_in_bb(bb.get_head_inst(), |inst|{
                let use_list = inst.get_use_list();
                let mut flag = false;
                for user in use_list{
                    if user.get_parent_bb()==bb{
                        flag = true;
                        break;
                    }
                }
                if !flag{
                    sink_inst(pools.1, &dominator_tree, inst, bb, use_list.clone());
                }
            })
        });
    });
}

pub fn sink_inst(pool: &mut ObjPool<Inst>,dominator_tree: & DominatorTree,inst:ObjPtr<Inst>,bb:ObjPtr<BasicBlock>,use_list:Vec<ObjPtr<Inst>>){
    let nexts = bb.get_next_bb();
    for next in nexts{// 后继节点中，若有节点满足其前继节点由自己支配，则该节点为循环头，不应将指令下沉，导致多次计算同一值
        let next_ups = next.get_up_bb();
        for next_up in next_ups{
            if dominator_tree.is_dominate(next, next_up){
                return;
            }
        }
    }
    for next in nexts{
        let mut vec_user = vec![];
        for user in &use_list{// 将该指令的use_list分类，构建新的下沉指令，并按user分类分别设置新的use_list
            if dominator_tree.is_dominate(next, &user.get_parent_bb()){
                vec_user.push(*user);
            }
        }
        if vec_user.len()>0{// 等于0代表这一节点的支配树中根本没有指令使用了这条指令，所以不用插
            let inst_temp = find_first_nophi_inst(*next);
            let inst_new = make_same_inst(inst, pool);
            inst_temp.as_mut().insert_before(inst_new);
            for user in vec_user {
                let index = user.get_operand_index(inst);
                user.as_mut().set_operand(inst_new, index);
            }
        }
    }
    inst.as_mut().remove_self();
}

pub fn find_first_nophi_inst(bb:ObjPtr<BasicBlock>)->ObjPtr<Inst>{
    let mut ret_inst = bb.get_head_inst();
    inst_process_in_bb(bb.get_head_inst(), |inst|{
        if !inst.is_phi(){
            ret_inst = inst;
            return;
        }
    });
    ret_inst
}