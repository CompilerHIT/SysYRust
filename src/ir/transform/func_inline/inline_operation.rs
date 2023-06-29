use super::*;
pub fn inline_func(
    caller: ObjPtr<Function>,
    callee: ObjPtr<Function>,
    callee_name: &str,
    global_var: Vec<(&String, ObjPtr<Inst>)>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    bfs_inst_process(caller.get_head(), |inst| {
        if let InstKind::Call(name) = inst.get_kind() {
            if name == callee_name {
                let arg_list = inst.get_operands().clone();

                inline_func_with_inst(
                    inst,
                    callee,
                    callee_name,
                    global_var.clone(),
                    arg_list,
                    pools,
                );
            }
        }
    })
}

///  删除掉除了main函数之外不会被调用的函数
pub fn delete_uncalled_func(module: &mut Module, call_map: &mut CallMap) {
    let mut uncalled_funcs = Vec::new();
    for func_name in call_map.get_all_func() {
        if func_name != "main" && call_map.find_predecessors(&func_name).is_empty() {
            uncalled_funcs.push(func_name);
        }
    }
    for func_name in uncalled_funcs.iter() {
        module.delete_function(&func_name);
        call_map.delete_func(func_name);
    }
}

fn inline_func_with_inst(
    caller_inst: ObjPtr<Inst>,
    callee: ObjPtr<Function>,
    callee_name: &str,
    global_var: Vec<(&String, ObjPtr<Inst>)>,
    arg_list: Vec<ObjPtr<Inst>>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut source_bb = caller_inst.get_parent_bb();
    let (copyed_head_bb, mut copyed_end_bb) =
        copy_func(callee_name, callee, global_var, arg_list, pools);

    // 将使用callee返回值的结果指向copyed_end_bb的返回值
    if let IrType::Void = caller_inst.get_ir_type() {
    } else {
        let return_value = copyed_end_bb.get_tail_inst().get_return_value();
        for user in caller_inst.get_use_list().clone().iter() {
            let index = user.get_operand_index(caller_inst);
            user.as_mut().set_operand(return_value, index);
        }
    }

    // 将调用函数的bb拆成两部分
    let mut new_bb = pools
        .0
        .new_basic_block(format!("after_{}", source_bb.get_name()));
    // 移动指令
    let next = caller_inst.get_next();
    caller_inst.as_mut().remove_self();
    inst_process_in_bb(next, |inst| {
        inst.as_mut().move_self();
        new_bb.push_back(inst);
    });

    source_bb.push_back(pools.1.make_jmp());

    // 设置cfg路径
    let map_up_bb = |bb: ObjPtr<BasicBlock>| {
        bb.get_up_bb()
            .iter()
            .map(|up_bb| {
                if up_bb == &source_bb {
                    new_bb
                } else {
                    up_bb.clone()
                }
            })
            .collect::<Vec<_>>()
    };
    if let InstKind::Return = new_bb.get_tail_inst().get_kind() {
    } else {
        // 修改相应的up_bb
        source_bb
            .get_next_bb()
            .iter()
            .for_each(|bb| bb.as_mut().set_up_bb(map_up_bb(bb.clone())));

        // 将new_bb的后继改为source_bb的next_bb
        new_bb.set_next_bb(source_bb.get_next_bb().clone());
    }
    source_bb.set_next_bb(vec![copyed_head_bb]);

    // 将copyed_end_bb的后继改为new_bb
    copyed_end_bb.get_tail_inst().remove_self();
    copyed_end_bb.push_back(pools.1.make_jmp());
    copyed_end_bb.add_next_bb(new_bb);
}
