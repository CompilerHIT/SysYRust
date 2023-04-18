use super::{basicblock::BasicBlock, ir_type::IrType, parameter::Parameter, value::Value};
use crate::utility::Pointer;
use std::cell::RefMut;

pub struct Function {
    value: Value,
    parameters: Vec<Parameter>,
    head_block: Pointer<BasicBlock>,
}

impl Function {
    fn make_function(parameters: Vec<Parameter>, head_block: Pointer<BasicBlock>) -> Function {
        Function {
            value: Value::make_value(IrType::Function),
            parameters,
            head_block,
        }
    }

    pub fn get_head(&self) -> RefMut<BasicBlock> {
        self.head_block.borrow_mut()
    }
}
