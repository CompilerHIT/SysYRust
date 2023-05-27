use super::{instruction::Inst, ir_type::IrType, value::Value};
use crate::utility::{ObjPool, ObjPtr};
pub struct BasicBlock {
    name: String,
    value: Value,
    inst_head: Inst,
    up_bb: Vec<ObjPtr<BasicBlock>>,
    next_bb: Vec<ObjPtr<BasicBlock>>,
}

impl ObjPool<BasicBlock> {
    /// 创建一个新的BasicBlock
    pub fn new_basic_block(&mut self, name: String) -> ObjPtr<BasicBlock> {
        let bb = BasicBlock::new(name);
        let bb = self.put(bb);

        // 初始化指令头
        bb.as_mut().inst_head.init_head();
        bb
    }
}

impl BasicBlock {
    /// 构造一个空的BasicBlock
    pub fn new(name: String) -> BasicBlock {
        BasicBlock {
            name,
            value: Value::new(IrType::BBlock),
            inst_head: Inst::make_head(),
            up_bb: Vec::new(),
            next_bb: Vec::new(),
        }
    }

    /// 获取BasicBlock的名字
    pub fn get_name(&self) -> &str {
        self.name.as_str()
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
    /// 请确保BasicBlock不为空再使用
    pub fn get_head_inst(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.is_empty(), false);
        self.inst_head.get_next()
    }

    /// 获取BasicBlock的最后一条指令
    /// 请确保BasicBlock不为空再使用
    pub fn get_tail_inst(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.is_empty(), false);
        self.inst_head.get_prev()
    }

    /// 将指令插入到BasicBlock的最后
    pub fn push_back(&mut self, inst: ObjPtr<Inst>) {
        self.inst_head.insert_before(inst);
    }

    /// 将指令插入到BasicBlock的最前
    pub fn push_front(&mut self, inst: ObjPtr<Inst>) {
        self.inst_head.insert_after(inst);
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
        debug_assert!(self.next_bb.len() <= 1, "BB已经存在两个后继BB",);

        // 给下一个BB添加上一个BB
        bb.as_mut().add_up_bb(self);

        self.next_bb.push(bb);
    }

    /// 增加上一个BB
    /// # Arguments
    /// * `bb` - 上一个BB
    pub fn add_up_bb(&mut self, bb: &BasicBlock) {
        let bb = ObjPtr::new(bb);
        self.up_bb.push(bb);
    }

    /// 获取下一个BB
    pub fn get_next_bb(&self) -> &Vec<ObjPtr<BasicBlock>> {
        &self.next_bb
    }

    /// 获取上一个BB
    pub fn get_up_bb(&self) -> &Vec<ObjPtr<BasicBlock>> {
        &self.up_bb
    }
}
