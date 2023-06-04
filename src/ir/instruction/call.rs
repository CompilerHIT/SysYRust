///! 本文件为函数调用指令的实现
use super::*;
impl ObjPool<Inst> {
    /// 创建一个返回int值的函数调用指令
    /// # Arguments
    /// * `callee` - 被调用的函数
    /// * `args` - 参数列表
    pub fn make_int_call(&mut self, callee: String, args: Vec<ObjPtr<Inst>>) -> ObjPtr<Inst> {
        // 正确性检查
        for arg in args.clone().iter() {
            let arg = arg.as_ref();
            if let InstKind::Parameter = arg.get_kind() {
            } else {
                unreachable!("Inst::make_int_call")
            }
        }

        let inst = self.put(Inst::new(IrType::Int, InstKind::Call(callee), args.clone()));

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
    pub fn make_void_call(&mut self, callee: String, args: Vec<ObjPtr<Inst>>) -> ObjPtr<Inst> {
        // 正确性检查
        for arg in args.clone().iter() {
            let arg = arg.as_ref();
            if let InstKind::Parameter = arg.get_kind() {
            } else {
                unreachable!("Inst::make_void_call")
            }
        }

        let inst = self.put(Inst::new(
            IrType::Void,
            InstKind::Call(callee),
            args.clone(),
        ));

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
    pub fn make_float_call(&mut self, callee: String, args: Vec<ObjPtr<Inst>>) -> ObjPtr<Inst> {
        // 正确性检查
        for arg in args.clone().iter() {
            let arg = arg.as_ref();
            if let InstKind::Parameter = arg.get_kind() {
            } else {
                unreachable!("Inst::make_float_call")
            }
        }

        let inst = self.put(Inst::new(
            IrType::Float,
            InstKind::Call(callee),
            args.clone(),
        ));

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
        // 正确性检查
        if let InstKind::Call(_) = self.kind {
        } else {
            unreachable!("Inst::get_callee")
        }

        match &self.kind {
            InstKind::Call(callee) => callee.as_str(),
            _ => panic!("not a call inst"),
        }
    }

    /// 获得函数调用指令的参数列表
    pub fn get_args(&self) -> &Vec<ObjPtr<Inst>> {
        // 正确性检查
        if let InstKind::Call(_) = self.kind {
        } else {
            unreachable!("Inst::get_args")
        }

        self.user.get_operands()
    }

    /// 找到参数对应的索引
    /// # Arguments
    /// * `arg` - 参数
    /// # Return
    /// 参数对应的索引
    pub fn find_arg_index(&self, arg: &Inst) -> Option<usize> {
        // 正确性检查
        if let InstKind::Call(_) = self.kind {
        } else {
            unreachable!("Inst::find_arg_index")
        }

        self.user.find_operand(arg)
    }

    /// 修改函数调用指令的参数
    /// # Arguments
    /// * `index` - 参数的索引
    /// * `arg` - 新的参数
    pub fn set_arg(&mut self, index: usize, arg: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Call(_) = self.kind {
            if let InstKind::Parameter = arg.as_ref().get_kind() {
                debug_assert!(index < self.user.get_operands_size())
            } else {
                unreachable!("Inst::set_arg")
            }
        } else {
            unreachable!("Inst::set_arg")
        }

        // 修改参数时，需要将原来的参数从use list中删除
        self.user.get_operand(index).as_mut().remove_user(self);
        arg.as_mut().add_user(self);
        self.user.set_operand(index, arg);
    }
}
