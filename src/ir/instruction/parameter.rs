///! 此文件为参数指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 创建参数指令
    /// # Arguments
    /// * 'ir_type' - 参数的类型
    pub fn make_param(&mut self, ir_type: IrType) -> ObjPtr<Inst> {
        self.put(Inst::new(ir_type, InstKind::Parameter, vec![]))
    }
}

impl Inst {
    /// 获得参数的类型
    pub fn get_param_type(&self) -> IrType {
        self.user.get_ir_type()
    }
}
