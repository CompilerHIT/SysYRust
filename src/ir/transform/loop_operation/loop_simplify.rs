use crate::{
    ir::{
        analysis::loop_tree::{LoopInfo, LoopList},
        basicblock::BasicBlock,
        instruction::{Inst, InstKind},
    },
    utility::{ObjPool, ObjPtr},
};

pub fn loop_simplify_run(
    loop_list: &mut LoopList,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    for loop_info in loop_list.get_loop_list() {
        simplify_one_loop(loop_info.clone(), pools);
    }
}

fn simplify_one_loop(
    loop_info: ObjPtr<LoopInfo>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    add_preheader(loop_info, pools);
}

fn add_preheader(
    mut loop_info: ObjPtr<LoopInfo>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut header = loop_info.get_header();
    let mut preheader = pools
        .0
        .new_basic_block(format!("preheader_{}", header.get_name()));
    loop_info.set_pre_header(preheader);

    // 获得不在循环中的前继块
    let mut up_bb_list: Vec<(usize, ObjPtr<BasicBlock>)> = header
        .get_up_bb()
        .iter()
        .cloned()
        .enumerate()
        .filter(|(_, bb)| !loop_info.is_in_loop(bb))
        .collect();
    up_bb_list.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // 将header中的phi拆成两部分：一部分为循环外的变量，一部分为循环自身的和preheader中的phi
    let mut inst = header.get_head_inst();
    let indexs: Vec<usize> = up_bb_list.iter().map(|(index, _)| index.clone()).collect();
    while let InstKind::Phi = inst.get_kind() {
        let operands: Vec<ObjPtr<Inst>> = inst
            .get_operands()
            .iter()
            .enumerate()
            .filter(|(index, _)| indexs.contains(index))
            .map(|(_, operand)| operand.clone())
            .collect();

        if operands.len() != 0 {
            let phi = pools.1.make_phi_with_operands(inst.get_ir_type(), operands);
            preheader.push_back(phi);
            inst.add_operand(phi);
        }
        inst = inst.get_next();
    }
    preheader.push_back(pools.1.make_jmp());

    for (_, mut up_bb) in up_bb_list {
        up_bb.replace_next_bb(header, preheader);
        preheader.add_up_bb(&up_bb);
        header.remove_up_bb(up_bb);
    }

    preheader.add_next_bb(header);
}
