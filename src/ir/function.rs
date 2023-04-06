use std::{cell::RefCell, rc::Rc};

use super::basicblock::BasicBlock;
use super::parameter::Parameter;
use super::value::Value;

pub struct Function {
    value: Value,
    parameters: Vec<Parameter>,
    head_block: Rc<RefCell<BasicBlock>>,
}

impl Function {
    fn make_function(
        name: String,
        parameters: Vec<Parameter>,
        head_block: Rc<RefCell<BasicBlock>>,
    ) -> Function {
        Function {
            value: Value::make_value(name, super::ir_type::IrType::Function),
            parameters,
            head_block,
        }
    }
}
