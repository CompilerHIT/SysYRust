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
        debug_assert_eq!(self.get_ir_type(), operand.as_ref().get_ir_type());

        // 修改use list
        self.user.get_operand(index).remove_user(self);
        operand.add_user(self);

        // 更新操作数的使用者
        self.user.set_operand(index, operand);
    }
}
