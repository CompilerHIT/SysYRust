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

    pub fn realloc_pre_split_func(&mut self) {
        self.realloc_main_with_priority_pre_split();
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
    fn realloc_main_with_priority_pre_split(&mut self) {
        let main_func = *self.name_func.get("main").unwrap();
        let mut rs = Reg::get_all_recolorable_regs();
        rs.remove(&Reg::get_s0());
        main_func.as_mut().p2v_pre_handle_call(rs);
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
                    let alloc_stat = perfect_alloc::alloc(&main_func, &callee_constraints);
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
            main_func.calc_live_for_handle_call();
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
            //对于call指令来说,不需要保存和恢复在call指令的时候定义的寄存器
            let mut live_now = live_now.clone();
            if let Some(def_reg) = inst.get_def_reg() {
                live_now.remove(&def_reg);
            }
            let live_now = live_now;

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
                .filter(|(_, p)| p.is_caller_save())
                .collect();
            keys.sort_by_key(|key| constraints_times.get(key).unwrap());
            let mut keys: LinkedList<(Reg, Reg)> = keys.iter().cloned().collect();
            loop {
                let alloc_stat = perfect_alloc::alloc(func, &constraints);
                if alloc_stat.is_some() {
                    return alloc_stat;
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
