use std::collections::HashMap;

use crate::{
    ir::{
        analysis::loop_tree::{loop_recognize::loop_recognize, LoopInfo, LoopList},
        basicblock::BasicBlock,
        dump_now,
        function::Function,
        instruction::Inst,
        module::Module,
        tools::{func_process, inst_process_in_bb},
    },
    utility::{ObjPool, ObjPtr},
};

use self::{
    auto_parallelization::auto_paralellization, licm::licm_run, livo::livo_run,
    loop_elimination::loop_elimination, loop_simplify::loop_simplify_run,
    loop_unrolling::loop_unrolling,
};

mod auto_parallelization;
mod licm;
mod livo;
mod loop_elimination;
mod loop_simplify;
mod loop_unrolling;

pub fn loop_optimize(
    module: &mut Module,
    max_loop_unrolling: usize,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    para: bool,
) {
    let mut loop_map = loop_recognize(module);
    func_process(module, |name, _| {
        loop_simplify_run(loop_map.get_mut(&name).unwrap(), pools);
    });

    super::phi_optimizer::phi_run(module);

    // 循环不变量外提
    func_process(module, |name, _| {
        licm_run(loop_map.get_mut(&name).unwrap(), pools);
    });

    // 循环归纳和删除
    loop_elimination(module, &mut loop_map, pools);

    // 循环展开
    loop_unrolling(module, &mut loop_map, max_loop_unrolling, pools);
    super::functional_optimizer(module, pools, false);

    if para {
        // 自动并行化
        // auto_paralellization(module, &mut loop_map, pools);
        // super::functional_optimizer(module, pools, false);

        // 归纳变量强度削减

        func_process(module, |name, func| {
            let dominator_tree =
                crate::ir::analysis::dominator_tree::calculate_dominator(func.get_head());
            livo_run(dominator_tree, loop_map.get_mut(&name).unwrap(), pools);
        });
    }
}
