use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

use crate::backend::func::Func;
use crate::backend::operand::ToString;
use crate::backend::structs::{FGlobalVar, GlobalVar, IGlobalVar};
use crate::ir::function::Function;
use crate::ir::instruction::InstKind;
use crate::ir::module::Module;
use crate::utility::{ObjPool, ObjPtr};

use super::structs::GenerateAsm;

pub struct AsmModule<'a> {
    global_var_list: Vec<GlobalVar>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    pub upper_module: &'a Module,
    func_mpool: ObjPool<Func>,
}

impl<'a> AsmModule<'a> {
    pub fn new(ir_module: &'a Module) -> Self {
        let global_var_list = Self::get_global(ir_module);
        Self {
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

    pub fn generator(&mut self, f: &mut File) {
        self.build_lir();
        self.allocate_reg(f);
        self.handle_spill();
        self.generate_global_var(f);
        //FIXME: generate array
        // self.gnerate_array(f);
        self.generate_asm(f);
    }

    fn allocate_reg(&mut self, f: &mut File) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            println!("allocate reg fun: {}", func.as_ref().label);
            func.as_mut().allocate_reg(f);
        });
    }

    fn handle_spill(&mut self) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            func.as_mut().handle_spill();
        });
    }

    fn get_global(ir_module: &Module) -> Vec<GlobalVar> {
        let map = &ir_module.global_variable;
        let mut list = Vec::with_capacity(map.len());
        for (name, iter) in map {
            //TODO: update ir translation，to use ObjPtr match
            match iter.as_ref().get_kind() {
                InstKind::GlobalConstInt(value) => list.push(GlobalVar::IGlobalVar(
                    IGlobalVar::init(name.to_string(), value, true),
                )),
                InstKind::GlobalConstFloat(value) => list.push(GlobalVar::FGlobalVar(
                    FGlobalVar::init(name.to_string(), value, true),
                )),
                _ => panic!("fail to analyse GlobalConst"),
            };
        }
        list
    }

    fn generate_global_var(&self, f: &mut File) {
        for iter in self.global_var_list.iter() {
            match iter {
                GlobalVar::IGlobalVar(ig) => {
                    let name = ig.get_name();
                    let value = ig.get_init().to_string();
                    writeln!(f, "{name}:\n        .word:   {value}\n");
                    writeln!(f, "    {value}\n");
                }
                GlobalVar::FGlobalVar(fg) => {
                    let name = fg.get_name();
                    let value = fg.get_init().to_hex_string();
                    writeln!(f, "{name}:\n        .word:   {value}\n");
                }
            }
        }
    }

    fn generate_asm(&mut self, f: &mut File) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            func.as_mut()
                .generate(ObjPtr::new(&crate::backend::structs::Context::new()), f);
        });
    }
}
