///! 本文件为函数调用指令的实现
use super::*;

impl Inst {
    /// 创建一个返回int值的函数调用指令
    /// # Arguments
    /// * `callee` - 被调用的函数
    /// * `args` - 参数列表
    pub fn make_int_call(callee: &str, args: Vec<ObjPtr<Inst>>) -> Self {
        Self {
            user: User::new(IrType::Int, args),
            list: IList {
                prev: None,
                next: None,
            },
            kind: InstKind::Call(callee),
        }
    }

    /// 创建一个返回void值的函数调用指令
    /// # Arguments
    /// * `callee` - 被调用的函数
    /// * `args` - 参数列表
    pub fn make_void_call(callee: &str, args: Vec<ObjPtr<Inst>>) -> Self {
        Self {
            user: User::new(IrType::Void, args),
            list: IList {
                prev: None,
                next: None,
            },
            kind: InstKind::Call(callee),
        }
    }

    /// 创建一个返回float值的函数调用指令
    /// # Arguments
    /// * `callee` - 被调用的函数
    /// * `args` - 参数列表
    pub fn make_float_call(callee: &str, args: Vec<ObjPtr<Inst>>) -> Self {
        Self {
            user: User::new(IrType::Float, args),
            list: IList {
                prev: None,
                next: None,
            },
            kind: InstKind::Call(callee),
        }
    }

    /// 获得函数调用指令的被调用函数名
    pub fn get_callee(&self) -> &str {
        match self.kind {
            InstKind::Call(callee) => callee,
            _ => panic!("not a call inst"),
        }
    }

    /// 获得函数调用指令的参数列表
    pub fn get_args(&self) -> &Vec<ObjPtr<Inst>> {
        self.user.get_operands()
    }
}
