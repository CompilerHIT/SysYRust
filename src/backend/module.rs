use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};
use std::ops::Deref;

use crate::ir::instruction::global_const_int::GlobalConstInt;
use crate::ir::module::Module;
use crate::ir::instruction::Instruction;
use crate::ir::function::Function;
use crate::backend::operand::Reg;
use crate::backend::structs::Func;
use crate::utility::Pointer;

#[derive(Clone)]
pub struct AsmModule {
    reg_mapping: HashMap<usize, Reg>,

    //TODO: add global mapping: complete init pointer to make sure empty or not
    global_mapping: HashMap<String, Pointer<Box<dyn Instruction>>>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    functions: HashMap<String, Pointer<Function>>,
}

impl AsmModule {
    pub fn new(ir_module: Module) -> Self {
        Self {
            reg_mapping: HashMap::new(),
            global_mapping: ir_module.global_variable.clone(),
            functions: ir_module.function.clone(),
        }
    }
    pub fn generator(&mut self, f: &mut File) -> Result<()> {
        self.generate_global_var(f)?;
        Ok(())
    }

    fn generate_global_var(&self, f: &mut File) -> Result<()> {
        for iter in self.global_mapping.iter() {
            let name = iter.0;
            if let Some(global) = iter.1.borrow().as_any().downcast_ref::<GlobalConstInt>(){
                let value = global.get_bonding();
                writeln!(f, "{name}:")?;
                writeln!(f, "    {value}")?;
            } else {
                panic!("fail to print");
            };  
        }
        Ok(())
    }
}