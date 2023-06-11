///! 本文件为phi指令的实现
use super::*;
impl ObjPool<Inst> {
    /// 创建int类型的phi指令
    pub fn make_int_phi(&mut self) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::Int, InstKind::Phi, vec![]))
    }

    /// 创建float类型的phi指令
    pub fn make_float_phi(&mut self) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::Float, InstKind::Phi, vec![]))
    }
}
impl Inst {
    /// 向phi指令中添加一个操作数
    pub fn add_operand(&mut self, mut operand: ObjPtr<Inst>) {
        // 正确性检查
        debug_assert_eq!(self.get_ir_type(), operand.get_ir_type());

        self.user.push_operand(operand);

        // 更新操作数的使用者
        operand.add_user(self)
    }

    /// 获得phi指令的操作数列表
    pub fn get_operands(&self) -> &Vec<ObjPtr<Inst>> {
        self.user.get_operands()
    }

    /// 设置phi指令的操作数
    /// # Arguments
    /// * 'operand' - 操作数
    /// * 'index' - 操作数的索引
    pub fn set_operand(&mut self, mut operand: ObjPtr<Inst>, index: usize) {
        // 正确性检查
        debug_assert_eq!(self.get_ir_type(), operand.get_ir_type());

        // 修改use list
        self.user.get_operand(index).remove_user(self);
        operand.add_user(self);

        // 更新操作数的使用者
        self.user.set_operand(index, operand);
    }

    /// 将一个操作数替换为另一个操作数
    /// # Arguments
    /// * 'old' - 被替换的操作数
    /// * 'new' - 新的操作数
    pub fn replace_operand(&mut self, old: ObjPtr<Inst>, new: ObjPtr<Inst>) {
        let index = self.get_operands().iter().position(|x| *x == old).unwrap();
        self.set_operand(new, index);
    }

    /// 删除一个操作数
    /// # Arguments
    /// * 'operand' - 被删除的操作数
    pub fn remove_operand(&mut self, operand: ObjPtr<Inst>) {
        let index = self
            .get_operands()
            .iter()
            .position(|x| *x == operand)
            .unwrap();
        self.user.get_operand(index).remove_user(self);
        self.user.remove_operand(index);
    }

    pub fn remove_operand_by_index(&mut self, index: usize) {
        self.user.get_operand(index).remove_user(self);
        self.user.remove_operand(index);
    }
}
