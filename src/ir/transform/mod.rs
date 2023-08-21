use self::gvn_hoist::hoist_to_loop_head;

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
mod gvn_hoist;
mod loop_operation;
mod meaningless_insts_folding;
mod partial_redundancy_elimination;
mod phi_optimizer;
mod return_unused;
mod simplify_cfg;
mod sink;
mod tail_call_optimize;
mod verify;

pub fn optimizer_run(
    module: &mut Module,
    mut pools: (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
    para: bool,
) {
    // 在功能点上对phi指令进行优化
    functional_optimizer(module, &mut pools, optimize_flag);

    if optimize_flag {
        // 简化cfg
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 局部冗余消除 指令上提
        // partial_redundancy_elimination::pre(module, optimize_flag, &mut pools);

        // 循环优化
        loop_operation::loop_optimize(module, 100, &mut pools, para);
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 指令下沉
        sink::sink(module, &mut pools);

        // 尾递归优化
        tail_call_optimize::tail_call_optimize(module, &mut pools);
        // 函数内联
        func_inline::inline_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);
        // 简化cfg
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 局部冗余消除 指令上提
        //partial_redundancy_elimination::pre(module, optimize_flag, &mut pools);

        // 循环优化
        loop_operation::loop_optimize(module, 100, &mut pools, false);
        simplify_cfg::simplify_cfg_run(module, &mut pools);
        functional_optimizer(module, &mut pools, optimize_flag);

        // 指令下沉
        sink::sink(module, &mut pools);
    }
}

/// 增加一些调用后端函数的接口
pub fn add_interface(
    module: &mut Module,
    func_pool: &mut ObjPool<Function>,
    inst_pool: &mut ObjPool<Inst>,
    optimize_flag: bool,
    pa_flag: bool,
) {
    if !optimize_flag {
        return;
    }

    if pa_flag {
        // 增加自动并行化接口

        // void hitsz_thread_init();
        // 初始化线程池，在main函数中调用一次即可
        let thread_init = func_pool.new_function();
        module.push_function("hitsz_thread_init".to_string(), thread_init);
        // 将这个函数插入到main函数的开头
        let thread_init_call = inst_pool.make_void_call("hitsz_thread_init".to_string(), vec![]);
        module
            .get_function("main")
            .get_head()
            .push_front(thread_init_call);

        // int hitsz_thread_create();
        // 创建一个新的线程，返回线程id
        let mut thread_create = func_pool.new_function();
        thread_create.set_return_type(super::ir_type::IrType::Int);
        module.push_function("hitsz_thread_create".to_string(), thread_create);

        // void hitsz_thread_join();
        // 等待线程结束
        let thread_join = func_pool.new_function();
        module.push_function("hitsz_thread_join".to_string(), thread_join);

        // int hitsz_get_thread_num();
        // 获取当前线程id
        let mut get_thread_num = func_pool.new_function();
        get_thread_num.set_return_type(super::ir_type::IrType::Int);
        module.push_function("hitsz_get_thread_num".to_string(), get_thread_num);
    }

    // void hitsz_memset(intptr array, int value, int n);
    let mut memset = func_pool.new_function();
    let array = inst_pool.make_param(super::ir_type::IrType::IntPtr);
    let value = inst_pool.make_param(super::ir_type::IrType::Int);
    let n = inst_pool.make_param(super::ir_type::IrType::Int);
    memset.set_parameter("array".to_string(), array);
    memset.set_parameter("value".to_string(), value);
    memset.set_parameter("n".to_string(), n);
    module.push_function("hitsz_memset".to_string(), memset);

    // void hitsz_memcopy(intptr dst, intptr src, int n);
    let mut hitsz_memcopy = func_pool.new_function();
    let dst = inst_pool.make_param(super::ir_type::IrType::IntPtr);
    let src = inst_pool.make_param(super::ir_type::IrType::IntPtr);
    let n = inst_pool.make_param(super::ir_type::IrType::Int);
    hitsz_memcopy.set_parameter("dst".to_string(), dst);
    hitsz_memcopy.set_parameter("src".to_string(), src);
    hitsz_memcopy.set_parameter("n".to_string(), n);
    module.push_function("hitsz_memcopy".to_string(), hitsz_memcopy);
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
