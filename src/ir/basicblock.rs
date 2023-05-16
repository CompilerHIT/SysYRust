use super::{instruction::Inst, ir_type::IrType, value::Value};
use crate::utility::ObjPtr;
pub struct BasicBlock {
    value: Value,
    inst_head: Inst,
}

impl BasicBlock {
    /// 构造一个空的BasicBlock
    pub fn new() -> BasicBlock {
        BasicBlock {
            value: Value::new(IrType::BBlock),
            inst_head: Inst::make_head(),
        }
    }

    /// 检查是否为空的BasicBlock
    pub fn is_empty(&self) -> bool {
        if self.inst_head.is_head() {
            debug_assert_eq!(self.inst_head.is_tail(), true);
            true
        } else {
            debug_assert_eq!(self.inst_head.is_tail(), false);
            false
        }
    }

    /// 获取BasicBlock的第一条指令
    pub fn get_head_inst(&self) -> ObjPtr<Inst> {
        assert_eq!(self.is_empty(), false);
        self.inst_head.get_next().unwrap()
    }

    /// 获取BasicBlock的最后一条指令
    pub fn get_tail_inst(&self) -> ObjPtr<Inst> {
        assert_eq!(self.is_empty(), false);
        self.inst_head.get_prev().unwrap()
    }

    /// 将指令插入到BasicBlock的最后
    pub fn push_back(&mut self, inst: ObjPtr<Inst>) {
        self.inst_head.insert_before(inst);
    }

    /// 将指令插入到BasicBlock的最前
    pub fn push_front(&mut self, inst: ObjPtr<Inst>) {
        self.inst_head.insert_after(inst);
    }

    /// 初始化BB
    /// 建议在向内存池申请内存后对BB使用一次此函数
    pub fn init_bb(&mut self) {
        self.inst_head.init_head();
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }
}
