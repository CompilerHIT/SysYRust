///! 本文件为 store 指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 存储一个int值
    /// # Arguments
    /// * 'dest' - 指向被存储空间的指针
    /// * 'value' - 需要存储的值'
    /// # Return
    /// 返回一个Inst实例
    pub fn make_int_store(&mut self, dest: ObjPtr<Inst>, value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Void,
            InstKind::Store,
            vec![dest, value],
        ));

        // 设置use list
        dest.as_mut().add_user(inst.as_ref());
        value.as_mut().add_user(inst.as_ref());

        inst
    }

    /// 存储一个float值
    /// # Arguments
    /// * 'dest' - 指向被存储空间的指针
    /// * 'value' - 需要存储的值'
    /// # Return
    /// 返回一个Inst实例
    pub fn make_float_store(&mut self, dest: ObjPtr<Inst>, value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Void,
            InstKind::Store,
            vec![dest, value],
        ));

        // 设置use list
        dest.as_mut().add_user(inst.as_ref());
        value.as_mut().add_user(inst.as_ref());

        inst
    }
}

impl Inst {
    /// 获得存储的目标地址
    /// # Return
    /// 目标地址指令的引用
    pub fn get_dest(&self) -> ObjPtr<Inst> {
        self.user.get_use_list()[0]
    }

    /// 修改存储的目标地址
    /// # Arguments
    /// * 'dest' - 新的目标地址
    pub fn set_dest(&mut self, dest: ObjPtr<Inst>) {
        // 设置use list
        self.user.get_operand(0).as_mut().remove_user(self);
        dest.as_mut().add_user(self);

        self.user.set_operand(0, dest);
    }

    /// 获得存储的值
    /// # Return
    /// 值指令的引用
    pub fn get_value(&self) -> ObjPtr<Inst> {
        self.user.get_use_list()[1]
    }

    /// 修改存储的值
    /// # Arguments
    /// * 'value' - 新的值
    pub fn set_value(&mut self, value: ObjPtr<Inst>) {
        // 设置use list
        self.user.get_operand(1).as_mut().remove_user(self);
        value.as_mut().add_user(self);

        self.user.set_operand(1, value);
    }
}
