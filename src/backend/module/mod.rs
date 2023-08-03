use std::collections::{HashMap, HashSet, LinkedList};
use std::fs::File;
use std::io::Write;

use crate::backend::block::NUM_SIZE;
use crate::backend::func::Func;
use crate::backend::operand::ToString;
use crate::backend::opt::BackendPass;
use crate::backend::structs::{FGlobalVar, FloatArray, GlobalVar, IGlobalVar, IntArray};
use crate::backend::BackendPool;
use crate::container::bitmap::Bitmap;
use crate::ir::function::Function;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
use crate::log_file;
// use crate::log;
use crate::utility::ObjPtr;

use super::instrs::{Context, InstrsType, LIRInst, BB};
use super::operand::Reg;
use super::regalloc::easy_gc_alloc;
use super::regalloc::structs::FuncAllocStat;
use super::structs::GenerateAsm;
pub mod build;
pub mod final_realloc;
pub mod handle_call;
pub mod reduce_caller_save;
pub mod rm_inst;
pub mod schedule;
mod test;
pub mod utils;
pub mod v3;
pub mod v4;
pub struct AsmModule {
    pub global_var_list: Vec<(ObjPtr<Inst>, GlobalVar)>,
    pub func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    call_map: HashMap<String, HashSet<String>>,
    func_groups: HashMap<String, HashSet<String>>,
    ///记录该被调用函数需要保存的所有寄存器
    callee_regs_to_saveds: HashMap<String, HashSet<Reg>>,
    ///记录调用该函数的函数应该保存的寄存器
    caller_regs_to_saveds: HashMap<String, HashSet<Reg>>,
    call_info: HashMap<String, HashMap<Bitmap, String>>, //每个base func name 对应调用的 不同callee need save函数
    pub name_func: HashMap<String, ObjPtr<Func>>,        //记录实际函数名和实际函数
    pub upper_module: Module,
}

///lir构建,module构建
impl AsmModule {
    pub fn new(ir_module: Module) -> Self {
        let global_var_list = Self::get_global(ir_module.clone());
        Self {
            global_var_list,
            // global_fvar_list,
            func_map: Vec::new(),
            func_groups: HashMap::new(),
            upper_module: ir_module,
            call_info: HashMap::new(),
            name_func: HashMap::new(),
            call_map: HashMap::new(),
            callee_regs_to_saveds: HashMap::new(),
            caller_regs_to_saveds: HashMap::new(),
        }
    }

    pub fn build_lir(&mut self, pool: &mut BackendPool) {
        let mut func_seq = 0;
        for (name, iter) in &self.upper_module.get_all_func() {
            let ir_func = iter.as_ref();
            let mut func = Func::new(name, pool.put_context(Context::new()));
            func.construct(&self, ir_func, func_seq, pool);
            let func_ptr = pool.put_func(func);
            self.func_map.push((iter.clone(), func_ptr));
            if !iter.is_empty_bb() {
                func_seq += 1;
            }
            if func_ptr.is_extern {
                continue;
            }
            self.name_func.insert(func_ptr.label.clone(), func_ptr);
        }
    }
}

///base:
///1. 生成汇编
///2. 生成函数栈信息
///3. overflow保存和恢复
impl AsmModule {
    // 寄存器分配和寄存器赋值前,移除无用指令(比如移除mv)
    fn remove_unuse_inst_pre_alloc(&mut self) {
        self.name_func.iter().for_each(|(_, func)| {
            func.as_mut().remove_unuse_inst();
        });
    }

    // 寄存器分配和使用后 移除无用指令(比如移除多余的缓存指令)
    fn remove_unuse_inst_suf_alloc(&mut self) {
        self.name_func
            .iter()
            .filter(|(_, func)| !func.is_extern)
            .for_each(|(_, func)| {
                func.as_mut().remove_unuse_inst();
            });
    }

    /// 设置栈大小 ，设置开合栈以及进行callee saved的保存和恢复需要的前沿和后沿函数
    /// 该函数需要在handle call后调用
    pub fn handle_callee(&mut self, f: &mut File) {
        for (_, func) in self.name_func.iter() {
            debug_assert!(!func.is_extern);
            func.as_mut().build_callee_map();
            func.as_mut().save_callee(f)
        }
    }

