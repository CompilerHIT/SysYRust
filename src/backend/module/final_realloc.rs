use crate::backend::regalloc;

use super::*;

impl AsmModule {
    //进行最终的重分配
    pub fn final_realloc(&mut self, pool: &mut BackendPool) {
        // let used_but_not_saved=self.builduse

        let callers_used = self.build_caller_used();
        let callees_used = self.build_callee_used();
        self.analyse_callee_regs_to_saved_for_final_realloc();
        let callees_saved = &self.callee_regs_to_saveds;
        let reg_used_but_not_saved =
            AsmModule::build_used_but_not_saveds(&callers_used, &callees_used, callees_saved);

        for (name, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            let callers_used = callers_used.get(name).unwrap();
            let mut used = callers_used.clone();
            used.extend(callees_used.get(name).unwrap().iter());
            let availables = used;

            //before alloc

            // Func::print_func(*func, "00.txt");
            while regalloc::merge::merge_reg_with_constraints(
                func.as_mut(),
                &availables,
                &reg_used_but_not_saved,
            ) {
                // Func::print_func(*func, "00.txt");
            }
        }
    }

    fn analyse_callee_regs_to_saved_for_final_realloc(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        let callee_used = self.build_callee_used();
        self.callee_regs_to_saveds.clear();
        for (name, _) in self.name_func.iter() {
            self.callee_regs_to_saveds
                .insert(name.clone(), HashSet::new());
        }
        for (_, func) in self.name_func.iter().filter(|(_, f)| !f.is_extern) {
            func.calc_live_base();
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
}
