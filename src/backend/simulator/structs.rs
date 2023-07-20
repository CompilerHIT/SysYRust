use std::collections::HashMap;

use crate::{
    backend::{instrs::LIRInst, operand::Reg, BackendPool},
    utility::{ObjPool, ObjPtr},
};

#[derive(Clone)]
pub enum Value {
    Inst(ObjPtr<LIRInst>),
    IImm(i64),
    Addr((String, i64)),
}

///程序资源状态
#[derive(Clone)]
pub struct ProgramStat {
    reg_val: HashMap<Reg, Option<Value>>,
    mem_val: HashMap<Value, Value>,
    pool: &'static BackendPool,
}

///程序执行状态
///如果确定程序的跳出目标,则返回一个跳出目标(否则返回所有可能跳出目标的列表)
#[derive(Clone)]
pub enum ExecuteStat {
    Jump(String),
    MayJump(Vec<String>),
}

impl ProgramStat {
    ///初始化程序状态
    pub fn new(pool: &'static BackendPool) -> ProgramStat {
        ProgramStat {
            reg_val: HashMap::new(),
            mem_val: HashMap::new(),
            pool: pool,
        }
    }

    ///吞入一条指令,修改程序状态
    pub fn consume_inst(&mut self, inst: ObjPtr<LIRInst>) {}

    //吞入一个块,修改程序状态
    pub fn consume_block(&mut self, inst: ObjPtr<LIRInst>) {}

    //

    ///判断两个寄存器的值是否是相同的
    /// 如果两个寄存器的值相同,返回true
    /// 如果其中任何一个寄存器的值为未知,或者两个寄存器的值不同，返回false
    pub fn is_equal(&mut self, reg1: &Reg, reg2: &Reg) -> bool {
        todo!()
    }
}
