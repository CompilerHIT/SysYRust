use crate::ir::instruction::Instruction;
use crate::ir::ir_type::IrType;
use crate::ir::user::User;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;

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
        name: String,
        ir_type: IrType,
        operator: Operator,
        lhs: Rc<RefCell<Instruction>>,
        rhs: Rc<RefCell<Instruction>>,
    ) -> Rc<RefCell<Instruction>> {
        let user = User::make_user(name, ir_type, vec![lhs, rhs]);
        let inst = BinaryOpInst { user, operator };
        Rc::new(RefCell::new(Instruction::IBinaryOpInst(inst)))
    }

    /// 构造一个加指令
    pub fn make_add_inst(
        name: String,
        lhs: Rc<RefCell<Instruction>>,
        rhs: Rc<RefCell<Instruction>>,
    ) -> Rc<RefCell<Instruction>> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Add, lhs, rhs)
    }

    /// 构造一个加指令
    pub fn make_sub_inst(
        name: String,
        lhs: Rc<RefCell<Instruction>>,
        rhs: Rc<RefCell<Instruction>>,
    ) -> Rc<RefCell<Instruction>> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Sub, lhs, rhs)
    }

    /// 构造一个加指令
    pub fn make_mul_inst(
        name: String,
        lhs: Rc<RefCell<Instruction>>,
        rhs: Rc<RefCell<Instruction>>,
    ) -> Rc<RefCell<Instruction>> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Mul, lhs, rhs)
    }

    /// 构造一个加指令
    pub fn make_div_inst(
        name: String,
        lhs: Rc<RefCell<Instruction>>,
        rhs: Rc<RefCell<Instruction>>,
    ) -> Rc<RefCell<Instruction>> {
        Self::make_binary_op_inst(name, IrType::Int, Operator::Div, lhs, rhs)
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
