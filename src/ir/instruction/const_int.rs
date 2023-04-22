use super::*;
use crate::ir::ir_type::IrType;
use crate::ir::user::User;
use crate::utility::Pointer;

pub struct ConstInt {
    user: User,
    bonding: i32,
    list: IList,
}

impl ConstInt {
    pub fn make_int(bonding: i32) -> Pointer<Box<dyn Instruction>> {
        let user = User::make_user(IrType::Int, vec![]);
        Pointer::new(Box::new(ConstInt {
            user,
            bonding,
            list: IList {
                prev: None,
                next: None,
            },
        }))
    }

    pub fn get_bonding(&self) -> i32 {
        self.bonding
    }
}

impl Instruction for ConstInt {
    fn get_type(&self) -> InstructionType {
        InstructionType::IConstInt
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
        self.list.insert_before(node);
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

    fn set_next(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.set_next(node);
    }

    fn set_prev(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.list.set_prev(node);
    }
}
