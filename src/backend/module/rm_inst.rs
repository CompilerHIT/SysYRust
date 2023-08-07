use super::*;

impl AsmModule {
    pub fn rm_inst_suf_handle_call(
        &mut self,
        pool: &mut BackendPool,
        used_but_not_saveds: &HashMap<String, HashSet<Reg>>,
    ) {
        for (_, func) in self.name_func.iter() {
            func.as_mut()
                .remove_unuse_inst_suf_handle_call(pool, &used_but_not_saveds);
        }
    }

    pub fn rm_inst_suf_update_array_offset(
        &mut self,
        pool: &mut BackendPool,
        used_but_not_saveds: &HashMap<String, HashSet<Reg>>,
    ) {
        for (_, func) in self.name_func.iter() {
            func.as_mut()
                .rm_inst_suf_update_array_offset(pool, &used_but_not_saveds);
        }
    }

    pub fn build_used_but_not_saveds(
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_used: &HashMap<String, HashSet<Reg>>,
        callees_saved: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<String, HashSet<Reg>> {
        let mut used_but_not_saved: HashMap<String, HashSet<Reg>> = HashMap::new();
        for (func, callers_used) in callers_used.iter() {
            let mut used: HashSet<Reg> = callers_used.clone();
            used.extend(callees_used.get(func).unwrap().iter());
            used.retain(|reg| !callees_saved.get(func).unwrap().contains(reg));
            used_but_not_saved.insert(func.clone(), used);
        }
        used_but_not_saved
    }
}
