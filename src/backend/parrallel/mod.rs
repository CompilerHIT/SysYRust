//分析一个块是否能够并行

//对一个块区分并行区间

//往一个块中插入线程库调用

use crate::utility::ObjPtr;

use super::{
    instrs::{Func, BB},
    module::AsmModule,
    BackendPool,
};

///衡量是否有做块内并行的价值
pub fn is_parrallelable(module: &mut AsmModule, func: &ObjPtr<Func>, bb: &ObjPtr<BB>) -> bool {
    if bb.insts.len() < 500 {
        return false;
    }
    return true;
}

pub fn parrallel_single_block(
    module: &mut AsmModule,
    func: &ObjPtr<Func>,
    bb: &ObjPtr<BB>,
    pool: &mut BackendPool,
) -> Vec<ObjPtr<BB>> {
    //区分块尾和块首,在块首把活着的寄存器的值保存到空间中
    //块尾把对后面还有作用的寄存器的值从空间中取下
    if !is_parrallelable(module, func, bb) {
        return vec![*bb];
    }
    unimplemented!();
}
