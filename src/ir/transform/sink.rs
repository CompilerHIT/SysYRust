use crate::{
    ir::{
        analysis::{dominator_tree::{calculate_dominator, DominatorTree}, downstream_tree::{self, DownStreamTree}},
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
        module::Module,
        tools::{bfs_bb_proceess, func_process, inst_process_in_bb_reverse, inst_process_in_bb, replace_inst},
    },
    utility::{ObjPool, ObjPtr},
};

use super::{gvn_hoist::make_same_inst, global_value_numbering::{CongruenceClass, compare_two_inst, self}};

pub fn sink(module: &mut Module, pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)) {
    func_process(module, |_, func| {
        let dominator_tree = calculate_dominator(func.get_head());
        bfs_bb_proceess(func.get_head(), |bb| {
            inst_process_in_bb_reverse(bb.get_tail_inst(), |inst| {
                let use_list = inst.get_use_list();
                let mut flag = false;
                for user in use_list {
                    if user.get_parent_bb() == bb {
                        flag = true;
                        break;
                    }
                }
                if !flag && use_list.len() != 0 {
                    // 没有user在当前块且至少有一个user(没user肯定不能移动)
                    sink_inst(pools.1, &dominator_tree, inst, bb, use_list.clone());
                }
            })
        });
    });
}

pub fn sink_inst(
    pool: &mut ObjPool<Inst>,
    dominator_tree: &DominatorTree,
    inst: ObjPtr<Inst>,
    bb: ObjPtr<BasicBlock>,
    use_list: Vec<ObjPtr<Inst>>,
) {
    match inst.get_kind() {
        InstKind::Alloca(_)
        | InstKind::Branch
        | InstKind::Call(_)
        | InstKind::Load
        | InstKind::Parameter
        | InstKind::Phi
        | InstKind::Return
        | InstKind::Store => {
            return;
        } // 这类指令不能随意移动位置
        _ => {}
    }
    let nexts = bb.get_next_bb();
    let mut vec_vec_user = vec![]; // 用于将user分类,绑定不同的后继块
    let mut len = 0;
    for next in nexts {
        let mut vec_user = vec![]; // 用于装载当前后继块对应的新绑定user
        for user in &use_list {
            // 将该指令的use_list分类，构建新的下沉指令，并按user分类分别设置新的use_list
            if dominator_tree.is_dominate(next, &user.get_parent_bb()) {
                // 该后继节点支配当前user所在块
                if user.is_phi() {
                    //如果有user是phi,且下一个节点就是phi所在的节点,则不下沉指令
                    if *next == user.get_parent_bb() {
                        return;
                    }
                }
                vec_user.push(*user);
                len += 1;
            }
        }
        vec_vec_user.push(vec_user);
    }
    if len < use_list.len() {
        // 有的user所在块仅由bb支配而不由任何bb的任何后继块所支配
        return;
    }

    for next in nexts {
        // 后继节点中，若有节点满足其前继节点由自己支配，且这个节点对应的支配树中有用到这条指令，则该节点为循环头，不应将指令下沉，导致多次计算同一值
        let next_ups = next.get_up_bb();
        for next_up in next_ups {
            if dominator_tree.is_dominate(next, next_up) {
                // 后继节点中，若有节点满足其前继节点由自己支配
                for user in &use_list {
                    if dominator_tree.is_dominate(next, &user.get_parent_bb()) {
                        // 这个节点对应的支配树中有用到这条指令
                        return;
                    }
                }
            }
        }
    }

    let mut index_next = 0;
    for next in nexts {
        if vec_vec_user[index_next].len() > 0 {
            // 等于0代表这一节点的支配树中根本没有指令使用了这条指令，所以不用插
            let inst_temp = find_first_nophi_inst(*next);
            let inst_new = make_same_inst(inst, pool);
            inst_temp.as_mut().insert_before(inst_new);
            for user in vec_vec_user[index_next].clone() {
                let index = user.get_operand_index(inst);
                user.as_mut().set_operand(inst_new, index);
            }
        }
        index_next += 1;
    }
    inst.as_mut().remove_self();
}

pub fn sink_opt(module: &mut Module, pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),opt_option: bool) {
    let vec_congruence_class = global_value_numbering::gvn(module, opt_option).unwrap();
    let mut index = 0;
    func_process(module, |_, func| {
        let dominator_tree = calculate_dominator(func.get_head());
        let downstream_tree = DownStreamTree::make_downstream_tree(func.get_head(), &dominator_tree);
        bfs_bb_proceess(func.get_head(), |bb| {
            inst_process_in_bb(bb.get_head_inst(), |inst| {
                if inst.is_phi(){
                    sink_inst_opt(pools.1, &dominator_tree, &downstream_tree, inst, bb, &vec_congruence_class[index])
                }
            })
        });
        index +=1;
    });
}

pub fn sink_inst_opt(
    pool: &mut ObjPool<Inst>,
    dominator_tree: &DominatorTree,
    downstream_tree: &DownStreamTree,
    inst: ObjPtr<Inst>,
    bb: ObjPtr<BasicBlock>,
    congruence_class:& CongruenceClass
){
    let ups = bb.get_up_bb();
    // todo: phi中每一个都同质
    let ops = inst.get_operands();

    let len = ups.len();
    for up in ups{// 前继节点中有多于一个后继则退出
        if up.get_next_bb().len()>1{
            return;
        }
        if !downstream_tree.is_upstream(*up, bb){//有前继不是上游(当前节点是循环头)
            return;
        }
    }
    for i in 0..len{
        for j in i+1..len{
            if !compare_two_inst(ops[i], ops[j], congruence_class){
                return;
            }
        }
    }// 有不同质的则退出
    for i in 0..len{
        if ops[i].is_global_var_or_param(){
            return;
        }
        if !(ops[i].get_parent_bb()==ups[i]){//有不在前继定义的退出
            return;
        }
    }
    //todo:判断uselist
    for op in ops{
        if op.is_global_var_or_param(){
            return;
        }
        for user in op.get_use_list(){
            if user.get_parent_bb()==op.get_parent_bb(){
                return;
            }
        }
    }
    // 寻找有无当前块可造的指令
    for op in ops{
        // todo:判断是否可造
        // todo:修改uselist
        // 插入新指令
        let oops = op.get_operands();
        let mut flag = true;
        for oop in oops{
            if oop.is_global_var_or_param(){
                continue;
            }
            if !dominator_tree.is_dominate(&oop.get_parent_bb(), &bb){
                flag = false;
                break;
            }
        }
        if flag{
            let inst_temp = find_first_nophi_inst(bb);
            let inst_new = make_same_inst(*op, pool);
            inst_temp.as_mut().insert_before(inst_new);
            replace_inst(inst, inst_new);
            return;
        }
    }
}

pub fn find_first_nophi_inst(bb: ObjPtr<BasicBlock>) -> ObjPtr<Inst> {
    let mut ret_inst = bb.get_head_inst();
    while !ret_inst.is_tail() {
        // 这里需要先获取next，因为predicate可能会删除当前指令
        let next = ret_inst.get_next();
        if !ret_inst.is_phi() {
            return ret_inst;
        }
        ret_inst = next;
    }
    ret_inst
}
