
use super::{basicblock::BasicBlock, instruction::Inst, module::Module};
use super::{dump_now, tools::*};
use crate::utility::ObjPool;

mod constant_folding;
mod dead_code_eliminate;
mod delete_redundant_load_store;
mod func_inline;
mod global_value_numbering;
mod loop_operation;
mod meaningless_insts_folding;
mod phi_optimizer;
mod simplify_cfg;
mod verify;

pub fn optimizer_run(
    module: &mut Module,
    mut pools: (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    // 在功能点上对phi指令进行优化
    functional_optimizer(module, &mut pools, optimize_flag);

    if optimize_flag {
        // 简化cfg
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 循环优化
        loop_operation::loop_optimize(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 函数内联
        func_inline::inline_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        //gvn
        // global_value_numbering::gvn(module);
        // functional_optimizer(module, &mut pools, optimize_flag);

        // // 全局值编号
        global_value_numbering::easy_gvn(module);
        functional_optimizer(module, &mut pools, optimize_flag);

        // //冗余load,store删除
        delete_redundant_load_store::load_store_opt(module);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 全局值编号
        global_value_numbering::easy_gvn(module);
        functional_optimizer(module, &mut pools, optimize_flag);

        // //冗余load,store删除
        delete_redundant_load_store::load_store_opt(module);
        functional_optimizer(module, &mut pools, optimize_flag);

        // TODO: 性能优化
    }
}

fn functional_optimizer(
    module: &mut Module,
    mut pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    debug_assert!(verify::verify_run(module));
    // 死代码删除
    dead_code_eliminate::dead_code_eliminate(module, optimize_flag);

    // phi优化
    phi_optimizer::phi_run(module);

    // 常量折叠
    constant_folding::constant_folding(module, &mut pools, optimize_flag);

    // 消除不必要的指令
    meaningless_insts_folding::meaningless_inst_folding(module, &mut pools);

    // 全局死代码删除
    dead_code_eliminate::global_eliminate(module);
}
