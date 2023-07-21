///程序执行状态
///如果确定程序的跳出目标,则返回一个跳出目标(否则返回所有可能跳出目标的列表)
///如果是函数调用,返回一条Call信息
///如果没有跳转,则顺序执行下一条指令
#[derive(Clone, PartialEq, Eq)]
pub enum ExecuteStat {
    Jump(String),
    MayJump(Vec<String>),
    Call(String),
    NextInst,
    Ret,
}
