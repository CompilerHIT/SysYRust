use std::collections::HashMap;
use std::fs::File;
use std::io::{Result, Write};

use crate::ir::basicblock::BasicBlock;
use crate::ir::module::Module;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::function::Function;
use crate::backend::operand::Reg;
use crate::backend::structs::{IGlobalVar, FGlobalVar, GlobalVar};
use crate::backend::func::Func;
use crate::backend::block::BB;
use crate::backend::operand::ToString;
use crate::utility::{ScalarType, ObjPtr};


#[derive(Clone)]    
pub struct AsmModule {
    reg_map: HashMap<i32, Reg>,

    global_var_list: Vec<GlobalVar>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    func_map: HashMap<&'static str, &'static Func>,
    block_map: HashMap<&'static str, &'static BB>,
}

impl AsmModule {
    pub fn new(ir_module: &Module) -> Self {
        let global_var_list = Self::get_global(ir_module);
        Self {
            reg_map: HashMap::new(),
            global_var_list,
            // global_fvar_list,
            func_map: HashMap::new(),
            block_map: HashMap::new(),
        }
    }
    
    pub fn build_lir(&self, ir_module: &Module) {
        let mut func_seq = 0;
        for (name, iter) in ir_module.function {
            func_seq += 1;
            let ir_func = iter.as_ref();
            let mut func = Func::new(name);
            func.construct(&self, ir_func, func_seq);
            self.func_map.insert(name, &func);
        }
    } 

    pub fn push_block(&self, label: &str, block: &BB) {
        self.block_map.insert(label, block);
    }

    pub fn generator(&mut self, f: &mut File) -> Result<()> {
        self.generate_global_var(f)?;
        Ok(())
    }


    fn get_global(ir_module: &Module) -> Vec<GlobalVar> {
        let map = ir_module.global_variable;
        let mut list = Vec::with_capacity(map.len());
        for (name, iter) in map {
            //TODO: update ir translation，to use ObjPtr match
            match iter.as_ref().get_kind() {
                InstKind::GlobalConstInt(value) => 
                    list.push(GlobalVar::IGlobalVar(
                        IGlobalVar::init(name.to_string(), value, true)
                    )),
                InstKind::GlobalConstFloat(value) => 
                    list.push(GlobalVar::FGlobalVar(
                        FGlobalVar::init(name.to_string(), value, true)
                    )),
                _ => panic!("fail to analyse GlobalConst"),
            };
            
        }
        list
    }

    fn generate_global_var(&self, f: &mut File) -> Result<()> {
        for iter in self.global_var_list.iter() {
            match iter {
                GlobalVar::IGlobalVar(ig) => {
                    let name = ig.get_name();
                    let value = ig.get_init().to_string();
                    writeln!(f, "{name}:\n        .word:   {value}")?;
                    writeln!(f, "    {value}")?;
                }
                GlobalVar::FGlobalVar(fg) => {
                    let name = fg.get_name();
                    let value = fg.get_init().to_hex_string();
                    writeln!(f, "{name}:\n        .word:   {value}")?;
                }
            }
        }
        Ok(())
    }
}