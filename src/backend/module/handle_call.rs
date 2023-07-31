use super::*;

impl AsmModule {
    pub fn handle_call_v4(&mut self, pool: &mut BackendPool) {
        let callees_used = self.build_callee_used();
        let callers_used = self.build_caller_used();
        let callees_be_saved = &self.callee_regs_to_saveds;
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.as_mut()
                .handle_call_v4(pool, &callers_used, &callees_used, &callees_be_saved);
        }
    }
}
