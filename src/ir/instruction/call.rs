///! 本文件为函数调用指令的实现
use super::*;
impl ObjPool<Inst> {
    /// 创建一个返回int值的函数调用指令
    /// # Arguments
    /// * `callee` - 被调用的函数
    /// * `args` - 参数列表
    pub fn make_int_call(&mut self, callee: &str, args: Vec<ObjPtr<Inst>>) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(IrType::Int, InstKind::Call(callee), args));

        // 设置use_list
        for arg in args {
            arg.as_mut().add_user(inst.as_ref());
        }
        inst
    }

    /// 创建一个返回void值的函数调用指令
    /// # Arguments
    /// * `callee` - 被调用的函数
    /// * `args` - 参数列表
    pub fn make_void_call(&mut self, callee: &str, args: Vec<ObjPtr<Inst>>) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(IrType::Void, InstKind::Call(callee), args));

        // 设置use_list
        for arg in args {
            arg.as_mut().add_user(inst.as_ref());
        }
        inst
    }

    /// 创建一个返回float值的函数调用指令
    /// # Arguments
    /// * `callee` - 被调用的函数
    /// * `args` - 参数列表
    pub fn make_float_call(&mut self, callee: &str, args: Vec<ObjPtr<Inst>>) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(IrType::Float, InstKind::Call(callee), args));

        // 设置use_list
        for arg in args {
            arg.as_mut().add_user(inst.as_ref());
        }
        inst
    }
}
impl Inst {
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

    /// 修改函数调用指令的参数
    /// # Arguments
    /// * `index` - 参数的索引
    /// * `args` - 新的参数
    pub fn set_args(&mut self, index: usize, args: ObjPtr<Inst>) {
        // 修改参数时，需要将原来的参数从use list中删除
        self.user.get_operand(index).as_mut().remove_user(self);
        args.as_mut().add_user(self);
        self.user.set_operand(index, args);
    }
}
