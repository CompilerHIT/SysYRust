use super::*;

impl ObjPool<Inst> {
    /// 创建int_to_float指令
    /// # Arguments
    /// * 'value' - 要转换的值
    pub fn make_int_to_float(&mut self, value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        if value.as_ref().get_ir_type() != IrType::Int {
            unreachable!("Inst::make_int_to_float")
        }

        let inst = self.put(Inst::new(IrType::Float, InstKind::ItoF, vec![value]));
        // 设置use list
        value.as_mut().add_user(inst.as_ref());
        inst
    }
}

impl Inst {
    /// 获得要转换的值
    pub fn get_int_to_float_value(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::ItoF = self.get_kind() {
            debug_assert_eq!(self.user.get_operands().len(), 1);
            if self.get_ir_type() == IrType::Float {
                self.user.get_operand(0)
            } else {
                unreachable!("Inst::get_int_to_float_value")
            }
        } else {
            unreachable!("Inst::get_int_to_float_value")
        }
    }

    /// 设置要转换的值
    pub fn set_int_to_float_value(&mut self, mut value: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::ItoF = self.get_kind() {
            debug_assert_eq!(self.user.get_operands().len(), 1);
            if self.get_ir_type() == IrType::Float {
                self.user.get_operand(0).remove_user(self);
                value.add_user(self);
                self.user.set_operand(0, value);
            } else {
                unreachable!("Inst::set_int_to_float_value")
            }
        } else {
            unreachable!("Inst::set_int_to_float_value")
        }
    }
}
