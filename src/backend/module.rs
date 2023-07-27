use std::collections::{HashMap, HashSet, LinkedList};
use std::fs::File;
use std::io::Write;

use crate::backend::block::NUM_SIZE;
use crate::backend::func::Func;
use crate::backend::operand::ToString;
use crate::backend::structs::{FGlobalVar, FloatArray, GlobalVar, IGlobalVar, IntArray};
use crate::backend::BackendPool;
use crate::backend::opt::BackendPass;
use crate::container::bitmap::Bitmap;
use crate::ir::function::Function;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::ir_type::IrType;
use crate::ir::module::Module;
// use crate::log;
use crate::utility::ObjPtr;

use super::instrs::{Context, InstrsType, LIRInst, BB};
use super::operand::Reg;
use super::regalloc::regalloc::Regalloc;
use super::regalloc::structs::RegUsedStat;
use super::regalloc::{self, easy_gc_alloc};
use super::structs::GenerateAsm;

pub struct AsmModule {
    pub global_var_list: Vec<(ObjPtr<Inst>, GlobalVar)>,
    pub func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    callee_regs_to_saveds: HashMap<String, HashSet<Reg>>,
    caller_regs_to_saveds: HashMap<String, HashSet<Reg>>,
    call_info: HashMap<String, HashMap<Bitmap, String>>, //每个base func name 对应调用的 不同callee need save函数
    pub name_func: HashMap<String, ObjPtr<Func>>,        //记录实际函数名和实际函数
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

    pub fn build(&mut self, f: &mut File, _f2: &mut File, pool: &mut BackendPool) {
        self.build_lir(pool);
        // TOCHECK 寄存器分配和handlespill前无用指令删除,比如删除mv指令方便寄存器分配
        // self.generate_row_asm(f2, pool); //注释
        self.remove_unuse_inst_pre_alloc();
        self.cal_tmp_var();
        // self.generate_row_asm(f2, pool); //注释

        self.allocate_reg();
        // self.generate_row_asm(f2, pool); //注释

        self.handle_spill(pool);

        self.handle_call(pool);
        self.handle_callee(f);

        // self.generate_row_asm(f2, pool); //注释
        self.map_v_to_p();
        // 代码调度
        self.list_scheduling_tech();

        // self.generate_row_asm(f2, pool);

        // 为临时寄存器分配寄存器
        self.clear_tmp_var();
        self.allocate_reg();
        self.map_v_to_p();
        // self.generate_row_asm(f2, pool);

        // self.generate_row_asm(f2, pool); //注释
        self.remove_unuse_inst_suf_alloc();
        // self.generate_row_asm(f2, pool); //注释
    }
    ///处理call前后caller saved 寄存器的保存和恢复
    /// 该函数应该在handle spill后调用
    pub fn handle_call(&mut self, pool: &mut BackendPool) {
        for (_, func) in self.name_func.iter() {
            debug_assert!(!func.is_extern);
            func.as_mut().handle_call(pool);
            func.as_mut().update_array_offset(pool);
        }
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

    /// 计算临时变量的个数
    pub fn cal_tmp_var(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().cal_tmp_var();
            }
        });
    }

    /// 清除临时变量
    pub fn clear_tmp_var(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().tmp_vars.clear();
            }
        });
    }

    /// 代码调度
    pub fn list_scheduling_tech(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().list_scheduling_tech();
            }
        });
    }

    pub fn handle_overflow(&mut self, pool: &mut BackendPool) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().handle_overflow(pool);
            }
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

    fn handle_spill(&mut self, pool: &mut BackendPool) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            debug_assert!(!func.is_extern);
            func.as_mut().handle_spill(pool);
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

/// build v2:
/// 1.紧缩spill过程使用的栈空间: 先分配后使用
impl AsmModule {
    //先进行寄存器分配再handle_spill,
    pub fn build_v2(&mut self, f: &mut File, _f2: &mut File, pool: &mut BackendPool) {
        self.build_lir(pool);
        self.remove_unuse_inst_pre_alloc();
        // self.generate_row_asm(_f2, pool); //generate row  asm可能会造成bug

        // self.generate_row_asm(_f2, pool);
        self.allocate_reg();
        // self.generate_row_asm(_f2, pool);
        self.map_v_to_p();

        // self.print_func();
        // self.generate_row_asm(_f2, pool);
        // ///重分配
        // self.name_func.iter().for_each(|(_, func)| {
        //     func.as_mut()
        //         .p2v_pre_handle_call(Reg::get_all_recolorable_regs())
        // });
        // self.generate_row_asm(_f2, pool);
        // self.allocate_reg();
        // self.map_v_to_p();

        self.handle_spill_v3(pool);
        self.remove_unuse_inst_suf_alloc();
        self.anaylyse_for_handle_call_v3();

        // let mut is_opt = false;
        // if is_opt {
        //     self.split_func(pool);
        // }

        self.handle_call_v3(pool);
        // self.remove_useless_func();
        self.rearrange_stack_slot();
        self.update_array_offset(pool);
        self.build_stack_info(f);
    }

