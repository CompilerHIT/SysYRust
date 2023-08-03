use super::*;

impl AsmModule {
    pub fn handle_call_v4(
        &mut self,
        pool: &mut BackendPool,
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_used: &HashMap<String, HashSet<Reg>>,
        callees_saved: &HashMap<String, HashSet<Reg>>,
    ) {
        let callees_be_saved = callees_saved;
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.as_mut()
                .handle_call_v4(pool, &callers_used, &callees_used, &callees_be_saved);
        }
    }
}
