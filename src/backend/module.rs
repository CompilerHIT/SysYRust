use std::collections::{HashMap, HashSet, LinkedList};
use std::fs::File;
use std::io::Write;

use crate::backend::block::NUM_SIZE;
use crate::backend::func::Func;
use crate::backend::operand::ToString;
use crate::backend::structs::{FGlobalVar, FloatArray, GlobalVar, IGlobalVar, IntArray};
use crate::backend::BackendPool;
use crate::container::bitmap::Bitmap;
use crate::ir::function::Function;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
use crate::log;
use crate::utility::ObjPtr;

use super::instrs::{Context, InstrsType};
use super::operand::Reg;
use super::opt::BackendPass;
use super::regalloc::structs::FuncAllocStat;
use super::structs::GenerateAsm;

pub struct AsmModule {
    pub global_var_list: Vec<(ObjPtr<Inst>, GlobalVar)>,

    pub func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    callees_saveds: HashMap<String, HashSet<Reg>>,
    callers_saveds: HashMap<String, HashSet<Reg>>,
    call_info: HashMap<String, HashMap<Bitmap, String>>, //每个base func name 对应调用的 不同callee need save函数
    name_func: HashMap<String, ObjPtr<Func>>,            //记录实际函数名和实际函数
    pub upper_module: Module,
}

impl AsmModule {
    pub fn new(ir_module: Module) -> Self {
        let global_var_list = Self::get_global(ir_module.clone());
        Self {
            global_var_list,
            // global_fvar_list,
            func_map: Vec::new(),
            upper_module: ir_module,
            call_info: HashMap::new(),
            name_func: HashMap::new(),
            callees_saveds: HashMap::new(),
            callers_saveds: HashMap::new(),
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
        }
    }

    // 寄存器分配和寄存器赋值前,移除无用指令(比如移除mv)
    fn remove_unuse_inst_pre_alloc(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            func.as_mut().remove_unuse_inst();
        });
    }

    // 寄存器分配和使用后 移除无用指令(比如移除多余的缓存指令)
    fn remove_unuse_inst_suf_alloc(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            func.as_mut().remove_unuse_inst();
        });
    }

    pub fn build(&mut self, f: &mut File, f2: &mut File, pool: &mut BackendPool) {
        self.build_lir(pool);
        // TOCHECK 寄存器分配和handlespill前无用指令删除,比如删除mv指令方便寄存器分配
        // self.generate_row_asm(f2, pool); //注释
        self.remove_unuse_inst_pre_alloc();
        self.generate_row_asm(f2, pool); //注释

        self.allocate_reg();
        // self.generate_row_asm(f2, pool); //注释

        self.handle_spill(pool, f);
        // self.generate_row_asm(f2, pool); //注释
        self.map_v_to_p();
        // self.generate_row_asm(f2, pool); //注释
        self.remove_unuse_inst_suf_alloc();
        // self.generate_row_asm(f2, pool); //注释
    }

    pub fn handle_overflow(&mut self, pool: &mut BackendPool) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().handle_overflow(pool);
            }
        });
    }

    fn map_v_to_p(&mut self) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    block.insts.iter().for_each(|inst| {
                        inst.as_mut().v_to_phy(func.context.get_reg_map().clone());
                    });
                });
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

    fn get_global(ir_module: Module) -> Vec<(ObjPtr<Inst>, GlobalVar)> {
        let map = &ir_module.get_all_var();
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
                    let zeros = array.size - array.value.len() as i32;
                    if zeros > 0 {
                        writeln!(f, "	.zero	{n}", n = zeros * NUM_SIZE);
                    }
                }
                GlobalVar::GlobalConstFloatArray(array) => {
                    writeln!(f, "   .globl {name}\n    .align  3\n     .type   {name}, @object\n   .size   {name}, {num}", name = array.name, num = array.size * 4);
                    writeln!(f, "{name}:", name = array.name);
                    for value in array.value.iter() {
                        writeln!(f, "    .word   {value}");
                    }
                    let zeros = array.size - array.value.len() as i32;
                    if zeros > 0 {
                        writeln!(f, "	.zero	{n}", n = zeros * NUM_SIZE);
                    }
                }
            }
        }
    }

    pub fn generate_asm(&mut self, f: &mut File, pool: &mut BackendPool) {
        // 生成全局变量与数组
        self.generate_global_var(f);

        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().generate(pool.put_context(Context::new()), f);
            }
        });
    }

    pub fn generate_row_asm(&mut self, f: &mut File, pool: &mut BackendPool) {
        debug_assert!(|| -> bool {
            self.func_map.iter_mut().for_each(|(_, func)| {
                if !func.is_extern {
                    func.as_mut()
                        .generate_row(pool.put_context(Context::new()), f);
                }
            });
            true
        }());
    }

    fn print_model(&self) {
        self.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                // func.print_func();
            }
        });
    }
}

