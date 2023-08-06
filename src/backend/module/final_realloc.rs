use crate::backend::regalloc;

use super::*;

impl AsmModule {
    //进行最终的重分配
    pub fn final_realloc(&mut self, pool: &mut BackendPool) {
        // let used_but_not_saved=self.builduse

        let callers_used = self.build_caller_used();
        let callees_used = self.build_callee_used();
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

            Func::print_func(*func, "00.txt");
            while regalloc::merge::merge_reg_with_constraints(
                func.as_mut(),
                &availables,
                &reg_used_but_not_saved,
            ) {
                Func::print_func(*func, "00.txt");
            }
        }
    }
}
