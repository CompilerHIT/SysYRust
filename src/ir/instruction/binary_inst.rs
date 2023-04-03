use super::super::instruction::Instruction;
use super::super::ir_type::IrType;
use crate::ir::user::User;
use std::rc::Rc;

pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct BinaryOpInst {
    user: User,
    operator: Operator,
    operand: Vec<Rc<Instruction>>,
}

impl BinaryOpInst {
    fn make_binary_op_inst(
        name: String,
        ir_type: IrType,
        operator: Operator,
        operand: Vec<Rc<Instruction>>,
    ) -> Rc<BinaryOpInst> {
        let user = User::make_user(name, ir_type);
        Rc::new(BinaryOpInst {
            user,
            operator,
            operand,
        })
    }

    /// 构造一个加指令
    pub fn make_add_inst(name: String, operand: Vec<Rc<Instruction>>) -> Rc<BinaryOpInst> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Add, operand)
    }

    /// 构造一个加指令
    pub fn make_sub_inst(name: String, operand: Vec<Rc<Instruction>>) -> Rc<BinaryOpInst> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Sub, operand)
    }

    /// 构造一个加指令
    pub fn make_mul_inst(name: String, operand: Vec<Rc<Instruction>>) -> Rc<BinaryOpInst> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Mul, operand)
    }

    /// 构造一个加指令
    pub fn make_div_inst(name: String, operand: Vec<Rc<Instruction>>) -> Rc<BinaryOpInst> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Div, operand)
    }

    /// 获得操作符
    ///
    /// # Panics
    /// 左操作数不存在，是空指针
    pub fn get_operator(&self) -> &Operator {
        &self.operator
    }

    /// 获得左操作数
    pub fn get_lhs(&mut self) -> &mut Instruction {
        let op = &mut self.operand[0];
        Rc::get_mut(op).unwrap()
    }

    /// 获得右操作数
    ///
    /// # Panics
    /// 右操作数不存在，是空指针
    pub fn get_rhs(&mut self) -> &mut Instruction {
        let op = &mut self.operand[1];
        Rc::get_mut(op).unwrap()
    }
}
