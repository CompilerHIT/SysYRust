use super::*;

impl AsmModule {
    pub fn handle_call(
        &mut self,
        pool: &mut BackendPool,
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_used: &HashMap<String, HashSet<Reg>>,
        callees_be_saved: &HashMap<String, HashSet<Reg>>,
    ) {
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.as_mut()
                .handle_call(pool, &callers_used, &callees_used, &callees_be_saved);
        }
    }

    pub fn handle_call_tmp(&mut self, pool: &mut BackendPool) {
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.as_mut().handle_call_tmp(pool);
        }
    }
}
