use crate::{
    ir::{
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
        module::Module,
        tools::{bfs_bb_proceess, func_process, replace_inst},
    },
    utility::{ObjPool, ObjPtr},
};

// pub fn multiple_branch_opt()

pub fn clear_block(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let pool = &mut pools.1;
    func_process(module, |_func_name, func| {
        bfs_bb_proceess(func.get_head(), |bb| {
            let mut inst = bb.get_head_inst();
            if let InstKind::Branch = inst.get_kind() {
                if inst.get_operands().len() == 0 && inst.get_next().is_tail() {
                    delete_block(bb);
                    // println!("bb:{:?}", bb.get_name());
                    while !inst.is_tail() {
                        // println!("inst:{:?}", inst.get_kind());
                        inst = inst.get_next();
                    }
                }
            }
        });
    });
    func_process(module, |_func_name, func| {
        bfs_bb_proceess(func.get_head(), |bb| {
            if !bb.is_empty() {
                let inst = bb.get_tail_inst();
                if let InstKind::Branch = inst.get_kind() {
                    let next = bb.get_next_bb();
                    if next.len() > 1 {
                        //多个后继
                        if next[0] == next[1] {
                            //两个后继相同
                            let inst_new = pool.make_jmp(); //无条件跳转替换条件跳转
                            inst.as_mut().insert_before(inst);
                            replace_inst(inst, inst_new);
                            bb.as_mut().set_next_bb(vec![next[0]]); //设置后继bb
                            next[0].as_mut().remove_up_bb(bb); //在后继节点中清理第一个符合要求的up_bb
                        }
                    }
                }
            }
        });
    });
}
pub fn delete_block(bb: ObjPtr<BasicBlock>) {
    let up = bb.get_up_bb().clone();
    let next = bb.get_next_bb();
    if up.len() == 0 || !check_delete(next[0], bb, up.clone()) {
        return;
    }
    // println!("删除空块:{:?}", bb.get_name());
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
