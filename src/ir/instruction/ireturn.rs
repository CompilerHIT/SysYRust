///! 此文件为return指令的实现文件
use super::*;

impl ObjPool<Inst> {
    /// 创建return指令
    /// # Arguments
    /// * 'value' - 返回值
    pub fn make_return(&mut self, mut value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(
            value.get_ir_type(),
            InstKind::Return,
            vec![value],
        ));

        // 设置use list
        value.add_user(inst.as_ref());

        inst
    }

    /// 创建return void指令
    pub fn make_return_void(&mut self) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(IrType::Void, InstKind::Return, vec![]));

        inst
    }
}

impl Inst {
    /// 设置返回值
    pub fn set_return_value(&mut self, mut value: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Return = self.get_kind() {
            if value.get_ir_type() != IrType::Void {
                debug_assert_eq!(self.get_use_list().len(), 0)
            } else {
                unreachable!("Inst::set_return_value")
            }
        } else {
            unreachable!("Inst::set_return_value")
        }

        // 设置use_list
        self.user.get_operand(0).as_mut().remove_user(self);
        value.add_user(self);

        self.user.set_operand(0, value);
    }

    /// 获得返回值
    pub fn get_return_value(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::Return = self.get_kind() {
            debug_assert_eq!(self.get_use_list().len(), 0);
            if self.get_ir_type() != IrType::Void {
            } else {
                unreachable!("Inst::get_return_value")
            }
        } else {
            unreachable!("Inst::get_return_value")
        }

        self.user.get_operand(0)
    }

    /// 判断是否为return指令
    pub fn is_return(&self) -> bool {
        InstKind::Return == self.get_kind()
    }

    /// 判断是否为return void指令
    pub fn is_void_return(&self) -> bool {
        InstKind::Return == self.get_kind() && self.get_ir_type() == IrType::Void
    }

    /// 判断是否为return value指令
    pub fn is_value_return(&self) -> bool {
        InstKind::Return == self.get_kind() && self.get_ir_type() != IrType::Void
    }
}
