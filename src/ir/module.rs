use std::collections::HashMap;

use crate::utility::Pointer;

use super::{function::Function, instruction::Instruction};

#[derive(Debug)]
pub struct Module {
    pub global_variable: HashMap<String, Pointer<Instruction>>,
    pub function: HashMap<String, Pointer<Function>>,
}

impl Module {
    pub fn make_module() -> Module {
        Module {
            global_variable: HashMap::new(),
            function: HashMap::new(),
        }
    }

    pub fn push_var(&mut self, name: &String, variable: Pointer<Instruction>) {
        match self.global_variable.get(name) {
            None => self.global_variable.insert(name.to_string(), variable),
            Some(_) => panic!("Inser global variable error, the name has presented!"),
        };
    }

    pub fn push_function(&mut self, name: &String, function: Pointer<Function>) {
        match self.function.get(name) {
            None => self.function.insert(name.to_string(), function),
            Some(_) => panic!("Inser function error, the name has presented!"),
        };
    }
}
