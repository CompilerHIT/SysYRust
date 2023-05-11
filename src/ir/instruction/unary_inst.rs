use crate::ir::instruction::binary_inst::Operator;
use crate::ir::{instruction::*, ir_type::IrType, user::User};
use crate::utility::Pointer;

pub struct UnaryInst {
    user: User,
    operator: Operator,
    list: IList,
}

impl UnaryInst {
    fn make_unary_inst(
        ir_type: IrType,
        operator: Operator,
        operand: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![operand]);
        let inst = UnaryInst {
            user,
            operator,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    /// 构造一个取反指令
    pub fn make_not_inst(operand: Pointer<Box<dyn Instruction>>) -> Pointer<Box<dyn Instruction>> {
        Self::make_unary_inst(IrType::Bool, Operator::Not, operand)
    }

    /// 构造一个取负指令
    pub fn make_minus_inst(
        operand: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        Self::make_unary_inst(IrType::Int, Operator::Minus, operand)
    }

    /// 构造一个取正指令
    pub fn make_plus_inst(operand: Pointer<Box<dyn Instruction>>) -> Pointer<Box<dyn Instruction>> {
        Self::make_unary_inst(IrType::Int, Operator::Plus, operand)
    }

    /// 获取操作符
    pub fn get_operator(&self) -> Operator {
        self.operator
    }

    /// 获取操作数
    pub fn get_operand(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(0)
    }

    /// 修改操作数
    pub fn set_operand(&mut self, operand: Pointer<Box<dyn Instruction>>) {
        self.user.set_operand(0, operand);
    }
}

impl Instruction for UnaryInst {
    fn get_inst_type(&self) -> InstructionType {
        InstructionType::IUnaryOpInst
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
