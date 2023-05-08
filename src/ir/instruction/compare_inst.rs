use crate::ir::instruction::binary_inst::Operator;
use crate::ir::{instruction::*, ir_type::IrType, user::User};
use crate::utility::Pointer;

pub struct CompareInst {
    user: User,
    operator: Operator,
    list: IList,
}

impl CompareInst {
    fn make_compare_inst(
        ir_type: IrType,
        operator: Operator,
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![lhs, rhs]);
        let inst = CompareInst {
            user,
            operator,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    /// 构造一个小于指令
    pub fn make_lesser_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_compare_inst(IrType::Bool, Operator::Lesser, lhs, rhs)
    }

    /// 构造一个大于指令
    pub fn make_grater_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_compare_inst(IrType::Bool, Operator::Grater, lhs, rhs)
    }

    /// 构造一个等于指令
    pub fn make_equal_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_compare_inst(IrType::Bool, Operator::Equal, lhs, rhs)
    }

    /// 构造一个小于等于指令
    pub fn make_lesser_equal_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_compare_inst(IrType::Bool, Operator::LesserEqual, lhs, rhs)
    }

    /// 构造一个大于等于指令
    pub fn make_grater_equal_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_compare_inst(IrType::Bool, Operator::GraeterEqual, lhs, rhs)
    }

    /// 构造一个不等于指令
    pub fn make_not_equal_inst(
        lhs: Pointer<Box<dyn Instruction>>,
        rhs: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_compare_inst(IrType::Bool, Operator::NotEqual, lhs, rhs)
    }

    /// 获取操作符
    pub fn get_operator(&self) -> Operator {
        self.operator
    }

    /// 获取左操作数
    pub fn get_lhs(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(0)
    }
}

impl Instruction for CompareInst {
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
