///! 此模块为 GEP 指令提供了实现
use super::*;
impl Inst {
    /// 构造一个 GEP 指令
    ///
    /// # Arguments
    /// * 'ptr' - 要计算偏移的指针
    /// * 'offset' - 偏移量
    /// # Returns
    /// 构造好的 GEP 指令
    pub fn make_gep(ptr: ObjPtr<Inst>, offset: ObjPtr<Inst>) -> Inst {
        Inst::new(
            crate::ir::ir_type::IrType::IntPtr,
            InstKind::Gep,
            vec![ptr, offset],
        )
    }

    /// 获得 GEP 指令的指针
    pub fn get_gep_ptr(&self) -> ObjPtr<Inst> {
        self.user.get_operand(0)
    }

    /// 设置 GEP 指令的指针
    pub fn set_gep_ptr(&mut self, ptr: ObjPtr<Inst>) {
        self.user.set_operand(0, ptr);
    }

    /// 获得 GEP 指令的偏移量
    pub fn get_gep_offset(&self) -> ObjPtr<Inst> {
        self.user.get_operand(1)
    }

    /// 设置 GEP 指令的偏移量
    pub fn set_gep_offset(&mut self, offset: ObjPtr<Inst>) {
        self.user.set_operand(1, offset);
    }
}
