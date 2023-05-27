use std::collections::HashMap;
use std::fs::File;
use std::io::Result;
use std::fs::write;
use std::hash::{Hash, Hasher};

use crate::frontend::context::Context;
use crate::ir::basicblock::BasicBlock;
use crate::ir::module::Module;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::function::Function;
use crate::backend::operand::Reg;
use crate::backend::structs::{IGlobalVar, FGlobalVar, GlobalVar};
use crate::backend::func::Func;
use crate::backend::block::BB;
use crate::backend::operand::ToString;
use crate::utility::{ScalarType, ObjPtr, ObjPool};


use super::structs::GenerateAsm;

pub struct AsmModule {
    reg_map: HashMap<i32, Reg>,

    global_var_list: Vec<GlobalVar>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    upper_module: &'static Module,
    func_mpool: ObjPool<Func>,
}

impl AsmModule {
    pub fn new(ir_module: &'static Module) -> Self {
        let global_var_list = Self::get_global(ir_module);
        Self {
            reg_map: HashMap::new(),
            global_var_list,
            // global_fvar_list,
            func_map: Vec::new(),
            upper_module: ir_module,
            func_mpool: ObjPool::new(),
        }
    }
    
    pub fn build_lir(&mut self) {
        let mut func_seq = 0;
        for (name, iter) in &self.upper_module.function {
            let ir_func = iter.as_ref();
            let mut func = Func::new(name);
            func.construct(&self, ir_func, func_seq);
            let func_ptr = self.func_mpool.put(func);
            self.func_map.push((iter.clone(), func_ptr));
            func_seq += 1;
        }
    }

    pub fn generator(&mut self) {
        self.build_lir();
        self.allocate_reg();
        // self.generate_global_var(f.clone())?;
        self.generate_asm();
    }

    fn allocate_reg<'a>(&mut self) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            func.as_mut().allocate_reg();
        });
    }

    fn get_global(ir_module: &Module) -> Vec<GlobalVar> {
        let map = &ir_module.global_variable;
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

    fn generate_global_var(&self) -> Result<()> {
        for iter in self.global_var_list.iter() {
            match iter {
                GlobalVar::IGlobalVar(ig) => {
                    let name = ig.get_name();
                    let value = ig.get_init().to_string();
                    print!("{name}:\n        .word:   {value}\n");
                    print!("    {value}\n");
                }
                GlobalVar::FGlobalVar(fg) => {
                    let name = fg.get_name();
                    let value = fg.get_init().to_hex_string();
                    print!("{name}:\n        .word:   {value}\n");
                }
            }
        }
        Ok(())
    }

    fn generate_asm(&mut self){
        self.func_map.iter_mut().for_each(|(_, func)| {
            func.as_mut().generate(ObjPtr::new(&crate::backend::structs::Context::new()));
        });
    }
}