    pub fn handle_overflow(&mut self, pool: &mut BackendPool) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().handle_overflow(pool);
            }
        });
    }

    // 再次进行指令重排
    pub fn re_list_scheduling(&mut self) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            func.list_scheduling_tech();
        });
    }

    /// 第一次运行v2p时不映射临时寄存器，第二次运行前清空tmp_vars set
    fn map_v_to_p(&mut self) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            debug_assert!(!func.is_extern);
            func.blocks.iter().for_each(|block| {
                block.insts.iter().for_each(|inst| {
                    inst.as_mut()
                        .v_to_phy(func.context.get_reg_map().clone(), func.tmp_vars.clone());
                });
            });
        });
    }

    fn allocate_reg(&mut self) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            // log!("allocate reg fun: {}", func.as_ref().label);
            debug_assert!(!func.is_extern);
            func.as_mut().allocate_reg();
        });
    }

    fn get_global(ir_module: Module) -> Vec<(ObjPtr<Inst>, GlobalVar)> {
        let map = &ir_module.get_all_var();
        let mut list = Vec::with_capacity(map.len());
        for (name, iter) in map {
            //TODO: update ir translation，to use ObjPtr match
            // log!("{:?}", iter.as_ref().get_kind());
            match iter.as_ref().get_kind() {
                InstKind::GlobalConstInt(value) | InstKind::GlobalInt(value) => list.push((
                    *iter,
                    GlobalVar::IGlobalVar(IGlobalVar::init(name.to_string(), value)),
                )),
                InstKind::GlobalConstFloat(value) | InstKind::GlobalFloat(value) => list.push((
                    *iter,
                    GlobalVar::FGlobalVar(FGlobalVar::init(name.to_string(), value)),
                )),
                InstKind::Alloca(size) => {
                    match iter.get_ir_type() {
                        IrType::IntPtr => {
                            let alloca = IntArray::new(
                                name.to_string(),
                                size,
                                true,
                                iter.as_ref()
                                    .get_int_init()
                                    .1
                                    .iter()
                                    .map(|(_, value)| *value)
                                    .collect::<Vec<_>>()
                                    .clone(),
                            );
                            list.push((*iter, GlobalVar::GlobalConstIntArray(alloca)));
                        }
                        IrType::FloatPtr => {
                            let alloca = FloatArray::new(
                                name.to_string(),
                                size,
                                true,
                                iter.as_ref()
                                    .get_float_init()
                                    .1
                                    .iter()
                                    .map(|(_, value)| *value)
                                    .collect::<Vec<_>>()
                                    .clone(),
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
            writeln!(f, "    .data").unwrap();
        }
        for (_, iter) in self.global_var_list.iter() {
            match iter {
                GlobalVar::IGlobalVar(ig) => {
                    let name = ig.get_name();
                    let value = ig.get_init().to_string();
                    //FIXME:数组8字节对齐，一般变量4字节对齐，数组size大小为4*array_size
                    writeln!(f, "   .globl {name}\n    .align  2\n     .type   {name}, @object\n   .size   {name}, 4").unwrap();
                    writeln!(f, "{name}:\n    .word   {value}\n").unwrap();
                }
                GlobalVar::FGlobalVar(fg) => {
                    let name = fg.get_name();
                    let value = fg.get_init().to_string();
                    writeln!(f, "{name}:\n    .word   {value}\n").unwrap();
                }
                GlobalVar::GlobalConstIntArray(array) => {
                    writeln!(f, "   .globl {name}\n    .align  3\n     .type   {name}, @object\n   .size   {name}, {num}", name = array.name, num = array.size * 4).unwrap();
                    writeln!(f, "{name}:", name = array.name).unwrap();
                    for value in array.value.iter() {
                        writeln!(f, "    .word   {value}").unwrap();
                    }
                    let zeros = array.size - array.value.len() as i32;
                    if zeros > 0 {
                        writeln!(f, "	.zero	{n}", n = zeros * NUM_SIZE).unwrap();
                    }
                }
                GlobalVar::GlobalConstFloatArray(array) => {
                    writeln!(f, "   .globl {name}\n    .align  3\n     .type   {name}, @object\n   .size   {name}, {num}", name = array.name, num = array.size * 4).unwrap();
                    writeln!(f, "{name}:", name = array.name).unwrap();
                    for value in array.value.iter() {
                        writeln!(f, "    .word   {value}").unwrap();
                    }
                    let zeros = array.size - array.value.len() as i32;
                    if zeros > 0 {
                        writeln!(f, "	.zero	{n}", n = zeros * NUM_SIZE).unwrap();
                    }
                }
            }
        }
    }

    pub fn generate_asm(&mut self, f: &mut File, pool: &mut BackendPool) {
        // 生成全局变量与数组
        self.generate_global_var(f);
        let mut asm_order: Vec<ObjPtr<Func>> = Vec::new();
        // println!("{}", self.call_info.len());
        if self.call_info.len() != 0 {
            for (_, func) in self.func_map.iter() {
                if func.is_extern {
                    continue;
                }
                if func.label == "main" {
                    asm_order.push(*self.name_func.get("main").unwrap());
                    continue;
                }
                let name = func.label.clone();
                for name in self.call_info.get(name.as_str()).unwrap() {
                    let name = name.1;
                    // println!("name{}", name.clone());
                    let func = self.name_func.get(name).unwrap();
                    asm_order.push(*func);
                }
            }
        } else {
            for (_, func) in self.func_map.iter() {
                // println!("{}", func.label);
                if func.is_extern {
                    continue;
                }
                asm_order.push(*func);
            }
        }

        asm_order.iter_mut().for_each(|func| {
            debug_assert!(!func.is_extern);
            func.as_mut().generate(pool.put_context(Context::new()), f);
        })
    }

    pub fn generate_row_asm(&mut self, f: &mut File, pool: &mut BackendPool) {
        debug_assert!(|| -> bool {
            self.name_func.iter_mut().for_each(|(_, func)| {
                debug_assert!(!func.is_extern);
                func.as_mut()
                    .generate_row(pool.put_context(Context::new()), f);
            });
            // self.func_map.iter_mut().for_each(|(_, func)| {
            //     if !func.is_extern {
            //         func.as_mut()
            //             .generate_row(pool.put_context(Context::new()), f);
            //     }
            // });
            true
        }());
    }
}
