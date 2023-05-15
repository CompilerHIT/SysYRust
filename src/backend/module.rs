use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};

use crate::ir::instruction::global_const_int::GlobalConstInt;
use crate::ir::module::Module;
use crate::ir::instruction::Instruction;
use crate::ir::function::Function;
use crate::backend::operand::Reg;
use crate::backend::structs::{Func, IGlobalVar, FGlobalVar};
use crate::utility::{ScalarType, ObjPtr};


#[derive(Clone)]    
pub struct AsmModule {
    reg_mapping: HashMap<usize, Reg>,

    global_ivar_list: Vec<IGlobalVar>,
    // global_fvar_list: Vec<FGlobalVar>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    functions: HashMap<String, ObjPtr<Function>>,
    blocks: usize,
}

impl AsmModule {
    pub fn new(ir_module: &Module) -> Self {
        let global_ivar_list = Self::get_global_int(ir_module);
        Self {
            reg_mapping: HashMap::new(),
            global_ivar_list,
            // global_fvar_list,
            functions: ir_module.function.clone(),
            blocks: 0,
        }
    }

    pub fn get_funcs(&self) -> &HashMap<String, ObjPtr<Function>> {
        &self.functions
    }

    pub fn get_blocks_num(&self) -> usize {
        self.blocks
    }

    pub fn get_reg_mapping(&self) -> &HashMap<usize, Reg> {
        &self.reg_mapping
    }

    pub fn set_reg_mapping(&mut self, reg: Reg, id: usize) {
        self.reg_mapping.insert(id, reg);
    }

    pub fn generator(&mut self, f: &mut File) -> Result<()> {
        self.generate_global_var(f)?;
        Ok(())
    }

    fn get_global_int(ir_module: &Module) -> Vec<IGlobalVar> {
        let map = ir_module.global_variable.clone();
        let mut list = Vec::with_capacity(map.len());
        for iter in map.iter() {
            let name = iter.0.to_string();
            //TODO: update ir translation，to use ObjPtr match
            // if let Some(value) = iter.1.borrow().as_any().downcast_ref::<GlobalConstInt>() {
            //     list.push(IGlobalVar::init(name, value.get_bonding()))
            // } else {
            //     panic!("fail to analyse GlobalConstInt");
            // }
        }
        list
    }

    fn generate_global_var(&self, f: &mut File) -> Result<()> {
        for iter in self.global_ivar_list.iter() {
            let name = iter.get_name();
            let value = iter.get_init().get_data();
            writeln!(f, "{name}:")?;
            writeln!(f, "    {value}")?;
        }
        Ok(())
    }
}