use crate::{ir::{module::Module, tools::{func_process, inst_process_in_bb, dfs_pre_order_bb_process, replace_inst}, basicblock::BasicBlock, instruction::{Inst, InstKind, BinOp, UnOp}, ir_type::IrType, analysis::dominator_tree::{self, calculate_dominator, DominatorTree}}, utility::{ObjPtr, ObjPool}};

use super::global_value_numbering::{self, CongruenceClass, compare_two_inst, Congruence};

pub fn hoist(module: &mut Module, opt_option: bool,pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
    if opt_option{
        let mut vec_congruence_class = global_value_numbering::gvn(module,opt_option).unwrap();
        let mut index = 0;
        func_process(module, |_, func| {
            let dominator_tree = calculate_dominator(func.get_head());
            loop {
                let mut changed = false;
                dfs_pre_order_bb_process(func.get_head(), |bb| {
                    let next = bb.get_next_bb().clone();
                    if next.len()==1{
                        move_insts(bb, &mut vec_congruence_class[index], pools.1);
                    }else if next.len()>1{
                        changed |= check_successor(bb,next,&mut vec_congruence_class[index],pools.1,&dominator_tree);
                    }
                });
                if !changed{
                    break;
                }
            }
            index +=1;
        });
    }
}

// pub fn hoist_inst(head:ObjPtr<BasicBlock>,dominator_tree: & DominatorTree,mut index_gp:usize,vec_congruence_class:&mut CongruenceClass,module: &mut Module,pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)){
//     loop {
//         let mut changed = false;
        
//         if !changed{
//             break;
//         }
//     }
// }

pub fn hoist_group(congruence:&mut Congruence,mut index_gp:usize,dominator_tree: & DominatorTree,pool: &mut ObjPool<Inst>) ->bool{
    let mut flag = false;
    loop{
        let mut flag2 = false;
        for i in 0..congruence.vec_class[index_gp].len(){
            for j in 0..congruence.vec_class[index_gp].len(){
                let bb1 = congruence.vec_class[index_gp][i].get_parent_bb();
                let bb2 = congruence.vec_class[index_gp][j].get_parent_bb();
                if bb1==bb2{
                    continue;
                }
                let ups1 = bb1.get_up_bb();
                let ups2 = bb2.get_up_bb();
                for up1 in ups1{
                    for up2 in ups2{
                        if up1 ==up2&&dominator_tree.is_dominate(up1, &bb1)&&dominator_tree.is_dominate(up1, &bb2){
                            let inst1 = congruence.vec_class[index_gp][i];
                            let inst2 = congruence.vec_class[index_gp][j];
                            let tail = up1.get_tail_inst();
                            let inst_new =make_same_inst(inst1, pool);
                            congruence.add_inst(inst_new,index_gp);
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
                    if flag2{
                        break;
                    }
                }
                if flag2{
                    break;
                }
            }
            if flag2{
                break;
            }
        }
        if !flag2{
            break;
        }
    }
    flag
}

pub fn check_successor(bb:ObjPtr<BasicBlock>,vec_successors:Vec<ObjPtr<BasicBlock>>,congruence_class:&mut CongruenceClass,pool: &mut ObjPool<Inst>,dominator_tree:& DominatorTree)->bool{
    let bb1 = vec_successors[0];
    let bb2 = vec_successors[1];
    let mut flag = false;
    inst_process_in_bb(bb1.get_head_inst(), |inst1|{
        inst_process_in_bb(bb2.get_head_inst(), |inst2|{
            if dominator_tree.is_dominate(&bb, &bb1)&&dominator_tree.is_dominate(&bb, &bb2) {
                if compare_two_inst(inst1, inst2, congruence_class){
                    let tail = bb.get_tail_inst();
                    let inst_new =make_same_inst(inst1, pool);
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

//todo:只有一个后继的情况也应该上提，其他优化结束之后再下沉，不考虑后继只有一个且是自己的情况
pub fn move_insts(bb:ObjPtr<BasicBlock>,congruence_class:&mut CongruenceClass,pool: &mut ObjPool<Inst>) ->bool{
    let next = bb.get_next_bb()[0];
    let mut flag = false;
    let tail = bb.get_tail_inst();
    inst_process_in_bb(next.get_head_inst(), |inst|{
        if !inst.is_br(){
            let inst_new =make_same_inst(inst, pool);
            congruence_class.add_inst(inst_new);
            congruence_class.remove_inst(inst);
            tail.as_mut().insert_before(inst_new);
            replace_inst(inst, inst_new);
            flag = true;
        }
    });
    flag
}

pub fn make_same_inst(inst_old:ObjPtr<Inst>,pool: &mut ObjPool<Inst>)->ObjPtr<Inst>{
    let ir_type = inst_old.as_ref().get_ir_type();
    let kind = inst_old.get_kind().clone();
    let operands = inst_old.get_operands().clone();
    let inst_new = pool.put(Inst::new(ir_type, kind, operands));
    for i in inst_new.get_operands(){
        i.as_mut().add_user(inst_new.as_ref());
    }
    inst_new
}