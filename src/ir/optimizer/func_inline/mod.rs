use std::collections::{HashMap, HashSet};

use crate::{
    ir::{basicblock::BasicBlock, function::Function, instruction::Inst, module::Module},
    utility::{ObjPool, ObjPtr},
};

mod call_map;
mod copy_func;

pub use call_map::call_map_gen;
pub use call_map::CallMap;

pub fn inline_run(module: &mut Module, pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>)) {
    let call_map = call_map::call_map_gen(module);

    // 先内联没有后继的函数
}

fn inline_func(
    caller: ObjPtr<Function>,
    callee: ObjPtr<Function>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) {
}
