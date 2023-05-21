///! 此文件为return指令的实现文件
use super::*;

impl ObjPool<Inst> {
    /// 创建return指令
    /// # Arguments
    /// * 'value' - 返回值
    pub fn make_return(&mut self, value: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(
            value.as_ref().get_ir_type(),
            InstKind::Return,
            vec![value],
        ));

        // 设置use list
        value.as_mut().add_user(inst.as_ref());

        inst
    }
}

impl Inst {
    /// 设置返回值
    pub fn set_return_value(&mut self, value: ObjPtr<Inst>) {
        // 设置use_list
        self.user.get_operand(0).as_mut().remove_user(self);
        value.as_mut().add_user(self);

        self.user.set_operand(0, value);
    }

    /// 获得返回值
    pub fn get_return_value(&self) -> ObjPtr<Inst> {
        self.user.get_operand(0)
    }
}
