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

    /// 创建intptr类型的phi指令
    pub fn make_intptr_phi(&mut self) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::IntPtr, InstKind::Phi, vec![]))
    }

    /// 创建floatptr类型的phi指令
    pub fn make_floatptr_phi(&mut self) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::FloatPtr, InstKind::Phi, vec![]))
    }
}
impl Inst {
    /// 向phi指令中添加一个操作数
    pub fn add_operand(&mut self, operand: ObjPtr<Inst>) {
        self.user.push_operand(operand);

        // 更新操作数的使用者
        operand.as_mut().add_user(self)
    }

    /// 获得phi指令的操作数列表
    pub fn get_operands(&self) -> &Vec<ObjPtr<Inst>> {
        self.user.get_operands()
    }
}
