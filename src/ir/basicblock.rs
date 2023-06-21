use super::{
    instruction::{Inst, InstKind},
    ir_type::IrType,
    value::Value,
};
use crate::utility::{ObjPool, ObjPtr};

#[derive(Debug, Clone)]
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
        bb.as_mut().inst_head.init_head(bb);
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

    /// 设置BasicBlock的名字
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// 获取BasicBlock的名字
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// 检查是否为空的BasicBlock
    pub fn is_empty(&self) -> bool {
        if self.inst_head.is_head() {
            true
        } else {
            false
        }
    }

    /// 判断是否为初始块
    pub fn is_entry(&self) -> bool {
        self.up_bb.len() == 0
    }

    /// 判断是否为结束块
    pub fn is_exit(&self) -> bool {
        self.next_bb.len() == 0
    }

    /// 获取BasicBlock的第一条指令
    /// 请确保BasicBlock不为空再使用
    pub fn get_head_inst(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.is_empty(), false);
        self.inst_head.get_next()
    }

    /// 初始化BasicBlock的Head指令
    pub fn init_head(&mut self) {
        self.inst_head.init_head(ObjPtr::new(self));
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
        if self.up_bb.contains(&ObjPtr::new(bb)) {
            return;
        }

        let bb = ObjPtr::new(bb);
        self.up_bb.push(bb);
    }

    /// 设置up_bb
    pub fn set_up_bb(&mut self, bb: Vec<ObjPtr<BasicBlock>>) {
        self.up_bb = bb;
    }

    /// 设置next_bb
    pub fn set_next_bb(&mut self, bb: Vec<ObjPtr<BasicBlock>>) {
        self.next_bb = bb;
    }

    /// 获取下一个BB
    pub fn get_next_bb(&self) -> &Vec<ObjPtr<BasicBlock>> {
        &self.next_bb
    }

    /// 获取上一个BB
    pub fn get_up_bb(&self) -> &Vec<ObjPtr<BasicBlock>> {
        &self.up_bb
    }

    /// 替换前继BB
    /// # Arguments
    /// * `old_bb` - 被替换的BB
    /// * `new_bb` - 新的BB
    pub fn replace_up_bb(&mut self, old_bb: ObjPtr<BasicBlock>, new_bb: ObjPtr<BasicBlock>) {
        let index = self.get_up_bb().iter().position(|x| *x == old_bb).unwrap();
        self.up_bb[index] = new_bb;
    }

    /// 替换后继BB
    /// # Arguments
    /// * `old_bb` - 被替换的BB
    /// * `new_bb` - 新的BB
    pub fn replace_next_bb(&mut self, old_bb: ObjPtr<BasicBlock>, new_bb: ObjPtr<BasicBlock>) {
        let index = self
            .get_next_bb()
            .iter()
            .position(|x| *x == old_bb)
            .unwrap();
        new_bb.as_mut().replace_up_bb(old_bb, ObjPtr::new(self));
        self.next_bb[index] = new_bb;
    }

    /// 删除前继BB
    /// # Arguments
    /// * `bb` - 被删除的BB
    fn remove_up_bb(&mut self, bb: ObjPtr<BasicBlock>) {
        let index = self.get_up_bb().iter().position(|x| *x == bb).unwrap();
        self.up_bb.remove(index);

        // 修改phi的参数
        let mut inst = self.get_head_inst();
        while let InstKind::Phi = inst.as_ref().get_kind() {
            inst.remove_operand_by_index(index);
            inst = inst.get_next();
        }
    }

    /// 删除后继BB
    /// # Arguments
    /// * `bb` - 被删除的BB
    pub fn remove_next_bb(&mut self, bb: ObjPtr<BasicBlock>) {
        let index = self.get_next_bb().iter().position(|x| *x == bb).unwrap();
        bb.as_mut().remove_up_bb(ObjPtr::new(self));
        self.next_bb.remove(index);
    }

    /// 清除自身记录的后继BB
    pub fn clear_next_bb(&mut self) {
        for bb in self.get_next_bb().iter() {
            bb.as_mut().remove_up_bb(ObjPtr::new(self));
        }
        self.next_bb.clear();
    }
}
