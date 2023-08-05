use crate::backend::regalloc;

use super::*;

impl AsmModule {
    //进行最终的重分配
    pub fn final_realloc(&mut self) {
        // let used_but_not_saved=self.builduse
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
                // }
                // let (_, p2v_actions) = func.as_mut().p2v(Reg::get_all_recolorable_regs());
                // let callee_constraints = &self.build_constraints_with_callee_used(callees_used);
                // let caller_constraints=&self.build_constraints(callers_used, callees_used)
                // let regalloc::merge::merge__reg_with_constraints(&mut func, availables, constraints);
            }
        }
    }
}
