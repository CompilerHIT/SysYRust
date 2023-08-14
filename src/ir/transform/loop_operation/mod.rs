use std::collections::HashMap;

use crate::{
    ir::{
        analysis::loop_tree::{loop_recognize::loop_recognize, LoopInfo, LoopList},
        basicblock::BasicBlock,
        dump_now,
        instruction::Inst,
        module::Module,
        tools::{func_process, inst_process_in_bb},
    },
    utility::{ObjPool, ObjPtr},
};

use self::{
    auto_parallelization::auto_paralellization, licm::licm_run, livo::livo_run,
    loop_simplify::loop_simplify_run, loop_unrolling::loop_unrolling,
};

mod auto_parallelization;
mod licm;
mod livo;
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

    // 循环展开
    loop_unrolling(module, &mut loop_map, max_loop_unrolling, pools);
    super::functional_optimizer(module, pools, false);

    if para {
        // 自动并行化
        auto_paralellization(module, &mut loop_map, pools);
        super::functional_optimizer(module, pools, false);

        // 归纳变量强度削减
        func_process(module, |name, func| {
            let dominator_tree =
                crate::ir::analysis::dominator_tree::calculate_dominator(func.get_head());
            livo_run(dominator_tree, loop_map.get_mut(&name).unwrap(), pools);
        });
    }
}

/// 识别一个循环是否是有利于优化的，即循环中不会有break和continue
/// 识别方法：
/// 1. 如果一个循环头有多于两个前驱，则存在continue
/// 2. 如果一个循环除了头还有别的能够跳转到外部的块，则存在break
fn is_break_continue_exist(loop_info: ObjPtr<LoopInfo>) -> bool {
    let header = loop_info.get_header();
    if header.get_up_bb().len() > 2 {
        true
    } else if loop_info
        .get_current_loop_bb()
        .iter()
        .any(|bb| !loop_info.is_in_loop(bb))
    {
        true
    } else {
        false
    }
}
