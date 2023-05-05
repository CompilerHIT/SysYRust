use std::collections::HashMap;

use crate::utility::Pointer;

use super::{function_cfg::CfgFunction, instruction_cfg::CfgInstruction};

pub struct CfgModule {
    pub global_variable: HashMap<String, Pointer<Box<dyn CfgInstruction>>>,
    pub function: HashMap<String, Pointer<CfgFunction>>,
}

impl CfgModule {
    pub fn make_module() -> CfgModule {
        CfgModule {
            global_variable: HashMap::new(),
            function: HashMap::new(),
        }
    }

    pub fn push_var(&mut self, name: &String, variable: Pointer<Box<dyn CfgInstruction>>) {
        match self.global_variable.get(name) {
            None => self.global_variable.insert(name.to_string(), variable),
            Some(_) => panic!("Inser global variable error, the name has presented!"),
        };
    }

    pub fn push_function(&mut self, name: &String, function: Pointer<CfgFunction>) {
        match self.function.get(name) {
            None => self.function.insert(name.to_string(), function),
            Some(_) => panic!("Inser function error, the name has presented!"),
        };
    }
}

// impl Deref for CfgModule

// impl Send for CfgModule {}
