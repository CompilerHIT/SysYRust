use crate::ir::{instruction::*, ir_type::IrType, user::User};
use crate::utility::Pointer;

pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct BinaryOpInst {
    user: User,
    operator: Operator,
    list: IList,
}

impl BinaryOpInst {
    fn make_binary_op_inst(
        ir_type: IrType,
        operator: Operator,
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![lhs, rhs]);
        let inst = BinaryOpInst {
            user,
            operator,
            list: IList {
                prev: None,
                next: None,
            },
        };
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

    // 获得操作符
    pub fn get_operator(&self) -> &Operator {
        &self.operator
    }

    // 获得左操作数
    // # Panics
    // 左操作数不存在，是空指针
    pub fn get_lhs(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(0)
    }

    // 获得右操作数
    //
    // # Panics
    // 右操作数不存在，是空指针
    pub fn get_rhs(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(1)
    }
}

impl Instruction for BinaryOpInst {
    fn get_type(&self) -> InstructionType {
        InstructionType::IBinaryOpInst
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
