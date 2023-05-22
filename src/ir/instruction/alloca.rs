///! 此文件为Inst中关于内存分配指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 申请一个int类型的数组
    ///
    /// # Arguments
    /// * 'length' - 要申请的数组的长度
    /// # Returns
    /// 构造好的数组指令，其值为Intptr
    pub fn make_int_array(&mut self, length: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        // 数组长度必须为整数
        let ir_type = length.as_ref().get_ir_type();
        debug_assert!(ir_type == IrType::Int || ir_type == IrType::ConstInt);

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::IntPtr,
            InstKind::Alloca,
            vec![length],
        ));

        // 设置use list
        length.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 申请一个double类型的数组
    ///
    /// # Arguments
    /// * 'length' - 要申请的数组的长度
    /// # Returns
    /// 构造好的数组指令，其值为Floatptr
    pub fn make_double_array(&mut self, length: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        // 数组长度必须为整数
        let ir_type = length.as_ref().get_ir_type();
        debug_assert!(ir_type == IrType::Int || ir_type == IrType::ConstInt);

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::FloatPtr,
            InstKind::Alloca,
            vec![length],
        ));

        // 设置use list
        length.as_mut().add_user(inst.as_ref());
        inst
    }
}

impl Inst {
    /// 获得数组长度
    pub fn get_array_length(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::Alloca = self.get_kind() {
            debug_assert!(self.user.get_operands_size() == 1);
        } else {
            unreachable!("Inst::get_array_length")
        }

        self.user.get_operand(0)
    }

    /// 设置数组长度
    pub fn set_array_length(&mut self, length: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Alloca = self.get_kind() {
            debug_assert!(self.user.get_operands_size() == 1);
            // 数组长度必须为整数或整数常量
            let ir_type = length.as_ref().get_ir_type();
            debug_assert!(ir_type == IrType::Int || ir_type == IrType::ConstInt);
        } else {
            unreachable!("Inst::set_array_length")
        }

        // 设置use list
        let old_length = self.get_array_length();
        old_length.as_mut().remove_user(self);
        length.as_mut().add_user(self);

        self.user.set_operand(0, length);
    }
}
