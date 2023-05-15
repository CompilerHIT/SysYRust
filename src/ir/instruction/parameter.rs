///! 此文件为参数指令的实现
use super::*;

impl Inst {
    /// 创建参数指令
    pub fn new_param(ir_type: IrType) -> Self {
        Self {
            user: User::new(ir_type, vec![]),
            list: IList {
                prev: None,
                next: None,
            },
            kind: InstKind::Parameter,
        }
    }

    /// 获得参数的类型
    pub fn get_param_type(&self) -> IrType {
        self.user.get_ir_type()
    }
}
