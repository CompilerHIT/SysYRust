use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};

use crate::backend::operand::Reg;
use crate::backend::structs::GlobalVar;
use crate::backend::structs::Func;
use crate::utility::Pointer;

#[derive(Clone)]
pub struct AsmModule {
    reg_mapping: HashMap<usize, Reg>,

    //TODO: add global mapping: complete init pointer to make sure empty or not
    global_mapping: HashMap<String, GlobalVar>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    functions: Vec<Pointer<Func>>,
}

impl AsmModule {
    pub fn new() -> Self {
        Self {
            reg_mapping: HashMap::new(),
            global_mapping: HashMap::new(),
            functions: Vec::new(),
        }
    }
    pub fn generator(&mut self, f: &mut File) -> Result<()> {
        self.generate_global_var(f)?;
        Ok(())
    }

    fn generate_global_var(&self, f: &mut File) -> Result<()> {
        for iter in self.global_mapping.clone() {
            let name = iter.0;
            let value = iter.1.size;
            writeln!(f, "{name}:")?;
            writeln!(f, "    {value}")?;
        }
        Ok(())
    }
}