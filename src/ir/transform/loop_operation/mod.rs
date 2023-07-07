use std::collections::HashMap;

use crate::{
    ir::{
        analysis::loop_tree::{loop_recognize::loop_recognize, LoopList},
        basicblock::BasicBlock,
        instruction::Inst,
        module::Module,
        tools::{func_process, inst_process_in_bb},
    },
    utility::ObjPool,
};

use self::{licm::licm_run, loop_simplify::loop_simplify_run};

mod licm;
mod loop_simplify;

pub fn loop_optimize(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut loop_map = loop_recognize(module);

    func_process(module, |name, _| {
        loop_simplify_run(loop_map.get_mut(&name).unwrap(), pools);
    });

    super::functional_optimizer(module, pools, false);

    // 循环不变量外提
    func_process(module, |name, _| {
        licm_run(loop_map.get_mut(&name).unwrap(), pools);
    });
}

fn scev_dump(module: &mut Module, loop_map: HashMap<String, LoopList>) {
    func_process(module, |name, _| {
        crate::log_file!("scev_log", "function: {}", name);
        let mut scev_analyzer = crate::ir::analysis::scev::SCEVAnalyzer::new();
        let loop_list = loop_map.get(&name).unwrap();
        scev_analyzer.set_loops(loop_list);
        for loop_info in loop_list.get_loop_list().iter() {
            for bb in loop_info.get_current_loop_bb() {
                crate::log_file!("scev_log", "bb: {}", bb.get_name());
                inst_process_in_bb(bb.get_head_inst(), |inst| {
                    let scev = scev_analyzer.analyze(inst);
                    crate::log_file!("scev_log", "inst: {:?} scev: {:?}", inst, scev);
                })
            }
        }
    });
}
