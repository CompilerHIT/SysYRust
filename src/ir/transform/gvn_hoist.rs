use crate::{
    ir::{
        analysis::dominator_tree::{calculate_dominator, DominatorTree},
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
        module::Module,
        tools::{dfs_pre_order_bb_process, func_process, inst_process_in_bb, replace_inst, bfs_bb_proceess},
    },
    utility::{ObjPool, ObjPtr},
};

use super::global_value_numbering::{self, compare_two_inst, Congruence, CongruenceClass};

pub fn hoist(
    module: &mut Module,
    opt_option: bool,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    if opt_option {
        let mut vec_congruence_class = global_value_numbering::gvn(module, opt_option).unwrap();
        let mut index = 0;
        func_process(module, |_, func| {
            let dominator_tree = calculate_dominator(func.get_head());
            loop {
                let mut changed = false;
                dfs_pre_order_bb_process(func.get_head(), |bb| {
                    // dfs后序遍历bb,自底向上上提指令
                    let next = bb.get_next_bb().clone();
                    if next.len() == 1 {
                        // 只有这一个后继
                        if dominator_tree.is_dominate(&bb, &next[0]) {
                            // 如果支配这个后继，就把后继中的指令上提
                            if next[0].get_up_bb().len() == 1 {
                                // 如果这个后继也只有当前块一个前继块,无条件上移指令
                                move_insts(bb, &mut vec_congruence_class[index], pools.1);
                            } else {
                                // 如果不只有当前块一个前继，那么则选择性的上移指令，这里可能需要留意一下
                                move_insts_select(bb, &mut vec_congruence_class[index], pools.1);
                            }
                        }
                    } else if next.len() > 1 {
                        // 两个后继
                        changed |= check_successor(
                            bb,
                            next,
                            &mut vec_congruence_class[index],
                            pools.1,
                            &dominator_tree,
                        );
                    }
                });
                if !changed {
                    break;
                }
            }
            index += 1;
        });
    }
}

pub fn hoist_group(
    congruence: &mut Congruence,
    index_gp: usize,
    dominator_tree: &DominatorTree,
    pool: &mut ObjPool<Inst>,
) -> bool {
    let mut flag = false;
    loop {
        let mut flag2 = false;
        for i in 0..congruence.vec_class[index_gp].len() {
            let bb1 = congruence.vec_class[index_gp][i].get_parent_bb();
            let ups1 = bb1.get_up_bb();
            for up1 in ups1 {
                if dominator_tree.is_dominate(up1, &bb1) && up1.get_next_bb().len() == 1 {
                    // 若前继块支配该块且只有当前块一个后继，则将指令移动到该前继中
                    let inst1 = congruence.vec_class[index_gp][i];
                    let mut tttflag = false;
                    for op in inst1.get_operands() {
                        //operand在当前块，不移动
                        if !op.is_global_var_or_param() {
                            if op.get_parent_bb() == bb1 {
                                tttflag = true;
                                break;
                            }
                        }
                    }
                    if tttflag {
                        // 该指令所在块的其他前继块不可能也支配当块，直接退出这条指令的上提判断
                        break;
                    }
                    // 满足条件,上移
                    let tail = up1.get_tail_inst();
                    let inst_new = make_same_inst(inst1, pool);
                    congruence.add_inst(inst_new, index_gp);
                    congruence.remove_inst(inst1);
                    tail.as_mut().insert_before(inst_new);
                    replace_inst(inst1, inst_new);
                    flag2 = true;
                    flag = true;
                    break;
                }
            }
            if flag2 {
                break;
            }
        }
        if flag2 {
            // 指令集变了，下一次循环
            continue;
        }
        for i in 0..congruence.vec_class[index_gp].len() {
            for j in i + 1..congruence.vec_class[index_gp].len() {
                let bb1 = congruence.vec_class[index_gp][i].get_parent_bb();
                let bb2 = congruence.vec_class[index_gp][j].get_parent_bb();
                if bb1 == bb2 {
                    continue;
                }
                let ups1 = bb1.get_up_bb();
                let ups2 = bb2.get_up_bb();
                for up1 in ups1 {
                    for up2 in ups2 {
                        if up1 == up2
                            && dominator_tree.is_dominate(up1, &bb1)
                            && dominator_tree.is_dominate(up1, &bb2)
                        {
                            let inst1 = congruence.vec_class[index_gp][i];
                            let inst2 = congruence.vec_class[index_gp][j];
                            let tail = up1.get_tail_inst();
                            let inst_new = make_same_inst(inst1, pool);
                            congruence.add_inst(inst_new, index_gp);
                            congruence.remove_inst(inst1);
                            congruence.remove_inst(inst2);
                            tail.as_mut().insert_before(inst_new);
                            replace_inst(inst1, inst_new);
                            replace_inst(inst2, inst_new);
                            flag2 = true;
                            flag = true;
                            break;
                        }
                    }
                    if flag2 {
                        break;
                    }
                }
                if flag2 {
                    break;
                }
            }
            if flag2 {
                break;
            }
        }
        // todo:如果指令所在块只有一个upbb,且upbb只有该块一个后继块，则应该把这条指令上提
        if !flag2 {
            break;
        }
    }
    flag
}

pub fn hoist_to_loop_head(module: &mut Module, pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
    let mut index = 0;
    func_process(module, |_, func| {
        let dominator_tree = calculate_dominator(func.get_head());
        bfs_bb_proceess(func.get_head(), |bb| {
            let ups = bb.get_up_bb();
            let mut vec_loop_end = vec![];
            for up in ups{
                if dominator_tree.is_dominate(&bb, up){
                    vec_loop_end.push(up);
                }
            }
            // 先不考虑复杂的情况，仅考虑一条回边的情况
            if vec_loop_end.len()==1{
                inst_process_in_bb(vec_loop_end[0].get_head_inst(), |inst| {
                    hoist_loop_inst(pools.1,inst,bb,&dominator_tree)
                })
            }
        });
        index +=1;
    });
}

