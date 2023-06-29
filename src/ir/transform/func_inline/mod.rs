use std::collections::HashMap;

use crate::{
    ir::{
        basicblock::BasicBlock,
        call_map_gen,
        function::Function,
        instruction::{Inst, InstKind},
        ir_type::IrType,
        module::Module,
        tools::func_process,
        CallMap,
    },
    utility::{ObjPool, ObjPtr},
};

mod copy_func;
mod get_optimizate;
mod inline_operation;

use self::{copy_func::copy_func, inline_operation::delete_uncalled_func};
use self::{get_optimizate::gep_optimize, inline_operation::inline_func};

use super::{bfs_inst_process, inst_process_in_bb};

pub fn inline_run(module: &mut Module, pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)) {
    let mut call_map = call_map_gen(module);

    // 先内联没有后继的函数
    inline_no_succ(module, &mut call_map, pools);

    // 再内联没有调用自己的函数
    inline_no_self_call(module, &mut call_map, pools);

    // 消去嵌套的gep指令
    func_process(module, |_, func| {
        gep_optimize(func.get_head(), pools);
    })
}

fn inline_no_self_call(
    module: &mut Module,
    call_map: &mut CallMap,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    loop {
        let mut changed = false;
        let mut delete_list = Vec::new();
        let mut add_list = Vec::new();
        for (func_name, succs) in call_map.iter() {
            for succ in succs.iter() {
                if call_map.contains_edge(succ, succ) {
                    break;
                }

                changed = true;

                // 内联后不存在调用边，删除该调用边
                delete_list.push((func_name.clone(), succ.clone()));
                // 调用者继承被调用者的调用关系，将其加入add_list中
                for next in call_map.get_succs(succ).iter() {
                    add_list.push((func_name.clone(), next.clone()));
                }

                // 内联函数
                let caller = module.get_function(&func_name);
                let callee = module.get_function(succ);
                inline_func(caller, callee, succ, module.get_all_var(), pools);
            }
        }

        // 删除已经内联的函数的调用边
        for (caller, callee) in delete_list.iter() {
            call_map.delete_edge(&caller, &callee);
        }

        // 添加新的调用边
        for (caller, callee) in add_list.iter() {
            call_map.add_edge(&caller, &callee);
        }

        // 删除没有调用者的函数
        delete_uncalled_func(module, call_map);

        if !changed {
            break;
        }
    }
}

fn inline_no_succ(
    module: &mut Module,
    call_map: &mut CallMap,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    loop {
        let mut changed = false;
        let mut delete_list = Vec::new();
        for (func_name, succs) in call_map.iter() {
            if succs.is_empty() && !func_name.eq("main") {
                changed = true;
                let callee = module.get_function(func_name);
                let callers = call_map.find_predecessors(func_name);
                for caller_name in callers {
                    let caller = module.get_function(&caller_name);
                    inline_func(caller, callee, &func_name, module.get_all_var(), pools);
                    delete_list.push((caller_name, func_name.clone()));
                }
            }
        }

        // 删除已经内联的函数的调用边
        for (caller, callee) in delete_list.iter() {
            call_map.delete_edge(&caller, &callee);
        }

        delete_uncalled_func(module, call_map);

        if !changed {
            break;
        }
    }
}
