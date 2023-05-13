use std::any::Any;

use crate::{
    ir::{function::Function, ir_type::IrType, user::User},
    utility::Pointer,
};

use super::{IList, Instruction, InstructionType};

pub struct CallInst {
    user: User,
    callee: Pointer<Function>,
    list: IList,
}

impl CallInst {
    fn make_call_inst(
        ir_type: IrType,
        callee: Pointer<Function>,
        args: Vec<Pointer<Box<dyn Instruction>>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, args);
        let inst = CallInst {
            user,
            callee,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    /// 构造一个返回int类型的函数调用
    pub fn make_int_call(
        callee: Pointer<Function>,
        args: Vec<Pointer<Box<dyn Instruction>>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_call_inst(IrType::Int, callee, args)
    }

    /// 构造一个返回bool类型的函数调用
    pub fn make_bool_call(
        callee: Pointer<Function>,
        args: Vec<Pointer<Box<dyn Instruction>>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_call_inst(IrType::Bool, callee, args)
    }

    /// 构造一个返回Void类型的函数调用
    pub fn make_void_call(
        callee: Pointer<Function>,
        args: Vec<Pointer<Box<dyn Instruction>>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_call_inst(IrType::Void, callee, args)
    }

    /// 获取被调用的函数
    pub fn get_callee(&self) -> Pointer<Function> {
        self.callee.clone()
    }

    /// 获取参数列表
    pub fn get_args(&self) -> &Vec<Pointer<Box<dyn Instruction>>> {
        self.user.get_operands()
    }
    pub fn get_args_mut(&mut self) -> &mut Vec<Pointer<Box<dyn Instruction>>> {
        self.user.get_operands_mut()
    }
}

impl Instruction for CallInst {
    fn get_inst_type(&self) -> InstructionType {
        InstructionType::ICallInst
    }
    fn get_value_type(&self) -> IrType {
        self.user.get_ir_type()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn next(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }

    fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.prev()
    }

    fn insert_before(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.insert_before(node)
    }

    fn insert_after(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.insert_after(node)
    }

    fn is_head(&self) -> bool {
        self.list.is_head()
    }

    fn is_tail(&self) -> bool {
        self.list.is_tail()
    }

    fn set_next(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.set_next(node);
    }

    fn set_prev(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(node);
    }

    fn remove_self(&mut self) {
        self.list.remove_self();
    }
}