pub fn hoist_loop_inst(
    pool: &mut ObjPool<Inst>,
    inst: ObjPtr<Inst>,//up中的inst
    bb_now: ObjPtr<BasicBlock>,
    dominator_tree: &DominatorTree,
){
    match inst.get_kind() {
        InstKind::Return |InstKind::Alloca(_)|InstKind::Branch|InstKind::Call(_)|InstKind::Load|InstKind::Parameter|InstKind::Phi|InstKind::Store =>{
            //保险起见,这些指令不移动  
            // return;  
        }
        _=>{
            // 判断指令是否可造
            // 创建新指令替换旧指令，修改uselist
            let oops = inst.get_operands();
            let mut flag = true;
            for oop in oops{
                if oop.is_global_var_or_param(){
                    continue;
                }
                if !dominator_tree.is_dominate(&oop.get_parent_bb(), &bb_now){
                    flag = false;
                    break;
                }
            }
            if flag{
                let inst_temp = bb_now.get_tail_inst();
                let inst_new = make_same_inst(inst, pool);
                inst_temp.as_mut().insert_before(inst_new);
                replace_inst(inst, inst_new);
                // return;
            }
        }
    }
}

pub fn check_successor(
    bb: ObjPtr<BasicBlock>,
    vec_successors: Vec<ObjPtr<BasicBlock>>,
    congruence_class: &mut CongruenceClass,
    pool: &mut ObjPool<Inst>,
    dominator_tree: &DominatorTree,
) -> bool {
    let bb1 = vec_successors[0];
    let bb2 = vec_successors[1];
    let mut flag = false;
    inst_process_in_bb(bb1.get_head_inst(), |inst1| {
        inst_process_in_bb(bb2.get_head_inst(), |inst2| {
            // 遍历过程中删除指令应该不会有问题
            if dominator_tree.is_dominate(&bb, &bb1) && dominator_tree.is_dominate(&bb, &bb2) {
                // 如果当前节点支配其两个后继，则可以考虑将相同计算提到当前节点中
                if compare_two_inst(inst1, inst2, congruence_class) {
                    let tail = bb.get_tail_inst();
                    let inst_new = make_same_inst(inst1, pool);
                    congruence_class.add_inst(inst_new);
                    congruence_class.remove_inst(inst1);
                    congruence_class.remove_inst(inst2);
                    tail.as_mut().insert_before(inst_new);
                    replace_inst(inst1, inst_new);
                    replace_inst(inst2, inst_new);
                    flag = true;
                }
            }
        })
    });
    flag
}

pub fn move_insts_select(
    bb: ObjPtr<BasicBlock>,
    congruence_class: &mut CongruenceClass,
    pool: &mut ObjPool<Inst>,
) -> bool {
    let next = bb.get_next_bb()[0];
    let mut flag = false;
    let tail = bb.get_tail_inst();
    inst_process_in_bb(next.get_head_inst(), |inst| {
        match inst.get_kind() {
            InstKind::Alloca(_)
            | InstKind::Branch
            | InstKind::Head
            | InstKind::Parameter
            | InstKind::Return
            | InstKind::Store
            | InstKind::Load
            | InstKind::GlobalConstFloat(_)
            | InstKind::GlobalConstInt(_)
            | InstKind::GlobalFloat(_)
            | InstKind::GlobalInt(_)
            | InstKind::Phi => {
                return;
            }
            _ => {}
        }
        for operand in inst.get_operands() {
            if operand.is_global_var_or_param() {
                continue;
            }
            if operand.get_parent_bb() == next {
                return;
            }
        }
        let inst_new = make_same_inst(inst, pool);
        congruence_class.add_inst(inst_new);
        congruence_class.remove_inst(inst);
        tail.as_mut().insert_before(inst_new);
        replace_inst(inst, inst_new);
        flag = true;
    });
    flag
}

//todo:只有一个后继的情况也应该上提，其他优化结束之后再下沉，不考虑后继只有一个且是自己的情况
pub fn move_insts(
    bb: ObjPtr<BasicBlock>,
    congruence_class: &mut CongruenceClass,
    pool: &mut ObjPool<Inst>,
) -> bool {
    let next = bb.get_next_bb()[0];
    let mut flag = false;
    let tail = bb.get_tail_inst();
    inst_process_in_bb(next.get_head_inst(), |inst| {
        if !inst.is_phi() && !inst.is_br() {
            for operand in inst.get_operands() {
                if operand.is_global_var_or_param() {
                    continue;
                }
                if operand.get_parent_bb() == next {
                    return;
                }
            }
        }
        let inst_new = make_same_inst(inst, pool);
        congruence_class.add_inst(inst_new);
        congruence_class.remove_inst(inst);
        tail.as_mut().insert_before(inst_new);
        replace_inst(inst, inst_new);
        flag = true;
    });
    flag
}

pub fn make_same_inst(inst_old: ObjPtr<Inst>, pool: &mut ObjPool<Inst>) -> ObjPtr<Inst> {
    let ir_type = inst_old.as_ref().get_ir_type();
    let kind = inst_old.get_kind().clone();
    let operands = inst_old.get_operands().clone();
    let inst_new = pool.put(Inst::new(ir_type, kind, operands));
    for i in inst_new.get_operands() {
        i.as_mut().add_user(inst_new.as_ref());
    }
    inst_new
}
