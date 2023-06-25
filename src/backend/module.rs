use std::fs::File;
use std::io::Write;

use crate::backend::func::Func;
use crate::backend::operand::ToString;
use crate::backend::structs::{FGlobalVar, FloatArray, GlobalVar, IGlobalVar, IntArray};
use crate::backend::BackendPool;
use crate::ir::function::Function;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
use crate::log;
use crate::utility::ObjPtr;

use super::instrs::Context;
use super::structs::GenerateAsm;

pub struct AsmModule<'a> {
    pub global_var_list: Vec<(ObjPtr<Inst>, GlobalVar)>,

    func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    pub upper_module: &'a Module,
}

impl<'a> AsmModule<'a> {
    pub fn new(ir_module: &'a Module) -> Self {
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
        for (name, iter) in &self.upper_module.function {
            let ir_func = iter.as_ref();
            let mut func = Func::new(name, pool.put_context(Context::new()));
            func.construct(&self, ir_func, func_seq, pool);
            let func_ptr = pool.put_func(func);
            self.func_map.push((iter.clone(), func_ptr));
            if !iter.is_empty_bb() {
                func_seq += 1;
            }
        }
    }

    // 移除无用指令(TODO,可以把窥孔优化放在这里)
    fn remove_unuse_inst(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            func.as_mut().remove_unuse_inst();
        });
    }

    pub fn generator(&mut self, f: &mut File, f2: &mut File, pool: &mut BackendPool) {
        self.build_lir(pool);
        // TOCHECK 删除函数中无用指令
        self.remove_unuse_inst();

        self.generate_row_asm(f2, pool); //注释
        self.allocate_reg();
        self.handle_spill(pool, f);
        // 检查地址溢出，插入间接寻址
        self.handle_overflow(pool);
        self.generate_global_var(f);
        self.print_model();
        self.generate_asm(f, pool);
    }

    fn handle_overflow(&mut self, pool: &mut BackendPool) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().handle_overflow(pool);
            }
        });
    }

    fn allocate_reg(&mut self) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            // log!("allocate reg fun: {}", func.as_ref().label);
            if !func.is_extern {
                func.as_mut().allocate_reg();
            }
        });
    }

    fn handle_spill(&mut self, pool: &mut BackendPool, f: &mut File) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().handle_spill(pool, f);
            }
        });
    }

    fn get_global(ir_module: &Module) -> Vec<(ObjPtr<Inst>, GlobalVar)> {
        let map = &ir_module.global_variable;
        let mut list = Vec::with_capacity(map.len());
        for (name, iter) in map {
            //TODO: update ir translation，to use ObjPtr match
            // log!("{:?}", iter.as_ref().get_kind());
            match iter.as_ref().get_kind() {
                InstKind::GlobalConstInt(value) | InstKind::GlobalInt(value) => list.push((
                    *iter,
                    GlobalVar::IGlobalVar(IGlobalVar::init(name.to_string(), value, true)),
                )),
                InstKind::GlobalConstFloat(value) | InstKind::GlobalFloat(value) => list.push((
                    *iter,
                    GlobalVar::FGlobalVar(FGlobalVar::init(name.to_string(), value, true)),
                )),
                InstKind::Alloca(size) => {
                    match iter.get_ir_type() {
                        IrType::IntPtr => {
                            let alloca = IntArray::new(
                                name.clone(),
                                size,
                                true,
                                iter.as_ref().get_int_init().clone(),
                            );
                            list.push((*iter, GlobalVar::GlobalConstIntArray(alloca)));
                        }
                        IrType::FloatPtr => {
                            let alloca = FloatArray::new(
                                name.clone(),
                                size,
                                true,
                                iter.as_ref().get_float_init().clone(),
                            );
                            list.push((*iter, GlobalVar::GlobalConstFloatArray(alloca)));
                        }
                        _ => unreachable!(),
                    };
                }
                _ => panic!("fail to analyse GlobalConst"),
            };
        }
        list
    }

    fn generate_global_var(&self, f: &mut File) {
        if self.global_var_list.len() > 0 {
            writeln!(f, "    .data");
        }
        for (inst, iter) in self.global_var_list.iter() {
            match iter {
                GlobalVar::IGlobalVar(ig) => {
                    let name = ig.get_name();
                    let value = ig.get_init().to_string();
                    //FIXME:数组8字节对齐，一般变量4字节对齐，数组size大小为4*array_size
                    writeln!(f, "   .globl {name}\n    .align  2\n     .type   {name}, @object\n   .size   {name}, 4");
                    writeln!(f, "{name}:\n    .word   {value}\n");
                }
                GlobalVar::FGlobalVar(fg) => {
                    let name = fg.get_name();
                    let value = fg.get_init().to_string();
                    writeln!(f, "{name}:\n    .word   {value}\n");
                }
                GlobalVar::GlobalConstIntArray(array) => {
                    writeln!(f, "   .globl {name}\n    .align  3\n     .type   {name}, @object\n   .size   {name}, {num}", name = array.name, num = array.size * 4);
                    writeln!(f, "{name}:", name = array.name);
                    for value in array.value.iter() {
                        writeln!(f, "    .word   {value}");
                    }
                }
                GlobalVar::GlobalConstFloatArray(array) => {
                    writeln!(f, "   .globl {name}\n    .align  3\n     .type   {name}, @object\n   .size   {name}, {num}", name = array.name, num = array.size * 4);
                    writeln!(f, "{name}:", name = array.name);
                    for value in array.value.iter() {
                        writeln!(f, "    .word   {value}");
                    }
                }
            }
        }
    }

    fn generate_asm(&mut self, f: &mut File, pool: &mut BackendPool) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().generate(pool.put_context(Context::new()), f);
            }
        });
    }

    fn generate_row_asm(&mut self, f: &mut File, pool: &mut BackendPool) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut()
                    .generate_row(pool.put_context(Context::new()), f);
            }
        });
    }

    fn print_model(&self) {
        self.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.print_func();
            }
        });
    }
}