/// build v2:
/// 1.紧缩spill过程使用的栈空间: 先分配后使用
impl AsmModule {
    //先进行寄存器分配再handle_spill,
    pub fn build_v2(&mut self, f: &mut File, f2: &mut File, pool: &mut BackendPool) {
        self.build_lir(pool);
        // TOCHECK 寄存器分配和handlespill前无用指令删除,比如删除mv指令方便寄存器分配
        self.remove_unuse_inst_pre_alloc();
        // self.generate_row_asm(f2, pool); //注释
        self.allocate_reg();
        self.handle_spill_v2(pool, f);
        // self.handle_spill(pool, f);
        self.map_v_to_p();
        self.remove_unuse_inst_suf_alloc();
    }

    fn handle_spill_v2(&mut self, pool: &mut BackendPool, f: &mut File) {
        self.func_map.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().handle_spill_v2(pool, f);
            }
        });
    }
}

/// build v3:
/// 1. 实现 函数分裂, 优化callee的保存恢复
/// 2. 指令级 上下文 caller 选择
/// 3. 对spill use和caller use的栈空间 紧缩
/// 4. 删除无用函数模板(可选)
impl AsmModule {
    ///TODO!
    pub fn build_v3(&mut self, f: &mut File, f2: &mut File, pool: &mut BackendPool) {
        self.build_lir(pool);
        self.remove_unuse_inst_pre_alloc();
        // self.generate_row_asm(f2, pool);     //generate row  asm可能会造成bug
        self.remove_unuse_inst_pre_alloc();
        self.allocate_reg();
        self.map_v_to_p();
        self.handle_spill_v3(pool);
        self.remove_unuse_inst_suf_alloc();
        self.anaylyse_for_handle_call_v3(pool);
        self.handle_call_v3(pool);
        self.remove_useless_func();
        self.rearrange_stack_slot();
        self.update_array_offset(pool);
        self.build_stack_info(f, pool);
        self.print_func();
        //删除无用的函数
    }
    pub fn handle_spill_v3(&mut self, pool: &mut BackendPool) {
        self.func_map
            .iter()
            .for_each(|(_, func)| func.as_mut().handle_spill_v3(pool));
    }

    ///对于caller save 和 handle spill  使用到的栈空间 进行紧缩
    pub fn rearrange_stack_slot(&mut self) {
        self.func_map
            .iter()
            .for_each(|(_, func)| func.as_mut().rearrange_stack_slot());
    }

