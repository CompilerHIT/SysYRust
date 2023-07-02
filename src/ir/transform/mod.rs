use super::{basicblock::BasicBlock, instruction::Inst, module::Module};
use super::{dump_now, tools::*};
use crate::ir::transform::constant_folding::constant_folding;
use crate::utility::ObjPool;

mod constant_folding;
mod dead_code_eliminate;
mod func_inline;
mod loop_operation;
mod phi_optimizer;
mod simplify_cfg;

pub fn optimizer_run(
    module: &mut Module,
    mut pools: (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    optimize_flag: bool,
) {
    // 在功能点上对phi指令进行优化
    functional_optimizer(module);
    constant_folding(module, &mut pools);
    dead_code_eliminate::dead_code_eliminate(module, true);

    if optimize_flag {
        // 函数内联
        func_inline::inline_run(module, &mut pools);

        // 死代码删除
        dead_code_eliminate::dead_code_eliminate(module, true);

        // 简化cfg
        simplify_cfg::simplify_cfg_run(module, &mut pools);

        // phi优化
        phi_optimizer::phi_run(module);

        // 循环优化
        loop_operation::loop_optimize(module, &mut pools);

        // TODO: 性能优化
    }
}

fn functional_optimizer(module: &mut Module) {
    // 死代码删除
    dead_code_eliminate::dead_code_eliminate(module, true);

    // phi优化
    phi_optimizer::phi_run(module);

    // 全局死代码删除
    dead_code_eliminate::global_eliminate(module);
}
