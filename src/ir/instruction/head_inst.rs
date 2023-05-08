use crate::ir::instruction::{Instruction, InstructionType};
use crate::utility::Pointer;

pub struct HeadInst {
    prev: Option<Pointer<Box<dyn Instruction>>>,
    next: Option<Pointer<Box<dyn Instruction>>>,
}

impl HeadInst {
    pub fn new() -> Self {
        HeadInst {
            prev: None,
            next: None,
        }
    }
}

impl Instruction for HeadInst {
    fn get_type(&self) -> InstructionType {
        InstructionType::IHead
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn next(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        match &self.next {
            Some(node) => Some(node.clone()),
            None => None,
        }
    }

    fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        match &self.prev {
            Some(node) => Some(node.clone()),
            None => None,
        }
    }

    fn set_prev(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.prev = Some(node);
    }

    fn set_next(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.next = Some(node);
    }

    fn insert_before(&mut self, _node: Pointer<Box<dyn Instruction>>) {}

    fn insert_after(&mut self, _node: Pointer<Box<dyn Instruction>>) {}

    fn is_head(&self) -> bool {
        false
    }
    fn is_tail(&self) -> bool {
        false
    }
    fn remove_self(&mut self) {}
}
