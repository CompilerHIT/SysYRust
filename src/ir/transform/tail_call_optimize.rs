use std::collections::HashSet;

use crate::{
    ir::{
        analysis::dominator_tree::calculate_dominator, function::Function, instruction::InstKind,
    },
    utility::ObjPtr,
};

use super::*;
pub fn tail_call_optimize(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    func_process(module, |name, func| {
        if func.get_return_type().is_void() {
            let mut self_call_vec = Vec::new();
            let dom_tree = calculate_dominator(func.get_head());
            bfs_inst_process(func.get_head(), |inst| {
                if let InstKind::Call(callee) = inst.get_kind() {
                    if callee == name
                        && inst.get_next().is_br()
                        && inst.get_next().is_br_jmp()
                        && inst.get_parent_bb().get_next_bb()[0].is_exit()
                        && !dom_tree.is_dominate(
                            &inst.get_parent_bb(),
                            &inst.get_parent_bb().get_next_bb()[0],
                        )
                    {
                        self_call_vec.push(inst);
                    }
                }
            });

            if self_call_vec.len() != 0 {
                tail_call_to_loop(&name, func, pools, self_call_vec);
            }
        }
    })
}

fn tail_call_to_loop(
    name: &str,
    mut func: ObjPtr<Function>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    mut self_call_vec: Vec<ObjPtr<Inst>>,
) {
    let mut pre_func = pools.0.new_basic_block(format!("pre_{}", name));
    let mut func_header = func.get_head();
    pre_func.push_back(pools.1.make_jmp());
    pre_func.add_next_bb(func_header);
    func.set_head(pre_func);

    for caller in self_call_vec.iter() {
        let mut parent_bb = caller.get_parent_bb();
        let next_bb = parent_bb.get_next_bb()[0].clone();
        parent_bb.remove_next_bb(next_bb);
        parent_bb.add_next_bb(func_header);
    }

    let params = func.get_parameter_list().clone();

    for i in 0..params.len() {
        let param = params[i].clone();

        let ir_type = param.get_ir_type();
        if ir_type.is_pointer() {
            continue;
        }

        let mut phi = pools.1.make_phi(param.get_ir_type());

        param.get_use_list().clone().iter_mut().for_each(|user| {
            let index = user.get_operand_index(param);
            user.set_operand(phi, index);
        });

        phi.add_operand(param);
        self_call_vec.iter_mut().for_each(|caller| {
            phi.add_operand(caller.get_operand(i));
        });
        func_header.push_front(phi);
    }

    self_call_vec
        .iter_mut()
        .for_each(|caller| caller.remove_self());
}
