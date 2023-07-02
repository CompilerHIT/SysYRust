use crate::{
    ir::{
        analysis::loop_tree::loop_recognize::loop_recognize, basicblock::BasicBlock,
        instruction::Inst, module::Module, tools::func_process,
    },
    utility::ObjPool,
};

use self::loop_simplify::loop_simplify_run;

mod loop_simplify;

pub fn loop_optimize(
    module: &mut Module,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
    let mut loop_map = loop_recognize(module);
    func_process(module, |name, _| {
        loop_simplify_run(loop_map.get_mut(&name).unwrap(), pools);
    })
}
