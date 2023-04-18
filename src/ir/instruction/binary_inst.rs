use crate::ir::{instruction::*, ir_type::IrType, user::User};
use crate::utility::Pointer;
use std::cell::RefMut;

pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct BinaryOpInst {
    user: User,
    operator: Operator,
}

impl BinaryOpInst {
    fn make_binary_op_inst(
        ir_type: IrType,
        operator: Operator,
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![lhs, rhs]);
        let inst = BinaryOpInst { user, operator };
        Pointer::new(Box::new(inst))
    }

    /// 构造一个加指令
    pub fn make_add_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_binary_op_inst(IrType::Int, Operator::Add, lhs, rhs)
    }

    /// 构造一个减指令
    pub fn make_sub_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_binary_op_inst(IrType::Int, Operator::Sub, lhs, rhs)
    }

    /// 构造一个乘指令
    pub fn make_mul_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_binary_op_inst(IrType::Int, Operator::Mul, lhs, rhs)
    }

    /// 构造一个除指令
    pub fn make_div_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_binary_op_inst(IrType::Int, Operator::Div, lhs, rhs)
    }

    /// 获得操作符
    pub fn get_operator(&self) -> &Operator {
        &self.operator
    }

    // 获得左操作数
    // # Panics
    // 左操作数不存在，是空指针

    // 获得右操作数
    //
    // # Panics
    // 右操作数不存在，是空指针
}

impl Instruction for BinaryOpInst {
    fn get_type(&self) -> InstructionType {
        InstructionType::IBinaryOpInst
    }
}
