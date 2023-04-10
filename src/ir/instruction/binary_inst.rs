use crate::ir::{instruction::Instruction, ir_type::IrType, user::User};
use crate::utility::Pointer;
use std::cell::RefMut;

#[derive(Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug)]
pub struct BinaryOpInst {
    user: User,
    operator: Operator,
}

impl BinaryOpInst {
    fn make_binary_op_inst(
        ir_type: IrType,
        operator: Operator,
        lhs: Pointer<Instruction>,
        rhs: Pointer<Instruction>,
    ) -> Pointer<Instruction> {
        let user = User::make_user(ir_type, vec![lhs, rhs]);
        let inst = BinaryOpInst { user, operator };
        Pointer::new(Instruction::IBinaryOpInst(inst))
    }

    /// 构造一个加指令
    pub fn make_add_inst(
        lhs: Pointer<Instruction>,
        rhs: Pointer<Instruction>,
    ) -> Pointer<Instruction> {
        Self::make_binary_op_inst(IrType::Int, Operator::Add, lhs, rhs)
    }

    /// 构造一个加指令
    pub fn make_sub_inst(
        lhs: Pointer<Instruction>,
        rhs: Pointer<Instruction>,
    ) -> Pointer<Instruction> {
        Self::make_binary_op_inst(IrType::Int, Operator::Sub, lhs, rhs)
    }

    /// 构造一个加指令
    pub fn make_mul_inst(
        lhs: Pointer<Instruction>,
        rhs: Pointer<Instruction>,
    ) -> Pointer<Instruction> {
        Self::make_binary_op_inst(IrType::Int, Operator::Mul, lhs, rhs)
    }

    /// 构造一个加指令
    pub fn make_div_inst(
        lhs: Pointer<Instruction>,
        rhs: Pointer<Instruction>,
    ) -> Pointer<Instruction> {
        Self::make_binary_op_inst(IrType::Int, Operator::Div, lhs, rhs)
    }

    /// 获得操作符
    pub fn get_operator(&self) -> &Operator {
        &self.operator
    }

    /// 获得左操作数
    /// # Panics
    /// 左操作数不存在，是空指针
    pub fn get_lhs(&self) -> RefMut<Instruction> {
        self.user.get_operand(0)
    }

    /// 获得右操作数
    ///
    /// # Panics
    /// 右操作数不存在，是空指针
    pub fn get_rhs(&self) -> RefMut<Instruction> {
        self.user.get_operand(1)
    }
}

#[test]
fn test_make_binary_op_inst() {
    let p = BinaryOpInst::make_add_inst(
        String::from("add"),
        super::ConstInt::make_int(String::from("lhs"), 1),
        super::ConstInt::make_int(String::from("rhs"), 2),
    );

    match *(*p).borrow_mut() {
        Instruction::IBinaryOpInst(ref b) => println!("{:?}", b),
        _ => panic!("not a BinaryOpInst!"),
    };
}