    fn handle_spill_v2(&mut self, pool: &mut BackendPool) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            debug_assert!(!func.is_extern);
            func.as_mut().handle_spill_v2(pool);
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
    pub fn build_v3(&mut self, f: &mut File, _f2: &mut File, pool: &mut BackendPool, is_opt: bool) {
        self.build_lir(pool);
        self.remove_unuse_inst_pre_alloc();

        // self.generate_row_asm(_f2, pool); //generate row  asm可能会造成bug

        if is_opt {
            // 设置一些寄存器为临时变量
            self.cal_tmp_var();

            self.allocate_reg();
            self.map_v_to_p();
            // 代码调度，列表调度法
            self.list_scheduling_tech();

            // 为临时寄存器分配寄存器
            self.clear_tmp_var();
            self.allocate_reg();
            self.map_v_to_p();
        } else {
            // self.generate_row_asm(_f2, pool);
            self.allocate_reg();
            // self.generate_row_asm(_f2, pool);
            // self.map_v_to_p();
            // self.generate_row_asm(_f2, pool);
            // // ///重分配
            // self.name_func.iter().for_each(|(_, func)| {
            //     func.as_mut()
            //         .p2v_pre_handle_call(Reg::get_all_recolorable_regs())
            // });
            // // self.generate_row_asm(_f2, pool);
            // self.allocate_reg();
            self.map_v_to_p();
        }

        self.handle_spill_v3(pool);
        self.remove_unuse_inst_suf_alloc();

        self.add_external_func(pool); //加入外部函数以进行分析
        self.anaylyse_for_handle_call_v3();

        if is_opt {
            self.split_func(pool);
        }

        self.remove_useless_func();
        self.handle_call_v3(pool);
        self.rearrange_stack_slot();
        self.update_array_offset(pool);
        self.build_stack_info(f);
        // self.print_func();
        //删除无用的函数
    }

    ///处理spillings的虚拟寄存器的临时物理寄存器借用
    pub fn handle_spill_v3(&mut self, pool: &mut BackendPool) {
        self.name_func
            .iter()
            .for_each(|(_, func)| func.as_mut().handle_spill_v3(pool));
    }

    ///对于caller save 和 handle spill  使用到的栈空间 进行紧缩
    pub fn rearrange_stack_slot(&mut self) {
        self.name_func
            .iter()
            .for_each(|(_, func)| func.as_mut().rearrange_stack_slot());
    }

    ///处理 函数调用前后的保存和恢复
    /// 1. 插入保存和恢复caller save的指令
    pub fn handle_call_v3(&mut self, pool: &mut BackendPool) {
        // 分析并刷新每个函数的call指令前后需要保存的caller save信息,以及call内部的函数需要保存的callee save信息
        // 对于 handle call
        for (_, func) in self.name_func.iter() {
            debug_assert!(!func.is_extern);
            func.as_mut()
                .handle_call_v3(pool, &self.caller_regs_to_saveds);
        }
    }

    ///加入外部函数,
    pub fn add_external_func(&mut self, pool: &mut BackendPool) {
        // debug_assert!(self.name_func.contains_key("putint"));
        //加入外部函数
        let build_external_func =
            |module: &mut AsmModule, name: &str, pool: &mut BackendPool| -> ObjPtr<Func> {
                let external_context = pool.put_context(Context::new());
                let external_func = pool.put_func(Func::new(name, external_context));
                external_func.as_mut().is_extern = true;
                module.name_func.insert(name.to_string(), external_func);
                external_func
            };
        //补充外部函数 memset 和memcpy
        let memset = build_external_func(self, "memset@plt", pool);
        let memcpy = build_external_func(self, "memcpy@plt", pool);
        let putint = build_external_func(self, "putint", pool);
        let getint = build_external_func(self, "getint", pool);
        let getarray = build_external_func(self, "getarray", pool);
        let putarray = build_external_func(self, "putarray", pool);
        let getch = build_external_func(self, "getch", pool);
        let putch = build_external_func(self, "putch", pool);
        let getfloat = build_external_func(self, "getfloat", pool);
        let putfloat = build_external_func(self, "putfloat", pool);
        let getfarray = build_external_func(self, "getfarray", pool);
        let putfarrray = build_external_func(self, "putfarray", pool);
        let putf = build_external_func(self, "putf", pool);
        let starttime = build_external_func(self, "_sysy_starttime", pool);
        let stoptime = build_external_func(self, "_sysy_stoptime", pool);
    }

