use std::collections::{HashMap, LinkedList};

use crate::utility::ObjPtr;

use self::{program_stat::ProgramStat, structs::BuiltInFunc};

use super::instrs::{Func, BB};

// 对程序进行建模
//以为 死代码消除 ,编译时计算等提供接口
pub mod execute_stat;
mod impl_consume_inst;
pub mod program_stat;
pub mod structs;

///解释器
pub struct Simulator {
    ///块缓存(只有在块缓存中的内容才能够解释执行)
    name_blocks: HashMap<String, ObjPtr<BB>>,
    name_funcs: HashMap<String, ObjPtr<Func>>,
    ///内置函数
    build_in_funcs: HashMap<String, BuiltInFunc>,
    ///程序资源状态
    program_stat: ProgramStat,
    ///调用栈 (实际解释执行的时候需要)
    call_stack: LinkedList<(ObjPtr<BB>, usize)>, //函数调用栈
}
