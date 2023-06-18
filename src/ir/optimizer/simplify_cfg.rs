use crate::{
    ir::{basicblock::BasicBlock, module::Module},
    utility::ObjPtr,
};

///! 对于block的优化
///! 1. 删除无法到达的block：除头block外没有前继的就是无法到达的
///! 2. 合并只有一个后继和这个后继只有一个前继的block
///! 3. 删除无法到达的分支

pub fn simplify_cfg(module: &mut Module) {}
