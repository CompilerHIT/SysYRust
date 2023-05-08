use super::{basicblock::BasicBlock, ir_type::IrType, parameter::Parameter, value::Value};
use crate::utility::Pointer;
use std::collections::HashMap;

pub struct Function {
    value: Value,
    parameters: HashMap<String, Pointer<Parameter>>,
    head_block: Pointer<BasicBlock>,
}

impl Function {
    fn make_function(
        parameters: HashMap<String, Pointer<Parameter>>,
        head_block: Pointer<BasicBlock>,
    ) -> Function {
        Function {
            value: Value::make_value(IrType::Function),
            parameters,
            head_block,
        }
    }

    pub fn get_head(&self) -> Pointer<BasicBlock> {
        self.head_block.clone()
    }

    pub fn get_parameter(&self, name: String) -> Option<Pointer<Parameter>> {
        match self.parameters.get(&name) {
            Some(p) => Some(p.clone()),
            None => None,
        }
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }
}
