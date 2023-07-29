use crate::backend::regalloc::perfect_alloc;

use super::*;

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
            let callee_save = self
                .callee_regs_to_saveds
                .get(callee_func_name.as_str())
                .unwrap();
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

impl AsmModule {
    pub fn handle_call_v4(&mut self) {
        unimplemented!()
    }
}