    ///准备 callee save和caller save需要的信息
    /// 1. 准备每个函数需要的callee save,以及进行函数分裂
    /// 2. 针对性地让函数自我转变 , 调整每个函数中使用到的寄存器分布等等
    /// 3. 该函数应该在vtop和handle spill后调用
    /// 4. 过程中会往name func中加入需要的外部函数的信息
    fn anaylyse_for_handle_call_v3(&mut self) {
        //TODO
        self.callee_regs_to_saveds.clear();
        self.caller_regs_to_saveds.clear();
        let mut caller_used: HashMap<ObjPtr<Func>, HashSet<Reg>> = HashMap::new();

        for (_, func) in self.name_func.iter() {
            if !func.is_extern {
                self.callee_regs_to_saveds
                    .insert(func.label.clone(), func.draw_used_callees());
                caller_used.insert(*func, func.draw_used_callers());
            } else {
                //对于外部函数(默认内部使用了所有的callers saved函数)
                //同时也默认认为其使用了所有callee saved的函数
                self.callee_regs_to_saveds
                    .insert(func.label.clone(), Reg::get_all_callees_saved());
                caller_used.insert(*func, Reg::get_all_callers_saved());
            }
        }

        //构造每个函数的caller save regs  (caller save表要递归调用分析)
        //首先获取所有函数的所有call指令 (caller func,callee func)
        let mut call_insts: Vec<(ObjPtr<Func>, ObjPtr<Func>)> = Vec::new();
        for (func, _) in caller_used.iter() {
            for bb in func.blocks.iter() {
                for inst in bb.insts.iter() {
                    if inst.get_type() == InstrsType::Call {
                        let label = inst.get_func_name().unwrap();
                        debug_assert!(self.name_func.contains_key(label.as_str()), "{label}");
                        let callee_func = *self.name_func.get(&label).unwrap();
                        call_insts.push((*func, callee_func));
                    }
                }
            }
        }

        loop {
            let mut if_finish = true;
            //直到无法发生更新了才退出
            //更新caller save
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
                    if_finish = false;
                }
            }
            if if_finish {
                break;
            }
        }

        //分析完caller saved的使用,把caller used表中的信息更新到func中
        for (func, caller_saved_regs) in caller_used {
            self.caller_regs_to_saveds
                .insert(func.label.clone(), caller_saved_regs);
        }
        //之后caller_used数据结构就没有用了 (信息已经存入了 self.callers_saved中)

        //更新基础callees saved uesd 表
        loop {
            let mut if_finish = true;
            //直到无法发生更新了才退出
            //更新caller save
            for (caller_func, callee_func) in call_insts.iter() {
                //
                let old_caller_func_used_callees = self
                    .callee_regs_to_saveds
                    .get(caller_func.label.as_str())
                    .unwrap();
                let old_num = old_caller_func_used_callees.len();
                let new_regs: HashSet<Reg> = self
                    .callee_regs_to_saveds
                    .get(callee_func.label.as_str())
                    .unwrap()
                    .clone();
                self.callee_regs_to_saveds
                    .get_mut(caller_func.as_ref().label.as_str())
                    .unwrap()
                    .extend(new_regs);
                if self
                    .callee_regs_to_saveds
                    .get(caller_func.label.as_str())
                    .unwrap()
                    .len()
                    > old_num
                {
                    if_finish = false;
                }
            }
            if if_finish {
                break;
            }
        }
    }

    ///函数分裂:
    /// 该函数只应该在analyse for handle call v3后被调用
    fn split_func(&mut self, pool: &mut BackendPool) {
        //
        self.callee_regs_to_saveds
            .insert("main".to_string(), HashSet::new());

        let regs_set_to_string = |regs: &HashSet<Reg>| -> String {
            let mut symbol = "".to_string();
            for id in 0..=63 {
                let reg = Reg::from_color(id);
                if !regs.contains(&reg) {
                    continue;
                }
                symbol.push_str(reg.to_string(false).as_str());
            }
            symbol
        };
        let regs_to_bitmap = |regs: &HashSet<Reg>| -> Bitmap {
            let mut map = Bitmap::new();
            for reg in regs {
                map.insert(reg.get_color() as usize);
            }
            map
        };
        let main_func = *self.name_func.get("main").unwrap();
        let mut func_to_process = Vec::new();
        func_to_process.push(main_func);

        let mut new_name_func: HashMap<String, ObjPtr<Func>> = HashMap::new();
        new_name_func.insert("main".to_string(), main_func);

        //call info加入非main函数
        for (name, func) in self.name_func.iter() {
            if name == "main" {
                continue;
            }
            self.call_info.insert(name.clone(), HashMap::new());
        }

        //然后分析callee save的使用情况,进行裂变,同时产生新的name func
        loop {
            let mut if_finish = true;
            let mut new_funcs: Vec<ObjPtr<Func>> = Vec::new();
            //分析调用的上下文
            for func in func_to_process.iter() {
                let call_insts = func.analyse_for_handle_call(&self.callee_regs_to_saveds);
                //通过对func 的上下文分析 (返回某个call指令附近需要保存的callee saved寄存器)
                //如果遇到新函数,加入callee saved
                for (call_inst, callee_regs) in call_insts.iter() {
                    let func_label = call_inst.get_func_name().unwrap();
                    let func_label_callee_maps = self.call_info.get(&func_label).unwrap();
                    let callee_func = self.name_func.get(&func_label).unwrap();
                    if callee_func.is_extern {
                        continue;
                    }
                    let map = regs_to_bitmap(callee_regs);
                    //如果该类型 callee 函数已经存在,直接变名
                    if func_label_callee_maps.contains_key(&map) {
                        let real_func_name = func_label_callee_maps.get(&map).unwrap().clone();
                        call_inst.as_mut().replace_label(real_func_name);
                        continue;
                    }
                    //否则产生一个新的函数

                    let new_callee_func = callee_func.real_deep_clone(pool);
                    let suffix = regs_set_to_string(callee_regs);
                    let mut new_name = suffix.clone();
                    new_name.push_str(&format!("_{}", func_label).to_string());
                    new_callee_func.as_mut().set_name(&new_name);
                    let suffix = format!("_{func_label}_{suffix}");
                    new_callee_func.as_mut().suffix_bb(&suffix);
                    if func_label_callee_maps.len() >= 1 {
                        new_callee_func.as_mut().is_header = false;
                    }

                    new_funcs.push(new_callee_func);
                    call_inst.as_mut().replace_label(new_name.clone());

                    self.call_info
                        .get_mut(&func_label)
                        .unwrap()
                        .insert(map, new_name.clone());

                    // 更新新函数的callees map
                    self.callee_regs_to_saveds
                        .insert(new_name.clone(), callee_regs.iter().cloned().collect());
                    // 继承旧函数的callers map
                    let old_callers_saved = self
                        .caller_regs_to_saveds
                        .get_mut(&callee_func.label)
                        .unwrap();
                    let new_callers_saved: HashSet<Reg> =
                        old_callers_saved.iter().cloned().collect();
                    self.caller_regs_to_saveds
                        .insert(new_name.clone(), new_callers_saved);
                    // 把新函数加入到名称表
                    new_name_func.insert(new_name.clone(), new_callee_func);
                    if_finish = false; //设置修改符号为错
                }
            }
            func_to_process = new_funcs;
            if if_finish {
                break;
            }
        }
        //加入name_func中的外部函数
        for (name, func) in self.name_func.iter() {
            if func.is_extern {
                new_name_func.insert(name.clone(), *func);
            }
        }
        self.name_func = new_name_func; //修改完成后只有名称表内的函数才是有用的函数
                                        // debug_assert!(false, "{}", self.name_func.len())
    }

    /// 计算栈空间,进行ra,sp,callee 的保存和恢复
    pub fn build_stack_info(&mut self, f: &mut File) {
        for (name, func) in self.name_func.iter() {
            debug_assert!(!func.is_extern);
            if func.label == "main" {
                func.as_mut().callee_saved.clear(); // main函数不需要保存任何callee saved
            } else {
                let callees = self.callee_regs_to_saveds.get_mut(name).unwrap();
                callees.remove(&Reg::new(2, crate::utility::ScalarType::Int)); //sp虽然是callee saved但不需要通过栈方式restore
                func.as_mut().callee_saved.extend(callees.iter());
            }
            func.as_mut().save_callee(f);
        }
    }

    ///删除进行函数分裂后的剩余无用函数
    pub fn remove_useless_func(&mut self) {
        self.name_func.retain(|_, f| !f.is_extern);
    }

    pub fn update_array_offset(&mut self, pool: &mut BackendPool) {
        for (_, func) in self.name_func.iter() {
            debug_assert!(!func.is_extern);
            func.as_mut().update_array_offset(pool);
        }
    }

    pub fn print_func(&self) {
        // debug_assert!(false, "{}", self.name_func.len());
        for (_, func) in self.name_func.iter() {
            func.print_func();
        }
    }
}