    ///处理 函数调用前后的保存和恢复
    /// 1. 插入保存和恢复caller save的指令
    pub fn handle_call_v3(&mut self, pool: &mut BackendPool) {
        // 分析并刷新每个函数的call指令前后需要保存的caller save信息,以及call内部的函数需要保存的callee save信息
        // 对于 handle call

        for (_, func) in self.func_map.iter() {
            func.as_mut().handle_call_v3(pool, &self.callers_saveds);
        }
    }
    ///准备 callee save和caller save需要的信息
    /// 1. 准备每个函数需要的callee save,以及进行函数分裂
    /// 2. 针对性地让函数自我转变 , 调整每个函数中使用到的寄存器分布等等
    fn anaylyse_for_handle_call_v3(&mut self, pool: &mut BackendPool) {
        //TODO
        let mut caller_used: HashMap<ObjPtr<Func>, HashSet<Reg>> = HashMap::new();
        for (_, func) in self.func_map.iter() {
            self.name_func.insert(func.label.clone(), *func);
            if !func.is_extern {
                self.callees_saveds
                    .insert(func.label.clone(), func.draw_used_callees());
                caller_used.insert(*func, func.draw_used_callers());
            } else {
                ///TODO,确认对于调用的库函数来说,使用的caller saved是否能够被忽略
                ///获取所有的caller saved寄存器
                caller_used.insert(*func, Reg::get_all_callers_saved());
            }
        }
        //加入外部函数
        let build_external_func =
            |module: &mut AsmModule, name: &str, pool: &mut BackendPool| -> ObjPtr<Func> {
                let external_context = pool.put_context(Context::new());
                let external_func = pool.put_func(Func::new(name, external_context));
                external_func.as_mut().is_extern = true;
                module.name_func.insert(name.to_string(), external_func);
                external_func
            };
        //补充外部函数
        caller_used.insert(
            build_external_func(self, "memset@plt", pool),
            Reg::get_all_callers_saved(),
        );
        caller_used.insert(
            build_external_func(self, "memcpy@plt", pool),
            Reg::get_all_callers_saved(),
        );

        //创建外部函数

        //对于额外使用的memset函数,单独记录s
        //构造每个函数的caller save regs  (caller save表要递归调用分析)
        //首先获取所有函数的所有call指令 (caller func,callee func)
        let mut call_insts: Vec<(ObjPtr<Func>, ObjPtr<Func>)> = Vec::new();
        for (func, _) in caller_used.iter() {
            for bb in func.blocks.iter() {
                for inst in bb.insts.iter() {
                    if inst.get_type() == InstrsType::Call {
                        let label = inst.get_label().get_func_name();
                        debug_assert!(self.name_func.contains_key(label.as_str()), "{label}");
                        let callee_func = *self.name_func.get(&label).unwrap();
                        call_insts.push((*func, callee_func));
                    }
                }
            }
        }

        loop {
            let mut ifFinish = true;
            //直到无法发生更新了才退出
            ///更新caller save
            for (caller_func, callee_func) in call_insts.iter() {
                //
                let old_caller_func_used_callers = caller_used.get(caller_func).unwrap();
                let old_num = old_caller_func_used_callers.len();
                let new_regs: HashSet<Reg> = caller_used
                    .get(callee_func)
                    .unwrap()
                    .iter()
                    .cloned()
                    .collect();
                caller_used.get_mut(caller_func).unwrap().extend(new_regs);
                if caller_used.get(caller_func).unwrap().len() > old_num {
                    ifFinish = false;
                }
            }
            if ifFinish {
                break;
            }
        }
        //分析完caller saved的使用,把caller used表中的信息更新到func中
        for (func, caller_saved_regs) in caller_used {
            self.callers_saveds
                .insert(func.label.clone(), caller_saved_regs);
        }

        //TODO,分析callee saved的使用并进行函数分裂
        //根据上下文分析,获取每个callee saved的函数调用的上下文需要保存的callee save寄存器
        for (func, callee_useds) in self.callees_saveds.iter() {
            //从name func中取出
        }
    }

    /// 计算栈空间,进行ra,sp,callee 的保存和恢复
    pub fn build_stack_info(&mut self, f: &mut File, pool: &mut BackendPool) {
        //把 callee used表中的信息更新到func中
        for (_, func) in self.func_map.iter() {
            if !func.is_extern {
                func.as_mut().callee_saved.extend(func.draw_used_callees());
                func.as_mut().save_callee(pool, f);
            }
        }
    }

    ///删除进行函数分裂后的剩余无用函数
    pub fn remove_useless_func(&mut self) {
        let mut new_func_map = Vec::new();
        for (f, func) in self.func_map.iter() {
            if self.name_func.contains_key(&func.label) {
                new_func_map.push((*f, *func));
            }
        }
        self.func_map = new_func_map;
    }

    pub fn update_array_offset(&mut self, pool: &mut BackendPool) {
        for (_, func) in self.func_map.iter() {
            if !func.is_extern {
                func.as_mut().update_array_offset(pool);
            }
        }
    }

    pub fn print_func(&self) {
        for (_, func) in self.func_map.iter() {
            func.print_func();
        }
    }
}
