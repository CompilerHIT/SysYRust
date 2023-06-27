use super::*;

impl ObjPool<Inst> {
    /// 创建float_to_int指令
    /// # Arguments
    /// * 'value' - 要转换的值
    pub fn make_float_to_int(&mut self, mut value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        if value.get_ir_type() != IrType::Float {
            unreachable!("Inst::make_float_to_int")
        }
        let inst = self.put(Inst::new(IrType::Int, InstKind::FtoI, vec![value]));
        // 设置use list
        value.add_user(inst.as_ref());
        inst
    }
}

impl Inst {
    /// 获得要转换的值
    pub fn get_float_to_int_value(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::FtoI = self.get_kind() {
            debug_assert_eq!(self.user.get_operands().len(), 1);
            if self.get_ir_type() == IrType::Int {
                self.user.get_operand(0)
            } else {
                unreachable!("Inst::get_float_to_int_value")
            }
        } else {
            unreachable!("Inst::get_float_to_int_value")
        }
    }

    /// 设置要转换的值
    pub fn set_float_to_int_value(&mut self, value: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::FtoI = self.get_kind() {
            debug_assert_eq!(self.user.get_operands().len(), 1);
            if self.get_ir_type() == IrType::Int {
                self.user.get_operand(0).as_mut().remove_user(self);
                value.as_mut().add_user(self);
                self.user.set_operand(0, value);
            } else {
                unreachable!("Inst::set_float_to_int_value")
            }
        } else {
            unreachable!("Inst::set_float_to_int_value")
        }
    }
}
