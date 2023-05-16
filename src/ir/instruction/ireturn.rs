///! 此文件为return指令的实现文件
use super::*;

impl Inst {
    /// 创建return指令
    /// # Arguments
    /// * 'value' - 返回值
    pub fn make_return(value: ObjPtr<Inst>) -> Self {
        Self {
            user: User::new(IrType::Void, vec![value]),
            list: IList {
                prev: None,
                next: None,
            },
            kind: InstKind::Return,
        }
    }

    /// 获得返回值
    pub fn get_return_value(&self) -> ObjPtr<Inst> {
        self.user.get_operand(0)
    }
}
