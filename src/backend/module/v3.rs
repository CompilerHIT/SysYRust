use super::*;

/// build v3:
/// 1. 实现 函数分裂, 优化callee的保存恢复
/// 2. 指令级 上下文 caller 选择
/// 3. 对spill use和caller use的栈空间 紧缩
/// 4. 删除无用函数模板(可选)
impl AsmModule {
    ///处理spillings的虚拟寄存器的临时物理寄存器借用
    pub fn handle_spill_v3(&mut self, pool: &mut BackendPool) {
        self.name_func.iter().for_each(|(_, func)| {
            if func.is_extern {
                return;
            }
            // Func::print_func(*func, "pre_spill.txt");
            func.as_mut().handle_spill_v3(pool);
            // Func::print_func(*func, "suf_spill.txt");
            // func.as_mut().handle_spill_v2(pool);
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
    /// 1. 该函数应该在vtop和handle spill后调用
    pub fn anaylyse_for_handle_call_v3_pre_split(&mut self) {
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
    pub fn split_func(&mut self, pool: &mut BackendPool) {
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
                callees.remove(&Reg::get_sp()); //sp虽然是callee saved但不需要通过栈方式restore
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
}
