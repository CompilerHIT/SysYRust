use super::function::Function;
use super::{basicblock::BasicBlock, instruction::Inst, module::Module};
use super::{dump_now, tools::*};
use crate::utility::ObjPool;

mod array_transform;
mod condition_transform;
mod constant_folding;
mod dead_code_eliminate;
mod delete_empty_block;
mod delete_redundant_load_store;
mod func_inline;
mod global_value_numbering;
mod global_var_transform;
mod hoist;
mod loop_operation;
mod meaningless_insts_folding;
mod partial_redundancy_elimination;
mod phi_optimizer;
mod return_unused;
mod simplify_cfg;
mod tail_call_optimize;
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

        // gvn hoist
        hoist::hoist(module, optimize_flag, &mut pools);

        // 循环优化
        loop_operation::loop_optimize(module, &mut pools, true);
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // // 尾递归优化
        tail_call_optimize::tail_call_optimize(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 函数内联
        func_inline::inline_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 简化cfg
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // gvn hoist
        hoist::hoist(module, optimize_flag, &mut pools);

        // 循环优化
        //loop_operation::loop_optimize(module, &mut pools, false);
        //simplify_cfg::simplify_cfg_run(module, &mut pools);
        //functional_optimizer(module, &mut pools, optimize_flag);

        // TODO: 性能优化

        // 再做一次
        functional_optimizer(module, &mut pools, optimize_flag);
    }
}

/// 增加一些调用后端函数的接口
pub fn add_interface(module: &mut Module, func_pool: &mut ObjPool<Function>, optimize_flag: bool) {
    if !optimize_flag {
        return;
    }

    // 增加自动并行化接口

    // void hitsz_thread_init();
    // 初始化线程池，在main函数中调用一次即可
    let thread_init = func_pool.new_function();
    module.push_function("hitsz_thread_init".to_string(), thread_init);

    // int hitsz_thread_create();
    // 创建一个新的线程，返回线程id
    let mut thread_create = func_pool.new_function();
    thread_create.set_return_type(super::ir_type::IrType::Int);
    module.push_function("hitsz_thread_create".to_string(), thread_create);

    // void hitsz_thread_join();
    // 等待线程结束，只有0号线程调用
    let thread_join = func_pool.new_function();
    module.push_function("hitsz_thread_join".to_string(), thread_join);

    // void hitsz_thread_exit();
    // 线程退出，非0号线程调用
    let thread_exit = func_pool.new_function();
    module.push_function("hitsz_thread_exit".to_string(), thread_exit);
}

fn functional_optimizer(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    debug_assert!(verify::verify_run(module));

    // phi优化
    phi_optimizer::phi_run(module);

    // 常量折叠
    constant_folding::constant_folding(module, pools, optimize_flag);
    // 死代码删除
    dead_code_eliminate::dead_code_eliminate(module, optimize_flag);

    // 消除不必要的指令
    meaningless_insts_folding::meaningless_inst_folding(module, pools);

    // gvn
    global_value_numbering::gvn(module, optimize_flag);

    // 常量折叠
    constant_folding::constant_folding(module, pools, optimize_flag);

    // 数组优化
    array_transform::array_optimize(module, pools, optimize_flag);

    // 全局变量转换
    global_var_transform::global_var_transform(module, pools, optimize_flag);

    // 函数返回值优化
    return_unused::return_unused(module);

    // 死代码删除
    dead_code_eliminate::dead_code_eliminate(module, optimize_flag);
    // 全局死代码删除
    dead_code_eliminate::global_eliminate(module);
}
