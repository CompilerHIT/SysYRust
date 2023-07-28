use super::*;

// realloc 实现 ,用于支持build v4
impl Func {
    // //进行贪心的寄存器分配
    // pub fn alloc_reg_with_priority(&mut self, ordered_regs: Vec<Reg>) {
    //     // 按照顺序使用ordered regs中的寄存器进行分配
    //     todo!()
    // }
    ///移除对特定的寄存器的使用,转为使用其他已经使用过的寄存器
    /// 如果移除成功返回true,移除失败返回false
    ///该函数只应该main以外的函数调用
    pub fn try_ban_certain_reg(
        &mut self,
        reg_to_ban: &Reg,
        caller_used: &HashMap<String, HashSet<Reg>>,
        callee_used: &HashMap<String, HashSet<Reg>>,
    ) -> bool {
        let ban_path = "ban_certain_reg.txt";
        debug_assert!(reg_to_ban.is_physic() && reg_to_ban != &Reg::get_sp());
        //首先把所有 regs_to_ban都替换成一个新虚拟寄存器
        let regs_to_ban: HashSet<Reg> = vec![*reg_to_ban].iter().cloned().collect();
        let new_v_regs = self.p2v_pre_handle_call(regs_to_ban);
        let mut callee_avialbled = self.draw_used_callees();
        let mut callers_aviabled = self.draw_used_callers();
        callee_avialbled.extend(callee_used.get(self.label.as_str()).unwrap());
        callers_aviabled.extend(caller_used.get(self.label.as_str()).unwrap());
        callee_avialbled.remove(reg_to_ban);
        callers_aviabled.remove(reg_to_ban);

        //对于产生的新虚拟寄存器进行分类
        let mut first_callee = HashSet::new(); //优先使用calleed saved 的一类寄存器
        self.calc_live_for_alloc_reg();
        let interference_graph = &regalloc::build_interference(self);
        let mut availables =
            regalloc::build_availables_with_interef_graph(self, interference_graph);
        //根据上下文给availables 增加新的规则,观察是否能够分配 (如果不能够分配，则ban 流程失败)
        for bb in self.blocks.iter() {
            let mut live_now: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                live_now.insert(*reg);
            });
            for inst in bb.insts.iter().rev() {
                for reg in inst.get_reg_def() {
                    live_now.remove(&reg);
                }
                //如果遇到call 指令,call指令前后的寄存器需要增加新的信息
                if inst.get_type() == InstrsType::Call {
                    let func = inst.get_func_name().unwrap();
                    let callee_used = callee_used.get(func.as_str()).unwrap();
                    let mut ban_list = RegUsedStat::new();
                    for other_callee in Reg::get_all_callees_saved().iter() {
                        if callee_used.contains(other_callee) {
                            continue;
                        }
                        ban_list.use_reg(other_callee.get_color());
                    }
                    for reg in live_now.iter() {
                        if new_v_regs.contains(reg) {
                            first_callee.insert(*reg);
                            availables.get_mut(reg).unwrap().merge(&ban_list);
                        }
                    }
                }
                for reg in inst.get_reg_use() {
                    live_now.insert(reg);
                }
            }
        }

        //最后对avilable 进行一次修改
        for reg in new_v_regs.iter() {
            availables
                .get_mut(reg)
                .unwrap()
                .use_reg(reg_to_ban.get_color());
            // 对于不在 available 列表内的颜色,进行排除
            for un_available in Reg::get_all_recolorable_regs() {
                if !callee_avialbled.contains(&un_available)
                    && !callers_aviabled.contains(&un_available)
                {
                    availables
                        .get_mut(reg)
                        .unwrap()
                        .use_reg(un_available.get_color());
                }
            }
        }
        //开始着色,着色失败则回退最初颜色
        let mut colors: HashMap<Reg, i32> = HashMap::new();
        let mut to_color: Vec<Reg> = Vec::new();
        for v_reg in new_v_regs.iter() {
            to_color.push(*v_reg);
        }
        loop {
            if to_color.len() == 0 {
                break;
            }
            debug_assert!(to_color.len() != 0);
            //初始化 to color
            to_color.sort_by_key(|reg| {
                availables
                    .get(reg)
                    .unwrap()
                    .num_available_regs(reg.get_type())
            });
            //对to color排序,只着色可用颜色最多的一个
            let reg = to_color.remove(to_color.len() - 1);
            let mut color: Option<i32> = None;
            let available = availables.get(&reg).unwrap();
            if first_callee.contains(&reg) {
                for callee_reg in callee_avialbled.iter() {
                    if color.is_some() {
                        break;
                    }
                    if callee_reg.get_type() != reg.get_type() {
                        continue;
                    }
                    if available.is_available_reg(callee_reg.get_color()) {
                        color = Some(callee_reg.get_color());
                    }
                }
                for caller_reg in callers_aviabled.iter() {
                    if color.is_some() {
                        break;
                    }
                    if caller_reg.get_type() != reg.get_type() {
                        continue;
                    }
                    if available.is_available_reg(caller_reg.get_color()) {
                        color = Some(caller_reg.get_color());
                    }
                }
            } else {
                for caller_reg in callers_aviabled.iter() {
                    if color.is_some() {
                        break;
                    }
                    if caller_reg.get_type() != reg.get_type() {
                        continue;
                    }
                    if available.is_available_reg(caller_reg.get_color()) {
                        color = Some(caller_reg.get_color());
                    }
                }
                for callee_reg in callee_avialbled.iter() {
                    if color.is_some() {
                        break;
                    }
                    if callee_reg.get_type() != reg.get_type() {
                        continue;
                    }
                    if available.is_available_reg(callee_reg.get_color()) {
                        color = Some(callee_reg.get_color());
                    }
                }
            }
            //着色
            if color.is_none() {
                to_color.push(reg); //着色失败的寄存器加回去
                break;
            }
            colors.insert(reg, color.unwrap());
            //根据冲突图,更新其他寄存器与之的影响
            let neighbors = interference_graph.get(&reg).unwrap();
            for neighbor in neighbors.iter() {
                availables
                    .get_mut(neighbor)
                    .unwrap()
                    .use_reg(color.unwrap());
            }
        }
        if to_color.len() != 0 {
            log_file!(ban_path, "fail");
            //ban 失败,恢复原本颜色
            for bb in self.blocks.iter() {
                for inst in bb.insts.iter() {
                    for reg in inst.get_reg_def() {
                        if new_v_regs.contains(&reg) {
                            inst.as_mut().replace_only_def_reg(&reg, reg_to_ban);
                        }
                    }
                    for reg in inst.get_reg_use() {
                        if new_v_regs.contains(&reg) {
                            inst.as_mut().replace_only_use_reg(&reg, reg_to_ban);
                        }
                    }
                }
            }
            false
        } else {
            log_file!(ban_path, "success");
            //ban 成功,写入颜色
            for bb in self.blocks.iter() {
                for inst in bb.insts.iter() {
                    for reg in inst.get_reg_def() {
                        if new_v_regs.contains(&reg) {
                            let new_reg = Reg::from_color(*colors.get(&reg).unwrap());
                            inst.as_mut().replace_only_def_reg(&reg, &new_reg);
                        }
                    }
                    for reg in inst.get_reg_use() {
                        if new_v_regs.contains(&reg) {
                            let new_reg = Reg::from_color(*colors.get(&reg).unwrap());
                            inst.as_mut().replace_only_use_reg(&reg, &new_reg);
                        }
                    }
                }
            }
            true
        }
    }
}
