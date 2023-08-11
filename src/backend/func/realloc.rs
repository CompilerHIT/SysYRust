use crate::backend::regalloc::perfect_alloc;

use super::*;

// realloc 实现 ,用于支持build v4
impl Func {
    // //进行贪心的寄存器重分配
    pub fn alloc_reg_with_priority(&mut self, ordered_regs: Vec<Reg>) {
        // 按照顺序使用ordered regs中的寄存器进行分配
        self.calc_live_for_handle_call();

        debug_assert!(self.draw_all_virtual_regs().len() == 0);
        let mut to_decolor = Reg::get_all_recolorable_regs();
        to_decolor.remove(&Reg::get_s0());
        // Func::print_func(
        //     ObjPtr::new(&self),
        //     "before_realloc_with_priority_before_p2v.txt",
        // );
        let (all_new_v_regs, p2v_actions) = self.p2v(&to_decolor);
        // Func::print_func(
        //     ObjPtr::new(&self),
        //     "before_realloc_with_priority_after_p2v.txt",
        // );
        self.calc_live_for_handle_call();
        //不能上二分，为了最好效果,使用最少的寄存器
        //所以直接地,
        let all_v_regs = self.draw_all_virtual_regs();
        debug_assert!(all_new_v_regs.len() >= all_v_regs.len());
        debug_assert!(self
            .draw_phisic_regs()
            .is_available_reg(Reg::get_s0().get_color()));

        let all_regs = Reg::get_all_regs();
        let mut last_alloc_stat: Option<FuncAllocStat> = None;
        let mut availables: HashSet<Reg> = ordered_regs.iter().cloned().collect();
        for reg in ordered_regs.iter() {
            availables.remove(reg);
            let mut unavailables = all_regs.clone();
            unavailables.retain(|reg| !availables.contains(reg));
            let mut constraints = HashMap::new();
            for v_reg in all_v_regs.iter() {
                constraints.insert(*v_reg, unavailables.clone());
            }
            let alloc_stat = perfect_alloc::alloc(&self, &constraints);
            if alloc_stat.is_some() {
                last_alloc_stat = alloc_stat;
                continue;
            } else {
                break;
            }
        }

        if last_alloc_stat.is_none() {
            //如果重分配失败,  恢复原样
            for (inst, p_reg, v_reg, if_def) in p2v_actions {
                if if_def {
                    inst.as_mut().replace_only_def_reg(&v_reg, &p_reg);
                } else {
                    inst.as_mut().replace_only_use_reg(&v_reg, &p_reg);
                }
            }
            return;
        }
        let alloc_stat = last_alloc_stat.unwrap();
        debug_assert!(alloc_stat.spillings.len() == 0);
        self.v2p(&alloc_stat.dstr);
    }

    ///移除对特定的寄存器的使用,转为使用其他已经使用过的寄存器
    /// 如果移除成功返回true,移除失败返回false
    ///该函数只应该main以外的函数调用
    /// 该函数内部会调用calc live for call
    pub fn try_ban_certain_reg(
        &mut self,
        reg_to_ban: &Reg,
        caller_used: &HashMap<String, HashSet<Reg>>,
        callee_used: &HashMap<String, HashSet<Reg>>,
    ) -> bool {
        //根据caller used和caller used,消去该函数中指定寄存器的使用
        //如果当前寄存器调用的函数中有人使用了该寄存器,则无法消去
        //通过禁止使用reg_to_ban的方式,优先使用自身callee_used的寄存器,然后再使用自身caller used的寄存器
        let self_name = self.label.clone();
        let mut self_used = caller_used.get(&self_name).unwrap().clone();
        self_used.extend(callee_used.get(self_name.as_str()).unwrap().iter());
        //self used中是自身已经使用了的寄存器
        //然后p2v自身调用过的寄存器
        let mut to_decolor = HashSet::new();
        to_decolor.insert(*reg_to_ban);

        self.calc_live_for_handle_call();
        let (new_v_regs, p2v_actions) = self.p2v(&to_decolor);

        if !self
            .draw_phisic_regs()
            .is_available_reg(reg_to_ban.get_color())
        {
            Func::undo_p2v(&p2v_actions);
            return false;
        }

        //p2v成功后使用self_used尝试分配
        self_used.remove(reg_to_ban);
        let mut constraints = HashMap::new();
        let mut unavailables = Reg::get_all_regs();
        unavailables.retain(|reg| !self_used.contains(reg));
        for new_v_reg in new_v_regs {
            constraints.insert(new_v_reg, unavailables.clone());
        }

        self.calc_live_for_handle_call();
        if let Some(alloc_stat) = perfect_alloc::alloc(&self, &constraints) {
            self.v2p(&alloc_stat.dstr);
            return true;
        } else {
            Func::undo_p2v(&p2v_actions);
        }
        false
    }
}

//对 final realloc的支持
impl Func {
    pub fn replace_v_reg(&mut self, old_reg: &Reg, new_reg: &Reg) {
        debug_assert!(!old_reg.is_physic() && !new_reg.is_physic());
        self.blocks.iter().for_each(|bb| {
            bb.insts.iter().for_each(|inst| {
                inst.as_mut().replace_reg(old_reg, new_reg);
            })
        })
    }
}
