use super::{basicblock::BasicBlock, instruction::Instruction, ir_type::IrType, value::Value};
use crate::utility::Pointer;
use std::collections::HashMap;

pub struct Function {
    value: Value,
    parameters: HashMap<String, Pointer<Box<dyn Instruction>>>,
    head_block: Pointer<BasicBlock>,
}

impl Function {
    pub fn make_function(head_block: Pointer<BasicBlock>) -> Function {
        Function {
            value: Value::make_value(IrType::Function),
            parameters: HashMap::new(),
            head_block,
        }
    }

    pub fn get_head(&self) -> Pointer<BasicBlock> {
        self.head_block.clone()
    }

    pub fn set_parameter(&mut self, name: String, parameter: Pointer<Box<dyn Instruction>>) {
        self.parameters.insert(name, parameter);
    }

    pub fn get_parameter(&self, name: String) -> Option<Pointer<Box<dyn Instruction>>> {
        match self.parameters.get(&name) {
            Some(p) => Some(p.clone()),
            None => None,
        }
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }
}
