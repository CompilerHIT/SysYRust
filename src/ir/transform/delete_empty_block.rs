use crate::{
    ir::{
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
        module::Module,
        tools::{bfs_bb_proceess, func_process, replace_inst, inst_process_in_bb},
    },
    utility::{ObjPool, ObjPtr},
};

// 把所有常量移动到头块
pub fn move_const_to_head(head:ObjPtr<BasicBlock>,pool: &mut ObjPool<Inst>){
    bfs_bb_proceess(head, |bb| {
        inst_process_in_bb(bb.get_head_inst(), |inst|{
            if bb!=head{
                match inst.get_kind() {
                    InstKind::ConstFloat(f) =>{
                        let const_float = pool.make_float_const(f);
                        head.as_mut().push_front(const_float);
                        replace_inst(inst, const_float);
                    }
                    InstKind::ConstInt(i) =>{
                        let const_int = pool.make_int_const(i);
                        head.as_mut().push_front(const_int);
                        replace_inst(inst, const_int);
                    }
                    _=>{}
                }
            }
        })
    })
}

// 有条件跳转(两个后继为同一块)转化为无条件跳转,清理块间关系
pub fn multiple_branch_opt(bb: ObjPtr<BasicBlock>,pool: &mut ObjPool<Inst>) ->bool{
    if !bb.is_empty() {
        let inst = bb.get_tail_inst();
        if inst.is_br() {
            if inst.is_br_cond(){
                let next = bb.get_next_bb();
                //多个后继
                if next[0] == next[1] {
                    //两个后继相同
                    let inst_new = pool.make_jmp(); //无条件跳转替换条件跳转
                    inst.as_mut().insert_before(inst_new);
                    replace_inst(inst, inst_new);
                    bb.as_mut().set_next_bb(vec![next[0]]); //设置后继bb
                    next[0].as_mut().remove_up_bb(bb); //在后继节点中清理第一个符合要求的up_bb
                    return true;
                }
            }
        }
    }
    false
}

// 删除仅含一条无条件跳转指令的块
pub fn clear_block(bb: ObjPtr<BasicBlock>){
    let mut inst = bb.get_head_inst();
    if let InstKind::Branch = inst.get_kind() {
        if inst.get_operands().len() == 0 && inst.get_next().is_tail() {
            delete_block(bb);
            while !inst.is_tail() {
                inst = inst.get_next();
            }
        }
    }
}

pub fn block_opt(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let pool = &mut pools.1;
    func_process(module, |_func_name, func| {
        move_const_to_head(func.get_head(), pool);
    });
    loop{
        func_process(module, |_func_name, func| {
            bfs_bb_proceess(func.get_head(), |bb| {
                clear_block(bb)
            });
        });
        let mut flag = false;
        func_process(module, |_func_name, func| {
            bfs_bb_proceess(func.get_head(), |bb| {
                flag |= multiple_branch_opt(bb, pool)
            });
        });
        if !flag{
            break;
        }
    }
}

pub fn delete_block(bb: ObjPtr<BasicBlock>) {
    let up = bb.get_up_bb().clone();
    let next = bb.get_next_bb();
    if up.len() == 0 || !check_delete(next[0], bb, up.clone()) {
        return;
    }
    for i in 0..up.clone().len() {
        up[i].as_mut().replace_next_bb(bb, next[0]);
    }

    replace_bb_with_bbs(next[0], bb, up)
}

pub fn replace_bb_with_bbs(
    bb: ObjPtr<BasicBlock>,
    bb_old: ObjPtr<BasicBlock>,
    bb_new: Vec<ObjPtr<BasicBlock>>,
) {
    let mut ups = bb.get_up_bb().clone();
    let len = bb_new.len();
    let index = ups.iter().position(|x| *x == bb_old).unwrap(); //找到旧bb位置
    ups.remove(index); //删除旧bb
    for i in 0..len {
        ups.push(bb_new[i]); //连接新的前继
    }
    bb.as_mut().set_up_bb(ups);
    let mut inst = bb.get_head_inst();
    while let InstKind::Phi = inst.as_ref().get_kind() {
        let operand = inst.get_operand(index);
        inst.remove_operand_by_index(index);
        for _ in 0..len {
            inst.add_operand(operand);
        }
        inst = inst.get_next();
    }
}

pub fn check_delete(
    bb: ObjPtr<BasicBlock>,
    bb_old: ObjPtr<BasicBlock>,
    bb_new: Vec<ObjPtr<BasicBlock>>,
) -> bool {
    let ups = bb.get_up_bb();
    let index = ups.iter().position(|x| *x == bb_old).unwrap(); //找到旧bb位置
    let len = bb_new.len();
    //相同前继组比较参数是否相同，若参数组不完全相同则不删除该节点
    let mut inst = bb.get_head_inst();
    while let InstKind::Phi = inst.as_ref().get_kind() {
        let operand = inst.get_operand(index);
        for i in 0..ups.len() {
            if i == index {
                continue;
            }
            for j in 0..len {
                if ups[i] == bb_new[j] {
                    if operand != inst.get_operand(i) {
                        return false;
                    }
                }
            }
        }
        inst = inst.get_next();
    }
    true
}