/// build v4:
/// 1.寄存器重分配:针对call上下文调整函数寄存器组成
/// 2.针对函数是否为main调整寄存器组成
impl AsmModule {
    pub fn build_v4(&mut self, f: &mut File, _f2: &mut File, pool: &mut BackendPool, is_opt: bool) {
        self.build_lir(pool);
        self.remove_unuse_inst_pre_alloc();

        // self.generate_row_asm(_f2, pool); //generate row  asm可能会造成bug

        if is_opt {
            // gep偏移计算合并
            BackendPass::new(ObjPtr::new(self)).opt_gep();

            // 设置一些寄存器为临时变量
            self.cal_tmp_var();

            // 对非临时寄存器进行分配
            self.allocate_reg();
            // 将非临时寄存器映射到物理寄存器
            self.map_v_to_p();
            // 代码调度，列表调度法
            self.list_scheduling_tech();

            // 为临时寄存器分配寄存器
            self.clear_tmp_var();
            self.allocate_reg();
            self.map_v_to_p();
        } else {
            // self.generate_row_asm(_f2, pool);
            self.allocate_reg();
            // self.generate_row_asm(_f2, pool);
            // self.map_v_to_p();
            // self.generate_row_asm(_f2, pool);
            // 重分配
            // self.name_func.iter().for_each(|(_, func)| {
            //     func.as_mut()
            //         .p2v_pre_handle_call(Reg::get_all_recolorable_regs())
            // });
            // // self.generate_row_asm(_f2, pool);
            // self.allocate_reg();
            self.map_v_to_p();
        }
        self.remove_unuse_inst_suf_alloc();
        //在寄存器分配后跑两遍寄存器接合
        // for i in 0..2 {
        //     self.p2v();
        //     self.allocate_reg();
        //     self.map_v_to_p();
        //     self.remove_unuse_inst_suf_alloc();
        // }

        self.handle_spill_v3(pool);
        self.remove_unuse_inst_suf_alloc();

        self.remove_useless_func(); //删除handle spill 后面可能产生的冗余指令

        self.add_external_func(pool);
        self.anaylyse_for_handle_call_v3();

        //寄存器重分配,重分析
        self.realloc_reg_with_priority();
        self.anaylyse_for_handle_call_v3();

        if is_opt {
            self.split_func(pool);
        }

        // self.split_func(pool);
        self.reduce_callee_used_after_func_split();

        self.remove_useless_func(); //在handle call之前调用,删掉前面往name func中加入的external func
        self.handle_call_v3(pool);
        self.rearrange_stack_slot();
        self.update_array_offset(pool);
        self.build_stack_info(f);

        // self.print_func();
        //删除无用的函数
    }

