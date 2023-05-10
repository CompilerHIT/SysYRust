use crate::ir::{
    instruction::{IList, Instruction, InstructionType},
    ir_type::IrType,
    user::User,
};
use crate::utility::Pointer;

pub struct ReturnInst {
    user: User,
    list: IList,
}

impl ReturnInst {
    pub fn make_return_inst(
        ir_type: IrType,
        value: Pointer<Box<dyn Instruction>>,
    ) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(ir_type, vec![value]);
        let inst = ReturnInst {
            user,
            list: IList {
                prev: None,
                next: None,
            },
        };
        Pointer::new(Box::new(inst))
    }

    pub fn get_return_value(&self) -> Pointer<Box<dyn Instruction>> {
        self.user.get_operand(0)
    }
}

impl Instruction for ReturnInst {
    fn get_type(&self) -> super::InstructionType {
        InstructionType::IReturn
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn next(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.next()
    }

    fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        self.list.prev()
    }

    fn set_next(&mut self, next: Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(next)
    }

    fn set_prev(&mut self, prev: Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(prev)
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

    fn remove_self(&mut self) {
        self.list.remove_self()
    }
}
