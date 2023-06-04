use std::fs::File;
use std::io::Write;

use crate::ir::module::Module;
use crate::ir::instruction:: InstKind;
use crate::ir::function::Function;
use crate::backend::BackendPool;
use crate::backend::structs::{IGlobalVar, FGlobalVar, GlobalVar};
use crate::backend::func::Func;
use crate::backend::operand::ToString;
use crate::utility::ObjPtr;


use super::instrs::Context;
use super::structs::GenerateAsm;


pub struct AsmModule {
    global_var_list: Vec<GlobalVar>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    pub upper_module: ObjPtr<Module>,
}

impl AsmModule {
    pub fn new(ir_module: ObjPtr<Module>) -> Self {
        let global_var_list = Self::get_global(ir_module);
        Self {
            global_var_list,
            // global_fvar_list,
            func_map: Vec::new(),
            upper_module: ir_module,
        }
    }
    
    pub fn build_lir(&mut self, pool: &mut BackendPool) {
        let mut func_seq = 0;
        for (name, iter) in &self.upper_module.as_ref().function {
            let ir_func = iter.as_ref();
            let mut func = Func::new(name, pool.put_context(Context::new()));
            func.construct(&self, ir_func, func_seq, pool);
            let func_ptr = pool.put_func(func);
            self.func_map.push((iter.clone(), func_ptr));
            func_seq += 1;
        }
    }

    pub fn generator(&mut self, f: &mut File, pool: &mut BackendPool) {
        self.build_lir(pool);
        self.allocate_reg(f);
        self.handle_spill(pool);
        // 检查地址溢出，插入间接寻址
        // self.handle_overflow(f, pool);
        // 第二次分配寄存器
        self.allocate_reg(f);
        self.handle_spill(pool);
        self.generate_global_var(f);
        //FIXME: generate array
        // self.gnerate_array(f);
        self.generate_asm(f, pool);
    }

    // fn handle_overflow(&mut self, f: &mut File) {
    //     self.func_map.iter_mut().for_each(|(_, func)| {
    //         func.as_mut().handle_overflow(f);
    //     });
    // }

    fn allocate_reg(&mut self, f: &mut File) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            println!("allocate reg fun: {}", func.as_ref().label);
            func.as_mut().allocate_reg(f);
        });
    }

    fn handle_spill(&mut self, pool: &mut BackendPool) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            func.as_mut().handle_spill(pool);
        });
    }

    fn get_global(ir_module: ObjPtr<Module>) -> Vec<GlobalVar> {
        let map = &ir_module.as_ref().global_variable;
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

    fn generate_global_var(&self, f: &mut File) {
        for iter in self.global_var_list.iter() {
            match iter {
                GlobalVar::IGlobalVar(ig) => {
                    let name = ig.get_name();
                    let value = ig.get_init().to_string();
                    writeln!(f , "{name}:\n        .word:   {value}\n");
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

    fn generate_asm(&mut self, f: &mut File, pool: &mut BackendPool) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            func.as_mut().generate(pool.put_context(Context::new()), f);
        });
    }
}