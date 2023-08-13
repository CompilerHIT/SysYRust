use crate::backend::regalloc::perfect_alloc;

use super::*;

impl AsmModule {
    pub fn count_callee_saveds_times(&mut self) -> HashMap<String, HashMap<Reg, usize>> {
        let mut callee_saved_times: HashMap<String, HashMap<Reg, usize>> = HashMap::new();
        let callee_used = self.build_callee_used();
        for (_, func_ptr) in self.name_func.iter() {
            if func_ptr.is_extern {
                continue;
            }
            func_ptr.calc_live_for_handle_call();
            debug_assert!(!func_ptr.draw_all_regs().contains(&Reg::get_s0()));
            AsmModule::analyse_inst_with_live_now(func_ptr.as_ref(), &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let mut live_now = live_now.clone();
                if let Some(def_reg) = inst.get_def_reg() {
                    live_now.remove(&def_reg);
                }
                //对于要保存的寄存器
                let func_called = inst.get_func_name();
                let func_called = func_called.as_ref().unwrap();
                let callee_used = callee_used.get(func_called).unwrap();
                live_now.retain(|reg| callee_used.contains(reg));
                if !callee_saved_times.contains_key(func_called) {
                    callee_saved_times.insert(func_called.clone(), HashMap::new());
                }
                let callee_saved_times = callee_saved_times.get_mut(func_called).unwrap();
                //统计次数
                for to_save in live_now.iter() {
                    let new_times = callee_saved_times.get(to_save).unwrap_or(&0) + 1;
                    callee_saved_times.insert(*to_save, new_times);
                }
            });
        }
        callee_saved_times
    }

    pub fn realloc_pre_spill(&mut self) {
        self.realloc_main_with_priority_pre_spill();
        self.realloc_not_main_with_priority();
    }

    ///使用进行函数分析后的结果先进行寄存器组成重构
    fn realloc_not_main_with_priority(&mut self) {
        //记录除了main函数外每个函数使用到的 callee saved和caller saved 需要的恢复次数
        let callee_saved_times: HashMap<String, HashMap<Reg, usize>> =
            self.count_callee_saveds_times();
        let call_map = &self.call_map;
        //对每个函数进行试图减少指定寄存器的使用
        for (func, func_ptr) in self.name_func.iter() {
            if func_ptr.is_extern {
                continue;
            }
            if func == "main" {
                continue;
            }
            let func_ptr = *func_ptr;
            //按照每个函数使用被调用时需要保存的自身使用到的callee saved寄存器的数量
            let callee_saved_time = callee_saved_times.get(func.as_str());
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
            let self_used = func_ptr.draw_used_callees();
            //自身调用的函数使用到的callee saved寄存器
            let mut callee_func_used: HashSet<Reg> = HashSet::new();
            for func_called in call_map.get(func_ptr.label.as_str()).unwrap() {
                if func_called == func_ptr.label.as_str() {
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
                func_ptr.calc_live_for_handle_call();
                let ok = func_ptr
                    .as_mut()
                    .try_ban_certain_reg(reg, &caller_used, &callee_used);
                if ok {
                    log_file!("ban_reg.txt", "{}", reg);
                    baned.insert(*reg);
                } else {
                    // break;
                }
            }
            for reg in callees.iter().rev() {
                if !self_used.contains(reg) {
                    continue;
                }
                if callee_func_used.contains(reg) {
                    continue;
                }
                let ok = func_ptr
                    .as_mut()
                    .try_ban_certain_reg(reg, &caller_used, &callee_used);
                if ok {
                    log_file!("ban_reg.txt", "{}", reg);
                    baned.insert(*reg);
                } else {
                    // break;
                }
            }
        }
        // // return;
    }

    //统计每个虚拟寄存器收到不同的物理寄存器的约束个数,根据个数从多到少进行减少

    ///重新调整main函数的寄存器分布以减少被调用函数需要保存的寄存器
    fn realloc_main_with_priority_pre_spill(&mut self) {
        let main_func = *self.name_func.get("main").unwrap();
        let mut rs = Reg::get_all_recolorable_regs();
        rs.remove(&Reg::get_s0());
        main_func.as_mut().p2v_pre_handle_call(&rs);
        main_func.as_mut().calc_live_for_alloc_reg();

        main_func.as_mut().allocate_reg();
        let callees_used = self.build_callee_used();
        let callee_constraints: HashMap<Reg, HashSet<Reg>> =
            self.build_constraints_with_callee_used(&callees_used);
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
                    let alloc_stat =
                        perfect_alloc::alloc_with_constraints(&main_func, &callee_constraints);
                    //每次减半直到分配成功
                    if alloc_stat.is_some() {
                        debug_assert!(alloc_stat.as_ref().unwrap().spillings.len() == 0);
                        return alloc_stat;
                    }
                    //否则分配失败,减少约束再分配
                    //首先减少约束最多的寄存器的约束
                    let ord: HashSet<Reg> =
                        callee_constraints.iter().map(|(reg, _)| *reg).collect();
                    for reg in ord.iter() {
                        let to_rms: Vec<Reg> = callee_constraints
                            .get(reg)
                            .unwrap()
                            .iter()
                            .cloned()
                            .collect();
                        for to_rm in to_rms {
                            callee_constraints.get_mut(reg).unwrap().remove(&to_rm);
                            break;
                        }
                        if callee_constraints.get(reg).unwrap().len() == 0 {
                            callee_constraints.remove(reg);
                            break;
                        }
                        break;
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
}

impl AsmModule {
    ///函数分裂后减少使用到的特定物理寄存器
    /// 该函数调用应该在handle call之前,
    /// 该函数的调用结果依赖于调用该函数前进行的analyse for handle call
    /// handlespill->[split func]->reduce_ctsafs->handle call
    pub fn reduce_caller_to_saved_after_func_split(&mut self) {
        //TODO,主要针对main函数重构reduce caller saved
    }
}
