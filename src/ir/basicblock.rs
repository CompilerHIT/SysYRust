use super::{instruction::Inst, ir_type::IrType, value::Value};
use crate::utility::ObjPtr;
pub struct BasicBlock {
    name: &'static str,
    value: Value,
    inst_head: Inst,
    next_bb: Vec<ObjPtr<BasicBlock>>,
}

impl BasicBlock {
    /// 构造一个空的BasicBlock
    pub fn new(name: &str) -> BasicBlock {
        BasicBlock {
            name,
            value: Value::new(IrType::BBlock),
            inst_head: Inst::make_head(),
            next_bb: Vec::new(),
        }
    }

    /// 获取BasicBlock的名字
    pub fn get_name(&self) -> &str {
        self.name
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

    /// 是否还有下一个BB
    pub fn has_next_bb(&self) -> bool {
        self.next_bb.len() != 0
    }

    /// 添加下一个BB
    pub fn add_next_bb(&mut self, bb: ObjPtr<BasicBlock>) {
        // 正确性检查
        if self.next_bb.len() > 1 {
            panic!("BasicBlock has more than one next bb");
        }

        self.next_bb.push(bb);
    }

    /// 获取下一个BB
    pub fn get_next_bb(&self) -> &Vec<ObjPtr<BasicBlock>> {
        &self.next_bb
    }
}
