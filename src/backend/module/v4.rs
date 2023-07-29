use std::collections::VecDeque;

use crate::{
    backend::{func, opt},
    ir::CallMap,
};

use super::*;

/// build v4:
/// 1.寄存器重分配:针对call上下文调整函数寄存器组成
/// 2.针对函数是否为main调整寄存器组成
impl AsmModule {
    ///建立函数间的直接调用表
    pub fn build_own_call_map(&mut self) {
        let mut call_map = HashMap::new();

        //首先建立直接函数调用表
        for (name, func) in self.name_func.iter() {
            let mut callee_funcs: HashSet<String> = HashSet::new();
            if func.is_extern {
                call_map.insert(name.clone(), callee_funcs);
                continue;
            }
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

        self.call_map = call_map;
        //然后建立函数调用族
        let func_group = AsmModule::build_func_groups(&self.call_map);
        self.func_groups = func_group;
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

    ///v4的analyse for handle call 依赖于前文调用build call map构建的call map
    pub fn anaylyse_for_handle_call_v4(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        self.analyse_callee_regs_to_saved();
        self.analyse_caller_regs_to_saved();
    }

    ///精确分析caller regs to saved
    pub fn analyse_caller_regs_to_saved(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        let caller_used = self.build_caller_used();
        self.caller_regs_to_saveds.clear();
        for (name, _) in self.name_func.iter() {
            self.caller_regs_to_saveds
                .insert(name.clone(), HashSet::new());
        }
        for (_, func) in self.name_func.iter().filter(|(_, f)| !f.is_extern) {
            func.calc_live_for_handle_call();
            AsmModule::analyse_inst_with_live_now(func, &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let callee_func_name = &inst.get_func_name().unwrap();
                let mut to_saved = live_now.clone();
                to_saved.retain(|reg| caller_used.get(callee_func_name).unwrap().contains(reg));
                self.caller_regs_to_saveds
                    .get_mut(callee_func_name)
                    .unwrap()
                    .extend(to_saved.iter());
            });
        }
    }
    ///精确分析callee regs to saved
    pub fn analyse_callee_regs_to_saved(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        let callee_used = self.build_callee_used();
        self.callee_regs_to_saveds.clear();
        for (name, _) in self.name_func.iter() {
            self.callee_regs_to_saveds
                .insert(name.clone(), HashSet::new());
        }
        for (_, func) in self.name_func.iter().filter(|(_, f)| !f.is_extern) {
            func.calc_live_for_handle_call();
            AsmModule::analyse_inst_with_live_now(func, &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let callee_func_name = &inst.get_func_name().unwrap();
                //刷新callee svaed
                if self.name_func.get(callee_func_name).unwrap().is_extern {
                    return;
                }
                let mut to_saved = live_now.clone();
                to_saved.retain(|reg| callee_used.get(callee_func_name).unwrap().contains(reg));
                self.callee_regs_to_saveds
                    .get_mut(callee_func_name)
                    .unwrap()
                    .extend(to_saved);
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
                        allocator.alloc_with_constraints(&main_func, &callee_constraints);
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
        new_callers_used
    }
}

impl AsmModule {
    ///函数分裂后减少使用到的特定物理寄存器
    /// 该函数调用应该在handle call之前,
    /// 该函数的调用结果依赖于调用该函数前进行的analyse for handle call
    /// handlespill->[split func]->reduce_ctsafs->handle call
    pub fn reduce_caller_to_saved_after_func_split(&mut self) {
        //对于 main函数中的情况专门处理, 对于 call前后使用 的 caller saved函数进行重分配,尝试进行recolor,(使用任意寄存器)
        let func = self.name_func.get("main").unwrap();
        // debug_assert!(func.as_ref().)
        let mut passed_i_p: HashSet<(ObjPtr<LIRInst>, Reg, bool)> = HashSet::new();
        //标记重整
        let mut inst_vreg_preg: Vec<(ObjPtr<LIRInst>, Reg, Reg, bool)> = Vec::new();
        let caller_used = AsmModule::build_caller_used(&self);
        let callee_used = AsmModule::build_callee_used(&self);
        let mut new_v_regs: HashSet<Reg> = HashSet::new();
        let mut constraints: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        func.calc_live_for_handle_call();
        debug_assert!(func.label == "main");
        debug_assert!(!func.draw_all_regs().contains(&Reg::get_s0()));

        //对于handle call这一步已经没有物理寄存器了,所有遇到的虚拟寄存器都是该处产生的
        //搜索并记录需要重分配列表
        AsmModule::analyse_inst_with_index_and_live_now(func, &mut |inst, index, live_now, bb| {
            //
            if inst.get_type() != InstrsType::Call {
                return;
            }
            let callee_func_name = inst.get_func_name().unwrap();
            let mut reg_cross = live_now.clone();
            reg_cross.retain(|reg| {
                reg.get_color() > 4
                    && reg != &Reg::get_s0()
                    && reg.is_physic()
                    // && reg.is_caller_save()
                    && !inst.get_regs().contains(reg)
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
                new_v_regs.insert(v_reg);
                constraints.insert(v_reg, HashSet::new());
                dinsts.iter().for_each(|(ist, if_replace_def)| {
                    passed_i_p.insert((*ist, *reg, *if_replace_def));
                    inst_vreg_preg.push((*ist, v_reg, *reg, *if_replace_def));
                });
            }
        });

        //p2v
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

        func.calc_live_for_handle_call(); //该处在handle spill之后，应该calc live for handle call

        //统计约束对次数 (vreg,p_reg)->times
        let mut constraints_times: HashMap<(Reg, Reg), usize> = HashMap::new();
        //统计对每个虚拟寄存器不同约束出现的次数,对于每个虚拟寄存器,减少约束的时候应该优先减少约束出现次数最少的
        AsmModule::analyse_inst_with_live_now(func, &mut |inst, live_now| {
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
                //约定原本的约束,原本存在的对物理寄存器的约束仍然要存在
                //此前使用了的但是函数中并没有保存到的寄存器被认为是 现在使用的,需要保存的,从而需要约束的
                let mut callee_used = callee_used.get(callee_func_name.as_str()).unwrap().clone();
                callee_used.retain(|reg| {
                    !self
                        .callee_regs_to_saveds
                        .get(callee_func_name.as_str())
                        .unwrap()
                        .contains(reg)
                });
                callee_used
            };
            let caller_used = caller_used.get(callee_func_name.as_str()).unwrap();
            live_now
                .iter()
                .filter(|reg| !reg.is_physic())
                .for_each(|reg| {
                    debug_assert!(new_v_regs.contains(reg));
                    let v_reg = *reg;
                    let mut p_regs = caller_used.clone();
                    p_regs.extend(callee_used.iter());
                    for p_reg in p_regs {
                        let key = (v_reg, p_reg);
                        let new_times = constraints_times.get(&key).unwrap_or(&0) + 1;
                        constraints_times.insert(key, new_times);
                    }
                    //加入约束
                    constraints.get_mut(reg).unwrap().extend(caller_used);
                    constraints.get_mut(reg).unwrap().extend(callee_used.iter());
                });
        }); //为新产生的虚拟寄存器建立约束

        let alloc_stat = || -> Option<FuncAllocStat> {
            //不断地减少约束,直到能够完美分配 (约束中关于callee的约束不能够减少,关于caller的约束可以减少)

            //首先统计可以减少的约束数量
            let mut keys: Vec<(Reg, Reg)> = constraints_times
                .iter()
                .map(|(k, _)| *k)
                .filter(|(v, p)| p.is_caller_save())
                .collect();
            keys.sort_by_key(|key| constraints_times.get(key).unwrap());
            let mut keys: LinkedList<(Reg, Reg)> = keys.iter().cloned().collect();
            loop {
                println!("{}", keys.len());
                let mut allocator = easy_gc_alloc::Allocator::new();
                let alloc_stat = allocator.alloc_with_constraints(func, &constraints);
                if alloc_stat.spillings.len() == 0 {
                    return Some(alloc_stat);
                }
                if keys.is_empty() {
                    break;
                }
                let to_de_constraint = keys.pop_front().unwrap();
                let (v_reg, p_reg) = to_de_constraint;
                let tmp = constraints.get_mut(&v_reg).unwrap();
                tmp.remove(&p_reg);
                if tmp.len() == 0 {
                    constraints.remove(&v_reg);
                }
            }
            //使用optgc2则如果理论上存在完全分配的话则一定能够分配出来
            None
        }();
        debug_assert!(alloc_stat.is_none() || alloc_stat.as_ref().unwrap().spillings.len() == 0);
        //使用新分配结果重新v2p
        if let Some(alloc_stat) = alloc_stat {
            AsmModule::iter_insts(*func, &mut |inst| {
                for reg in inst.get_regs() {
                    if reg.is_physic() {
                        continue;
                    }
                    let color = alloc_stat.dstr.get(&reg.get_id()).unwrap();
                    inst.as_mut().replace(reg.get_id(), *color);
                }
            });
            func.as_mut().reg_alloc_info = alloc_stat;
        } else {
            //否则原样恢复
            for (inst, v_reg, p_reg, if_replace_def) in inst_vreg_preg.iter() {
                log_file!(
                    "v2p_suf_reduce_caller.txt",
                    "{},{v_reg},{p_reg},{if_replace_def}",
                    inst.as_ref()
                );
                if *if_replace_def {
                    inst.as_mut().replace_only_def_reg(v_reg, p_reg);
                } else {
                    inst.as_mut().replace_only_use_reg(v_reg, p_reg);
                }
            }
        }
    }
}
