use super::*;

///! 此文件为一元指令的实现

impl ObjPool<Inst> {
    /// 创建取正指令
    /// # Arguments
    /// * `value` - 要取正的值
    pub fn make_neg(&mut self, value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let ir_type = value.as_ref().get_ir_type();
        let kind = InstKind::Unary(UnOp::Pos);
        let operands = vec![value];
        self.put(Inst::new(ir_type, kind, operands))
    }

    /// 创建取负指令
    /// # Arguments
    /// * `value` - 要取负的值
    pub fn make_pos(&mut self, value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let ir_type = value.as_ref().get_ir_type();
        let kind = InstKind::Unary(UnOp::Neg);
        let operands = vec![value];
        self.put(Inst::new(ir_type, kind, operands))
    }

    /// 创建取反指令
    /// # Arguments
    /// * `value` - 要取反的值
    pub fn make_not(&mut self, value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let ir_type = value.as_ref().get_ir_type();
        let kind = InstKind::Unary(UnOp::Not);
        let operands = vec![value];
        self.put(Inst::new(ir_type, kind, operands))
    }
}

impl Inst {
    /// 获得一元指令的操作数
    pub fn get_unary_operand(&self) -> ObjPtr<Inst> {
        self.user.get_operand(0)
    }

    /// 设置一元指令的操作数
    /// # Arguments
    /// * `operand` - 操作数
    pub fn set_unary_operand(&mut self, operand: ObjPtr<Inst>) {
        self.user.set_operand(0, operand);
    }
}
