use core::time;

use crate::backend::regalloc;

use super::*;

impl AsmModule {
    //进行最终的重分配
    pub fn final_realloc(&mut self, pool: &mut BackendPool) {
        // let used_but_not_saved=self.builduse
        let callers_used = self.build_caller_used();
        let callees_used = self.build_callee_used();
        let callees_saved = &self.callee_regs_to_saveds;

        let mut reg_used_but_not_saved =
            AsmModule::build_used_but_not_saveds(&callers_used, &callees_used, callees_saved);
        //禁止在函数调用前后使用s0
        for (_, used_but_not_saved) in reg_used_but_not_saved.iter_mut() {
            used_but_not_saved.insert(Reg::get_s0());
        }
        for (func, used_but_not_saved) in reg_used_but_not_saved.iter() {
            log_file!("final_realloc_actions.txt", "func:{}", func);
            for reg in used_but_not_saved.iter() {
                log_file!("final_realloc_actions.txt", "{}", reg);
            }
        }

        let reg_used_but_not_saved = reg_used_but_not_saved;
        let mut to_realloc: Vec<ObjPtr<Func>> = self.name_func.iter().map(|(_, f)| *f).collect();
        to_realloc.retain(|f| !f.is_extern);

        for func in to_realloc.iter() {
            let name = &func.label;
            let callers_used = callers_used.get(name).unwrap().clone();
            let callees_used = callees_used.get(name).unwrap().clone();
            let mut used = callers_used.clone();
            used.extend(callees_used);
            used.insert(Reg::get_s0());
            let availables = used;
            func.as_mut().remove_unuse_def();
            func.as_mut().remove_self_mv();
            while regalloc::merge::merge_reg_with_constraints(
                func.as_mut(),
                &availables,
                &reg_used_but_not_saved,
            ) {}
        }
    }
}