    //构建函数调用表
    pub fn build_call_map(
        name_func: &HashMap<String, ObjPtr<Func>>,
    ) -> HashMap<String, HashSet<String>> {
        let mut call_map = HashMap::new();
        for (name, func) in name_func.iter() {
            // if func.is_extern {continue;}
            let mut callee_funcs: HashSet<String> = HashSet::new();
            for bb in func.blocks.iter() {
                bb.insts
                    .iter()
                    .filter(|inst| inst.get_type() == InstrsType::Call)
                    .for_each(|inst| {
                        let func_name = inst.get_func_name().unwrap();
                        callee_funcs.insert(func_name);
                    });
            }
            call_map.insert(name.clone(), callee_funcs);
        }
        call_map
    }

    pub fn p2v(&mut self) {
        self.name_func
            .iter()
            .filter(|(_, f)| !f.is_extern)
            .for_each(|(_, f)| {
                f.as_mut()
                    .p2v_pre_handle_call(Reg::get_all_recolorable_regs());
            });
    }

    ///使用进行函数分析后的结果先进行寄存器组成重构
    pub fn realloc_reg_with_priority(&mut self) {
        //记录除了main函数外每个函数使用到的 callee saved和caller saved 需要的恢复次数
        let mut callee_saved_times: HashMap<ObjPtr<Func>, HashMap<Reg, usize>> = HashMap::new();

        let mut callee_used = self.build_callee_used();

        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.calc_live_for_handle_call();
            for bb in func.blocks.iter() {
                let mut livenow: HashSet<Reg> = HashSet::new();
                bb.live_out.iter().for_each(|reg| {
                    livenow.insert(*reg);
                });
                for inst in bb.insts.iter().rev() {
                    for reg in inst.get_reg_def() {
                        livenow.remove(&reg);
                    }
                    if inst.get_type() == InstrsType::Call {
                        let mut callees_to_saved: HashSet<Reg> = HashSet::new();
                        let func_called = inst.get_func_name().unwrap();
                        let callee_used = callee_used.get(func_called.as_str()).unwrap();
                        let func_called = self.name_func.get(func_called.as_str()).unwrap();
                        callees_to_saved = callees_to_saved
                            .iter()
                            .filter(|reg| callee_used.contains(&reg))
                            .cloned()
                            .collect();
                        if !callee_saved_times.contains_key(func_called) {
                            callee_saved_times.insert(*func_called, HashMap::new());
                        }
                        for callee_to_saved in callees_to_saved.iter() {
                            let new_times = callee_saved_times
                                .get(func_called)
                                .unwrap()
                                .get(callee_to_saved)
                                .unwrap_or(&0)
                                + 1;
                            callee_saved_times
                                .get_mut(func_called)
                                .unwrap()
                                .insert(*callee_to_saved, new_times);
                        }
                    }
                    for reg in inst.get_reg_use() {
                        livenow.insert(reg);
                    }
                }
            }
        }
        let call_map = AsmModule::build_call_map(&self.name_func);
        //对每个函数进行试图减少指定寄存器的使用
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            let func = *func;
            //按照每个函数使用被调用时需要保存的自身使用到的callee saved寄存器的数量
            let callee_saved_time = callee_saved_times.get(&func);
            if callee_saved_time.is_none() {
                break;
            }
            let callee_saved_time = callee_saved_time.unwrap();
            let mut callees: Vec<Reg> = callee_saved_time.iter().map(|(reg, _)| *reg).collect();
            callees.sort_by_cached_key(|reg| callee_saved_time.get(reg));
            let caller_used = self.build_caller_used();
            let callee_used = self.build_callee_used();
            let self_used = func.draw_used_callees();
            //自身调用的函数使用到的callee saved寄存器
            let mut callee_func_used: HashSet<Reg> = HashSet::new();
            for func_called in call_map.get(func.label.as_str()).unwrap() {
                if func_called == func.label.as_str() {
                    continue;
                }
                let callee_used_of_func_called = callee_used.get(func_called).unwrap();
                callee_func_used.extend(callee_used_of_func_called.iter());
            }
            //对于自身使用到的callee_used的寄存器
            // let mut self_callee_used
            //从该函数需要保存次数最多的寄存器开始ban
            let mut baned = HashSet::new();
            for reg in callees.iter().rev() {
                if !self_used.contains(reg) {
                    continue;
                }
                if callee_func_used.contains(reg) {
                    continue;
                }
                let ok = func
                    .as_mut()
                    .try_ban_certain_reg(reg, &caller_used, &callee_used);
                if ok {
                    log_file!("ban_reg.txt", "{}", reg);
                    baned.insert(*reg);
                }
            }
            let caller_used = self.build_caller_used();
            let callee_used = self.build_callee_used();
            for reg in callees.iter().rev() {
                if !self_used.contains(reg) {
                    continue;
                }
                if baned.contains(reg) {
                    continue;
                }
                let ok = func
                    .as_mut()
                    .try_ban_certain_reg(reg, &caller_used, &callee_used);
                if ok {
                    log_file!("ban_reg.txt", "{}", reg);
                }
            }
        }

        //对于main函数单独处理
    }

    pub fn build_callee_used(&self) -> HashMap<String, HashSet<Reg>> {
        let mut calleed_useds = HashMap::new();
        for (_, func) in self.name_func.iter() {
            let callees_used = self.draw_callee_used(*func);
            calleed_useds.insert(func.label.clone(), callees_used);
        }
        calleed_useds
    }
    pub fn build_caller_used(&self) -> HashMap<String, HashSet<Reg>> {
        let mut caller_useds = HashMap::new();
        for (_, func) in self.name_func.iter() {
            let callers_used = self.draw_caller_used(*func);
            caller_useds.insert(func.label.clone(), callers_used);
        }
        caller_useds
    }

    ///重新分析出一个函数递归地影响到的callee saved的寄存器的组成
    pub fn draw_callee_used(&self, func: ObjPtr<Func>) -> HashSet<Reg> {
        let mut new_callee_uesd: HashSet<Reg> = func.draw_used_callees();
        // 首先递归地找到这个函数内部调用过地所有函数集合
        let mut callee_funcs: HashSet<ObjPtr<Func>> = HashSet::new();
        loop {
            let last_len = callee_funcs.len();
            for bb in func.blocks.iter() {
                for inst in bb.insts.iter() {
                    if inst.get_type() != InstrsType::Call {
                        continue;
                    }
                    let func_called = inst.get_func_name().unwrap();
                    let func_called = self.name_func.get(func_called.as_str()).unwrap();
                    callee_funcs.insert(*func_called);
                }
            }
            let mut new_callee_funcs = HashSet::new();
            for func_called in callee_funcs.iter() {
                for bb in func_called.blocks.iter() {
                    for inst in bb.insts.iter() {
                        if inst.get_type() != InstrsType::Call {
                            continue;
                        }
                        let func_called = inst.get_func_name().unwrap();
                        let func_called = self.name_func.get(func_called.as_str()).unwrap();
                        new_callee_funcs.insert(*func_called);
                    }
                }
            }
            callee_funcs.extend(new_callee_funcs.iter());
            if last_len == callee_funcs.len() {
                break;
            }
        }
        for func_called in callee_funcs.iter() {
            let callee_used = func_called.draw_used_callees();
            new_callee_uesd.extend(callee_used);
        }
        new_callee_uesd
    }

    ///递归分析一个函数调用影响到的caller saved寄存器
    pub fn draw_caller_used(&self, func_called: ObjPtr<Func>) -> HashSet<Reg> {
        let mut new_callers_used: HashSet<Reg> = func_called.draw_used_callers();
        // 首先递归地找到这个函数内部调用过地所有函数集合
        let mut funcs_called: HashSet<ObjPtr<Func>> = HashSet::new();
        loop {
            let last_len = funcs_called.len();
            for bb in func_called.blocks.iter() {
                for inst in bb.insts.iter() {
                    if inst.get_type() != InstrsType::Call {
                        continue;
                    }
                    let func_called = inst.get_func_name().unwrap();
                    let func_called = self.name_func.get(func_called.as_str()).unwrap();
                    funcs_called.insert(*func_called);
                }
            }
            let mut new_callee_funcs = HashSet::new();
            for func_called in funcs_called.iter() {
                for bb in func_called.blocks.iter() {
                    for inst in bb.insts.iter() {
                        if inst.get_type() != InstrsType::Call {
                            continue;
                        }
                        let func_called = inst.get_func_name().unwrap();
                        let func_called = self.name_func.get(func_called.as_str()).unwrap();
                        new_callee_funcs.insert(*func_called);
                    }
                }
            }
            funcs_called.extend(new_callee_funcs.iter());
            if last_len == funcs_called.len() {
                break;
            }
        }
        for func_called in funcs_called.iter() {
            let callee_used = func_called.draw_used_callers();
            new_callers_used.extend(callee_used);
        }
        new_callers_used
    }

    ///函数分裂后减少使用到的特定物理寄存器
    /// 该函数调用应该在remove useless func之前,
    /// 该函数的调用结果依赖于调用该函数前进行的analyse for handle call
    pub fn reduce_callee_used_after_func_split(&mut self) {
        //对于 main函数中的情况专门处理, 对于 call前后使用 的 caller saved函数进行重分配,尝试进行recolor,(使用任意寄存器)
        let func = self.name_func.get("main").unwrap();
        //标记重整
        let mut to_recolor: Vec<(ObjPtr<LIRInst>, Reg)> = Vec::new();
        let caller_used = AsmModule::build_caller_used(&self);
        let mut constraints: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        func.calc_live_for_handle_call();
        for bb in func.blocks.iter() {
            let mut live_now: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                live_now.insert(*reg);
            });
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_def() {
                    live_now.remove(&reg);
                }
                if inst.get_type() == InstrsType::Call {
                    //
                    let func_name = inst.get_func_name().unwrap();
                    debug_assert!(
                        self.name_func.contains_key(func_name.as_str()),
                        "{}",
                        func_name
                    );
                    let func = self.name_func.get(func_name.as_str()).unwrap();
                    if !func.is_extern {
                        let caller_to_saved = caller_used.get(func_name.as_str()).unwrap();
                        let mut reg_cross = HashSet::new();
                        for reg in live_now.iter() {
                            if !caller_to_saved.contains(reg) {
                                continue;
                            }
                            reg_cross.insert(*reg);
                        }
                        //call指令往后,call指令往前
                        let mut to_forward: LinkedList<(ObjPtr<BB>, i32, Reg)> = LinkedList::new();
                        let mut to_backward: LinkedList<(ObjPtr<BB>, i32, Reg)> = LinkedList::new();
                        let mut p2v: HashMap<Reg, Reg> = HashMap::new();
                        for reg in reg_cross.iter() {
                            to_forward.push_back((*bb, index as i32, *reg));
                            to_backward.push_back((*bb, index as i32, *reg));
                            let v_reg = Reg::init(reg.get_type());
                            p2v.insert(*reg, v_reg);
                            constraints.insert(*reg, caller_to_saved.clone());
                        }
                        let p2v: HashMap<Reg, Reg> = p2v;
                        let mut backward_passed: HashSet<(ObjPtr<BB>, i32, Reg)> = HashSet::new();
                        let mut forward_passed: HashSet<(ObjPtr<BB>, i32, Reg)> = HashSet::new();
                        //对遇到的符合要求的物理寄存器进行p2v,并且缓存等待重着色
                        loop {
                            while !to_forward.is_empty() {
                                let item = to_forward.pop_front().unwrap();
                                if forward_passed.contains(&item) {
                                    continue;
                                }
                                forward_passed.insert(item);
                                let (bb, mut index, reg) = item;
                                let v_reg = *p2v.get(&reg).unwrap();
                                index += 1;
                                debug_assert!(index >= 0);
                                while index < bb.insts.len() as i32 {
                                    let inst = bb.insts.get(index as usize).unwrap();
                                    if inst.get_reg_use().contains(&reg) {
                                        inst.as_mut().replace_only_use_reg(&reg, &v_reg);
                                        to_recolor.push((*inst, v_reg));
                                    }
                                    if inst.get_reg_def().contains(&reg) {
                                        break;
                                    }
                                    index += 1;
                                }
                                if index < bb.insts.len() as i32 {
                                    continue;
                                }
                                //传播到外界,并且对于进入外界的情况还会反向传播,
                                let mut new_forward = Vec::new();
                                for out_bb in bb.out_edge.iter() {
                                    //out_bb里面通过
                                    if !out_bb.live_in.contains(&reg) {
                                        continue;
                                    }
                                    to_forward.push_back((*out_bb, -1, reg));
                                    new_forward.push(out_bb);
                                }
                                for bb in new_forward.iter() {
                                    for in_bb in bb.in_edge.iter() {
                                        if !in_bb.live_out.contains(&reg) {
                                            continue;
                                        }
                                        to_backward.push_back((
                                            *in_bb,
                                            in_bb.insts.len() as i32,
                                            reg,
                                        ));
                                    }
                                }
                            }
                            while !to_backward.is_empty() {
                                let item = to_backward.pop_front().unwrap();
                                if backward_passed.contains(&item) {
                                    continue;
                                }
                                backward_passed.insert(item);
                                let (bb, mut index, reg) = item;
                                let v_reg = *p2v.get(&reg).unwrap();
                                index -= 1;
                                debug_assert!(index < bb.insts.len() as i32);
                                while index >= 0 {
                                    let inst = bb.insts.get(index as usize).unwrap();
                                    if inst.get_reg_def().contains(&reg) {
                                        inst.as_mut().replace_only_def_reg(&reg, &v_reg);
                                        to_recolor.push((*inst, v_reg));
                                        break;
                                    }
                                    if inst.get_reg_use().contains(&reg) {
                                        inst.as_mut().replace_only_use_reg(&reg, &v_reg);
                                        to_recolor.push((*inst, v_reg));
                                    }
                                    index -= 1;
                                }
                                if index >= 0 {
                                    continue;
                                }
                                let mut new_backward = Vec::new();
                                for in_bb in bb.in_edge.iter() {
                                    if !in_bb.live_out.contains(&reg) {
                                        continue;
                                    }
                                    to_backward.push_back((*in_bb, in_bb.insts.len() as i32, reg));
                                    new_backward.push(*in_bb);
                                }
                                for bb in new_backward.iter() {
                                    for out_bb in bb.out_edge.iter() {
                                        if !out_bb.live_in.contains(&reg) {
                                            continue;
                                        }
                                        to_forward.push_back((*out_bb, -1, reg));
                                    }
                                }
                            }
                            if to_forward.is_empty() && to_backward.is_empty() {
                                break;
                            }
                        }
                    }
                }
                for reg in inst.get_reg_use() {
                    live_now.insert(reg);
                }
            }
        }
        //获取增加约束的重新分配结果
        func.calc_live_for_handle_call(); //该处在handle spill之后，应该calc live for handle call
        let mut allocator = easy_gc_alloc::Allocator::new();
        let alloc_stat = allocator.alloc_with_constraint(func, &constraints);
        let alloc_stat = if alloc_stat.spillings.len() == 0 {
            alloc_stat
        } else {
            let mut allocator = easy_gc_alloc::Allocator::new();
            let alloc_stat = allocator.alloc(func);
            alloc_stat
        };
        debug_assert!(alloc_stat.spillings.len() == 0);
        //使用新分配结果重新v2p
        let colors = alloc_stat.dstr;
        for (inst, v_reg) in to_recolor {
            let p_color = colors.get(&v_reg.get_id()).unwrap();
            let p_reg = Reg::from_color(*p_color);
            inst.as_mut().replace_reg(&v_reg, &p_reg);
        }
        return;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    pub fn test_hash() {
        let mut set = HashSet::new();
        for i in 0..=10000000 {
            let if_insert: bool = rand::random();
            if if_insert {
                set.insert(i);
            }
        }
        let set2 = set.clone();
        assert!(set.len() == set2.len());
        for v in set.iter() {
            assert!(set2.contains(v));
        }
    }
}
