use std::collections::{HashMap, HashSet, LinkedList};
use std::fs::File;
use std::hash::Hash;
use std::io::Write;

use rand::seq::index;

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
use super::regalloc::regalloc::Regalloc;
use super::regalloc::structs::FuncAllocStat;
use super::structs::GenerateAsm;

pub struct AsmModule {
    pub global_var_list: Vec<(ObjPtr<Inst>, GlobalVar)>,
    pub func_map: Vec<(ObjPtr<Function>, ObjPtr<Func>)>,
    call_map: HashMap<String, HashSet<String>>,
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
        self.anaylyse_for_handle_call_v3_pre_split();

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
        self.name_func.iter().for_each(|(_, func)| {
            if func.is_extern {
                return;
            }
            func.as_mut().handle_spill_v3(pool)
        });
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
        // let extern_func_path = "extern_func.txt";
        //补充外部函数 memset 和memcpy
        let extern_funcs = vec![
            "memset@plt",
            "memcpy@plt",
            "putint",
            "getint",
            "getarray",
            "putarray",
            "getch",
            "putch",
            "getfloat",
            "putfloat",
            "getfarray",
            "putfarray",
            "putf",
            "_sysy_starttime",
            "_sysy_stoptime",
        ];
        for name in extern_funcs.iter() {
            build_external_func(self, &name, pool);
        }
    }

    ///准备 callee save和caller save需要的信息
    /// 1. 准备每个函数需要的callee save,以及进行函数分裂
    /// 2. 针对性地让函数自我转变 , 调整每个函数中使用到的寄存器分布等等
    /// 3. 该函数应该在vtop和handle spill后调用
    /// 4. 过程中会往name func中加入需要的外部函数的信息
    fn anaylyse_for_handle_call_v3_pre_split(&mut self) {
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
        for (name, _) in self.name_func.iter() {
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
        // // debug_assert!(false, "{}", self.name_func.len());
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            Func::print_func(*func);
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

        //加入外部函数
        self.add_external_func(pool);

        //建立调用表
        self.build_own_call_map();
        //寄存器重分配,重分析

        // self.realloc_reg_with_priority();

        self.handle_spill_v3(pool);
        self.remove_unuse_inst_suf_alloc();

        self.anaylyse_for_handle_call_v3_pre_split();
        // self.anaylyse_for_handle_call_v4();

        if is_opt {
            self.split_func(pool);
            self.build_own_call_map();
            // self.anaylyse_for_handle_call_v3();
        }
        // self.split_func(pool);
        // self.print_func();
        // self.reduce_caller_to_saved_after_func_split();

        self.remove_useless_func(); //在handle call之前调用,删掉前面往name func中加入的external func
        self.handle_call_v3(pool);
        self.rearrange_stack_slot();
        self.update_array_offset(pool);
        self.build_stack_info(f);

        // self.print_func();
        //删除无用的函数
    }

    ///建立函数间的直接调用表
    pub fn build_own_call_map(&mut self) {
        let mut call_map = HashMap::new();
        // let mut calls = Vec::new();
        for (name, func) in self.name_func.iter() {
            // if func.is_extern {continue;}
            let mut callee_funcs: HashSet<String> = HashSet::new();
            for bb in func.blocks.iter() {
                bb.insts
                    .iter()
                    .filter(|inst| inst.get_type() == InstrsType::Call)
                    .for_each(|inst| {
                        let func_name = inst.get_func_name().unwrap();
                        callee_funcs.insert(func_name.clone());
                        // calls.push((name.clone(), func_name));
                    });
            }
            call_map.insert(name.clone(), callee_funcs);
        }
        // loop {
        //     let finish_flag = true;
        //     for (call_func, callee_func) in calls.iter() {

        //     }
        //     if finish_flag {
        //         break;
        //     }
        // }
        self.call_map = call_map;
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

    pub fn anaylyse_for_handle_call_v4(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        let caller_used = self.build_caller_used();
        let callee_used = self.build_callee_used();
        ///对于name_func里面的每个函数,除了externel,都要总结个to save
        self.callee_regs_to_saveds.clear();
        self.caller_regs_to_saveds.clear();
        for (name, func) in self.name_func.iter() {
            //
            self.callee_regs_to_saveds
                .insert(name.clone(), func.draw_used_callees());
            self.caller_regs_to_saveds
                .insert(name.clone(), func.draw_used_callers());
        }
        for (_name, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.calc_live_for_handle_call();
            AsmModule::analyse_inst_with_live_now(func, &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let name_of_func_called = inst.get_func_name().unwrap();
                let mut caller_used = caller_used
                    .get(name_of_func_called.as_str())
                    .unwrap()
                    .clone();
                let mut callee_used = callee_used
                    .get(name_of_func_called.as_str())
                    .unwrap()
                    .clone();
                caller_used.retain(|reg| live_now.contains(reg));
                callee_used.retain(|reg| live_now.contains(reg));
                self.callee_regs_to_saveds
                    .get_mut(name_of_func_called.as_str())
                    .unwrap()
                    .extend(callee_used);
                self.caller_regs_to_saveds
                    .get_mut(name_of_func_called.as_str())
                    .unwrap()
                    .extend(caller_used);
            });
        }
    }

    pub fn split_func_v4(&mut self, pool: &mut BackendPool) {
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
        for (name, _) in self.name_func.iter() {
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

        //保留原name_func中的外部函数
        for (name, func) in self.name_func.iter() {
            if func.is_extern {
                new_name_func.insert(name.clone(), *func);
            }
        }
        self.name_func = new_name_func; //修改完成后只有名称表内的函数才是有用的函数
                                        // debug_assert!(false, "{}", self.name_func.len())
    }

    ///使用进行函数分析后的结果先进行寄存器组成重构
    pub fn realloc_reg_with_priority(&mut self) {
        //记录除了main函数外每个函数使用到的 callee saved和caller saved 需要的恢复次数
        let mut callee_saved_times: HashMap<ObjPtr<Func>, HashMap<Reg, usize>> = HashMap::new();

        let callee_used = self.build_callee_used();

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
                        let func_called = inst.get_func_name().unwrap();
                        let callee_used = callee_used.get(func_called.as_str()).unwrap();
                        let func_called = self.name_func.get(func_called.as_str()).unwrap();
                        let callees_to_saved: HashSet<Reg> = livenow
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
        let call_map = &self.call_map;
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
            let mut callees: Vec<Reg> = callee_saved_time
                .iter()
                .map(|(reg, _)| *reg)
                .filter(|reg| reg != &Reg::get_sp())
                .collect();
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
        // return;
        //对于main函数单独处理
        //节省callee,能够节省多少节省多少 (然后试图节省caller)
        self.realloc_main_with_priority_pre_split();
    }

    fn realloc_main_with_priority_pre_split(&mut self) {
        let callee_used = self.build_callee_used();
        let main_func = self.name_func.get("main").unwrap();
        main_func
            .as_mut()
            .p2v_pre_handle_call(Reg::get_all_recolorable_regs());
        debug_assert!(main_func.label == "main");

        main_func.as_mut().allocate_reg();
        let mut callee_constraints: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        //然后分析需要加入限制的虚拟寄存器
        //首先尝试进行一波完全寄存器分配
        main_func.calc_live_for_handle_call();
        AsmModule::analyse_inst_with_live_now(&main_func, &mut |inst, live_now| {
            if inst.get_type() != InstrsType::Call {
                return;
            }
            //对于 call指令,分析上下文造成的依赖关系
            let func_name = inst.get_func_name().unwrap();
            let func = self.name_func.get(func_name.as_str()).unwrap();
            if func.is_extern {
                //遇到 is_extern的情况,不能节省,也不应节省
                return;
            } else {
                let callee_used = callee_used.get(func.label.as_str()).unwrap();
                for reg in live_now.iter() {
                    if reg.is_physic() {
                        continue;
                    }

                    if !callee_constraints.contains_key(reg) {
                        callee_constraints.insert(*reg, callee_used.clone());
                    } else {
                        callee_constraints.get_mut(reg).unwrap().extend(callee_used);
                    }
                }
            }
        });
        //约束建立好之后尝试寄存器分配 (如果实在分配后存在spill,就只好存在spill了)
        let alloc_stat = || -> FuncAllocStat {
            //首先尝试获取一个基础的分配结果
            let base_alloc_stat = main_func.as_mut().reg_alloc_info.to_owned(); //首先暂存最初分配结果
                                                                                // main_func.as_mut().reg_alloc_info = FuncAllocStat::new();

            let try_alloc = |spill_limit_num: usize,
                             callee_constraints: HashMap<Reg, HashSet<Reg>>|
             -> Option<FuncAllocStat> {
                let mut callee_constraints = callee_constraints;
                loop {
                    let mut allocator = easy_gc_alloc::Allocator::new();
                    let alloc_stat =
                        allocator.alloc_with_constraint(&main_func, &callee_constraints);
                    //每次减半直到分配成功
                    if alloc_stat.spillings.len() <= spill_limit_num {
                        return Some(alloc_stat);
                    }
                    //否则分配失败,减少约束再分配
                    //首先减半
                    let ord: Vec<Reg> = callee_constraints.iter().map(|(reg, _)| *reg).collect();
                    for reg in ord.iter() {
                        let mut new_baned: HashSet<Reg> = HashSet::new();
                        let old_baneds = callee_constraints.get(reg).unwrap();
                        for old_ban in old_baneds {
                            if new_baned.len() >= old_baneds.len() / 2 {
                                break;
                            }
                            new_baned.insert(*old_ban);
                        }
                        if new_baned.len() == 0 {
                            callee_constraints.remove(reg);
                        } else {
                            callee_constraints
                                .get_mut(reg)
                                .unwrap()
                                .retain(|reg| new_baned.contains(reg));
                        }
                    }
                    //如果约束消失,则退出
                    if callee_constraints.len() == 0 {
                        break;
                    }
                }
                debug_assert!(callee_constraints.len() == 0);
                None
            };
            let alloc_stat = try_alloc(0, callee_constraints.clone());
            if let Some(alloc_stat) = alloc_stat {
                return alloc_stat;
            }

            main_func.calc_live_for_alloc_reg();
            let alloc_stat = try_alloc(base_alloc_stat.spillings.len(), callee_constraints);
            if let Some(alloc_stat) = alloc_stat {
                return alloc_stat;
            }
            return base_alloc_stat;
        };
        let alloc_stat = alloc_stat();
        // let alloc_stat = main_func.reg_alloc_info.to_owned();

        main_func.as_mut().v2p(&alloc_stat.dstr);
        main_func.as_mut().reg_alloc_info = alloc_stat;
        main_func
            .context
            .as_mut()
            .set_reg_map(&main_func.reg_alloc_info.dstr);
    }

    ///最后得到的表中不会包含sp
    pub fn build_callee_used(&self) -> HashMap<String, HashSet<Reg>> {
        let mut calleed_useds = HashMap::new();
        for (_, func) in self.name_func.iter() {
            let mut callees_used = self.draw_callee_used(*func);
            callees_used.remove(&Reg::get_sp());
            calleed_useds.insert(func.label.clone(), callees_used);
        }
        calleed_useds
    }

    //最后得到的表中不会包含ra
    pub fn build_caller_used(&self) -> HashMap<String, HashSet<Reg>> {
        let mut caller_useds = HashMap::new();
        for (_, func) in self.name_func.iter() {
            let mut callers_used = self.draw_caller_used(*func);
            callers_used.remove(&Reg::get_ra());
            caller_useds.insert(func.label.clone(), callers_used);
        }
        caller_useds
    }

    ///重新分析出一个函数递归地影响到的callee saved的寄存器的组成
    /// 它只会统计该函数用到的callee saved以及它调用的非外部函数用到的callee saved寄存器
    pub fn draw_callee_used(&self, func: ObjPtr<Func>) -> HashSet<Reg> {
        if func.is_extern {
            return HashSet::new();
        }
        let mut new_callee_uesd: HashSet<Reg> = func.draw_used_callees();
        // 首先递归地找到这个函数内部调用过地所有函数集合
        let mut callee_funcs: HashSet<ObjPtr<Func>> = HashSet::new();
        // let call_map = AsmModule::build_call_map(name_func);
        for func in self.call_map.get(func.label.as_str()).unwrap() {
            let func = self.name_func.get(func).unwrap();
            if func.is_extern {
                continue;
            }
            callee_funcs.insert(*func);
        }
        //处理多重递归调用的情况
        loop {
            let mut break_flag = true;
            let mut callee_to_add = HashSet::new();
            for func in callee_funcs.iter() {
                let func = *func;
                for func in self.call_map.get(func.label.as_str()).unwrap() {
                    let func = *self.name_func.get(func).unwrap();
                    if func.is_extern {
                        continue;
                    }
                    if !callee_funcs.contains(&func) && !callee_to_add.contains(&func) {
                        callee_to_add.insert(func);
                        break_flag = false;
                    }
                }
            }
            callee_funcs.extend(callee_to_add);
            if break_flag {
                break;
            }
        }
        for func_called in callee_funcs.iter() {
            debug_assert!(!func_called.is_extern);
            let callee_used = func_called.draw_used_callees();
            new_callee_uesd.extend(callee_used);
        }
        new_callee_uesd
    }

    ///递归分析一个函数调用影响到的caller saved寄存器=
    pub fn draw_caller_used(&self, func: ObjPtr<Func>) -> HashSet<Reg> {
        let mut new_callers_used: HashSet<Reg> = func.draw_used_callers();
        // 首先递归地找到这个函数内部调用过地所有函数集合
        let mut callee_funcs: HashSet<ObjPtr<Func>> = HashSet::new();
        if func.is_extern {
            return Reg::get_all_callers_saved();
        }
        for func in self.call_map.get(func.label.as_str()).unwrap() {
            let func = self.name_func.get(func).unwrap();
            callee_funcs.insert(*func);
            if func.is_extern {
                return Reg::get_all_callers_saved();
            }
        }
        //处理多重递归调用的情况
        loop {
            let mut break_flag = true;
            let mut callee_to_add = HashSet::new();
            for func in callee_funcs.iter() {
                let func = *func;
                for func in self.call_map.get(func.label.as_str()).unwrap() {
                    let func = *self.name_func.get(func).unwrap();
                    if func.is_extern {
                        return Reg::get_all_callers_saved();
                    }
                    if !callee_funcs.contains(&func) && !callee_to_add.contains(&func) {
                        callee_to_add.insert(func);
                        break_flag = false;
                    }
                }
            }
            callee_funcs.extend(callee_to_add);
            if break_flag {
                break;
            }
        }
        for func in callee_funcs.iter() {
            debug_assert!(!func.is_extern);
            let caller_used = func.draw_used_callers();
            new_callers_used.extend(caller_used);
        }
        new_callers_used.extend(func.draw_used_callers());
        new_callers_used
    }

    ///函数分裂后减少使用到的特定物理寄存器
    /// 该函数调用应该在remove useless func之前,
    /// 该函数的调用结果依赖于调用该函数前进行的analyse for handle call
    /// handlespill->[split func]->reduce_ctsafs->handle call
    pub fn reduce_caller_to_saved_after_func_split(&mut self) {
        //对于 main函数中的情况专门处理, 对于 call前后使用 的 caller saved函数进行重分配,尝试进行recolor,(使用任意寄存器)
        let func = self.name_func.get("main").unwrap();
        let mut passed_i_p: HashSet<(ObjPtr<LIRInst>, Reg, bool)> = HashSet::new();
        //标记重整
        let mut inst_vreg_preg: Vec<(ObjPtr<LIRInst>, Reg, Reg, bool)> = Vec::new();
        let caller_used = AsmModule::build_caller_used(&self);
        let callee_used = AsmModule::build_callee_used(&self);
        let mut constraints: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        func.calc_live_for_handle_call();
        debug_assert!(func.label == "main");

        //对于handle call这一步已经没有物理寄存器了,所有遇到的虚拟寄存器都是该处产生的
        AsmModule::analyse_inst_with_index_and_live_now(func, &mut |inst, index, live_now, bb| {
            //
            if inst.get_type() != InstrsType::Call {
                return;
            }
            let callee_func_name = inst.get_func_name().unwrap();
            debug_assert!(
                self.name_func.contains_key(callee_func_name.as_str()),
                "{}",
                callee_func_name
            );
            let callee_func = self.name_func.get(callee_func_name.as_str()).unwrap();
            let callee_used = if callee_func.is_extern {
                HashSet::new()
            } else {
                callee_used.get(callee_func_name.as_str()).unwrap().clone()
            };
            let caller_used = caller_used.get(callee_func_name.as_str()).unwrap();
            let mut reg_cross = live_now.clone();
            reg_cross.retain(|reg| {
                reg.get_color() > 4 && reg.is_physic() && !inst.get_regs().contains(reg)
            });
            for reg in reg_cross.iter() {
                let dinsts = AsmModule::get_to_recolor(bb, index, *reg);
                let mut ok = true;
                dinsts.iter().for_each(|item| {
                    let (inst, if_replace_def) = item.clone();
                    if inst.get_type() == InstrsType::Call {
                        ok = false;
                    }
                    if passed_i_p.contains(&(inst, *reg, if_replace_def)) {
                        ok = false;
                    }
                });
                if !ok {
                    continue;
                }
                debug_assert!(dinsts.len() != 0, "func:{},reg:{}", callee_func_name, reg);
                let v_reg = Reg::init(reg.get_type());
                // println!("{}{reg}{v_reg}", callee_func.label);
                constraints.insert(v_reg, caller_used.clone());
                constraints
                    .get_mut(&v_reg)
                    .unwrap()
                    .extend(callee_used.iter());
                // passed_i_p.insert((inst, *reg));
                dinsts.iter().for_each(|(ist, if_replace_def)| {
                    passed_i_p.insert((*ist, *reg, *if_replace_def));
                    inst_vreg_preg.push((*ist, v_reg, *reg, *if_replace_def));
                });
            }
        });
        //获取增加约束的重新分配结果

        //p2v
        // Func::print_func(*func);
        for (inst, v_reg, p_reg, if_replace_def) in inst_vreg_preg.iter() {
            log_file!(
                "p2v_for_reduce_caller.txt",
                "{},{v_reg},{p_reg},{if_replace_def}",
                inst.as_ref()
            );
            if *if_replace_def {
                inst.as_mut().replace_only_def_reg(p_reg, v_reg);
            } else {
                inst.as_mut().replace_only_use_reg(p_reg, v_reg);
            }
        }
        // Func::print_func(*func);

        func.calc_live_for_handle_call(); //该处在handle spill之后，应该calc live for handle call
        let alloc_stat = || -> FuncAllocStat {
            //不断地减少约束,直到能够完美分配 (约束中关于callee的约束不能够减少,关于caller的约束可以减少)

            //首先统计可以减少的约束数量
            let mut num_to_reduce_constraint = 0;
            for (_, constraint) in constraints.iter() {
                for r_inter in constraint.iter() {
                    if r_inter.is_caller_save() {
                        num_to_reduce_constraint += 1;
                    }
                }
            }

            loop {
                let mut allocator = easy_gc_alloc::Allocator::new();
                let alloc_stat = allocator.alloc_with_constraint(func, &constraints);
                if alloc_stat.spillings.len() == 0 {
                    return alloc_stat;
                }
                if num_to_reduce_constraint == 0 {
                    break;
                }
                //减少约束 (以随机的方式(因为难以衡量约束))
                let old_num = num_to_reduce_constraint;
                let keys: Vec<Reg> = constraints.iter().map(|(reg, _)| *reg).collect();
                while num_to_reduce_constraint > old_num * 4 / 5 {
                    //随机找一个约束目标,随机地减少其约束
                    let target_reg: usize = rand::random();
                    let target_reg = target_reg % keys.len();
                    let target_reg = *keys.get(target_reg).unwrap();
                    let r_inter: Vec<Reg> = constraints
                        .get(&target_reg)
                        .unwrap()
                        .iter()
                        .filter(|reg| reg.is_caller_save())
                        .cloned()
                        .collect();
                    for i in 0..r_inter.len() {
                        let if_rm: bool = rand::random();
                        if !if_rm {
                            continue;
                        }
                        let r_inter = r_inter.get(i).unwrap();
                        constraints.get_mut(&target_reg).unwrap().remove(r_inter);
                        num_to_reduce_constraint -= 1;
                    }
                }
            }
            easy_gc_alloc::Allocator::new().alloc(func)
        }();
        debug_assert!(alloc_stat.spillings.len() == 0);
        //使用新分配结果重新v2p
        func.as_mut().v2p(&alloc_stat.dstr);
        func.as_mut().reg_alloc_info = alloc_stat;
        return;
    }
}

///一些进行分析需要用到的工具
impl AsmModule {
    pub fn analyse_inst_with_live_now(
        func: &Func,
        inst_analyser: &mut dyn FnMut(ObjPtr<LIRInst>, &HashSet<Reg>),
    ) {
        for bb in func.blocks.iter() {
            let mut livenow: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                livenow.insert(*reg);
            });
            for inst in bb.insts.iter().rev() {
                for reg in inst.get_reg_def() {
                    livenow.remove(&reg);
                }
                //
                inst_analyser(*inst, &livenow);
                for reg in inst.get_reg_use() {
                    livenow.insert(reg);
                }
            }
        }
    }
    pub fn analyse_inst_with_index_and_live_now(
        func: &Func,
        inst_analyser: &mut dyn FnMut(ObjPtr<LIRInst>, usize, &HashSet<Reg>, ObjPtr<BB>),
    ) {
        for bb in func.blocks.iter() {
            let mut livenow: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                livenow.insert(*reg);
            });
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_def() {
                    livenow.remove(&reg);
                }
                //
                inst_analyser(*inst, index, &livenow, *bb);
                for reg in inst.get_reg_use() {
                    livenow.insert(reg);
                }
            }
        }
    }

    //从某个指令出发,往左右两边延申,反着色某个物理寄存器 (返回去色成功后的指令列表,(所有涉及去色的指令))
    pub fn get_to_recolor(
        bb: ObjPtr<BB>,
        index: usize,
        p_reg: Reg,
    ) -> Vec<(ObjPtr<LIRInst>, bool)> {
        let get_to_recolor_path = "get_to_recolor.txt";
        log_file!(get_to_recolor_path, "start recolor:{}:{}", bb.label, p_reg);
        let mut decolored_insts = Vec::new();
        let mut to_pass: LinkedList<(ObjPtr<BB>, i32, i32)> = LinkedList::new();
        let mut passed = HashSet::new();
        if bb.insts.len() > index && bb.insts.get(index).unwrap().get_reg_def().contains(&p_reg) {
            unreachable!();
            // decolored_insts.push((*bb.insts.get(index).unwrap(),));
        } else {
            to_pass.push_back((bb, index as i32, -1));
        }
        to_pass.push_back((bb, index as i32, 1));
        while !to_pass.is_empty() {
            let (bb, mut index, refresh) = to_pass.pop_front().unwrap();
            if passed.contains(&(bb, index, refresh)) {
                continue;
            }
            // println!("{}:{}:{}", bb.label, index, refresh);
            log_file!(get_to_recolor_path, "{}:{}:{}", bb.label, index, refresh);
            passed.insert((bb, index, refresh));
            index += refresh;
            while index >= 0 && index < bb.insts.len() as i32 {
                log_file!(get_to_recolor_path, "{}:{}:{}", bb.label, index, refresh);
                passed.insert((bb, index, refresh));
                let inst = bb.insts.get(index as usize).unwrap();
                if refresh == 1 {
                    if inst.get_reg_use().contains(&p_reg) {
                        decolored_insts.push((*inst, false));
                        log_file!(get_to_recolor_path, "{}", inst.as_ref());
                    }
                    if inst.get_reg_def().contains(&p_reg) {
                        break;
                    }
                } else if refresh == -1 {
                    if inst.get_reg_def().contains(&p_reg) {
                        decolored_insts.push((*inst, true));
                        log_file!(get_to_recolor_path, "{}", inst.as_ref());
                        break;
                    }
                    if inst.get_reg_use().contains(&p_reg) {
                        decolored_insts.push((*inst, false));
                        log_file!(get_to_recolor_path, "{}", inst.as_ref());
                    }
                } else {
                    unreachable!()
                }
                index += refresh;
            }
            if index >= 0 && index < bb.insts.len() as i32 {
                continue;
            }
            //加入新的块
            let mut new_forward = HashSet::new();
            let mut new_backward = HashSet::new();
            if index < 0 {
                log_file!(get_to_recolor_path, "expand backward");
                for in_bb in bb.in_edge.iter() {
                    log_file!(
                        get_to_recolor_path,
                        "{}'s live out:{:?}",
                        in_bb.label,
                        in_bb.live_out
                    );
                    if in_bb.live_out.contains(&p_reg) {
                        new_backward.insert((*in_bb, in_bb.insts.len() as i32, -1));
                    }
                }
            } else {
                log_file!(get_to_recolor_path, "expand forward");
                for out_bb in bb.out_edge.iter() {
                    log_file!(
                        get_to_recolor_path,
                        "{}'s live in:{:?}",
                        out_bb.label,
                        out_bb.live_in
                    );
                    if out_bb.live_in.contains(&p_reg) {
                        new_forward.insert((*out_bb, -1, 1));
                    }
                }
            }
            log_file!(get_to_recolor_path, "expand backward");
            for (bb, _, _) in new_forward.iter() {
                for in_bb in bb.in_edge.iter() {
                    if in_bb.live_out.contains(&p_reg) {
                        new_backward.insert((*in_bb, in_bb.insts.len() as i32, -1));
                    }
                }
            }
            log_file!(get_to_recolor_path, "expand forward");
            for (bb, _, _) in new_backward.iter() {
                for out_bb in bb.out_edge.iter() {
                    if out_bb.live_in.contains(&p_reg) {
                        new_forward.insert((*out_bb, -1, 1));
                    }
                }
            }
            // todo!();
            for forward in new_forward {
                to_pass.push_back(forward);
            }
            for backward in new_backward {
                to_pass.push_back(backward);
            }
        }

        decolored_insts
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

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

    //该测试表明HashMap的clone也是深clone
    #[test]
    pub fn test_hash_map() {
        let mut set = HashMap::new();
        for i in 0..=100000 {
            let if_insert: bool = rand::random();
            if if_insert {
                let vb: bool = rand::random();
                set.insert(i, vb);
            }
        }
        let set2 = set.clone();
        assert!(set.len() == set2.len());
        for (k, v) in set.iter() {
            assert!(set2.contains_key(k) && set2.get(k).unwrap() == v);
        }
    }
}
