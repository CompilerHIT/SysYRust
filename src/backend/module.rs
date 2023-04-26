use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};

use crate::ir::instruction::global_const_int::GlobalConstInt;
use crate::ir::module::Module;
use crate::ir::instruction::Instruction;
use crate::ir::function::Function;
use crate::backend::operand::Reg;
use crate::backend::structs::Func;
use crate::utility::{Pointer, ScalarType};

use super::structs::GlobalVar;

#[derive(Clone)]
pub struct AsmModule {
    reg_mapping: HashMap<usize, Reg>,

    // TODO: add Float global var, tmp for i32
    global_var_list: Vec<GlobalVar<i32>>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    functions: HashMap<String, Pointer<Function>>,
}

impl AsmModule {
    pub fn new(ir_module: &Module) -> Self {
        let global_var_list = Self::get_global_int(ir_module);
        Self {
            reg_mapping: HashMap::new(),
            global_var_list,
            functions: ir_module.function.clone(),
        }
    }

    pub fn generator(&mut self, f: &mut File) -> Result<()> {
        self.generate_global_var(f)?;
        Ok(())
    }

    fn get_global_int(ir_module: &Module) -> Vec<GlobalVar<i32>> {
        let map = ir_module.global_variable.clone();
        let mut list = Vec::with_capacity(map.len());
        for iter in map.iter() {
            let name = iter.0.to_string();
            if let Some(value) = iter.1.borrow().as_any().downcast_ref::<GlobalConstInt>() {
                list.push(
                    GlobalVar::new(
                        name,
                        value.get_bonding(),
                        ScalarType::Int,
                    ))
            } else {
                panic!("fail to analyse GlobalConstInt");
            }
        }
        list
    }

    fn generate_global_var(&self, f: &mut File) -> Result<()> {
        for iter in self.global_var_list.iter() {
            let name = iter.get_name();
            let value = iter.get_value();
            writeln!(f, "{name}:")?;
            writeln!(f, "    {value}")?;
        }
        Ok(())
    }
}