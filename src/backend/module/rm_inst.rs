use super::*;

impl AsmModule {
    pub fn rm_inst_before_rearrange(
        &mut self,
        pool: &mut BackendPool,
        used_but_not_saveds: &HashMap<String, HashSet<Reg>>,
    ) {
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.as_mut().remove_unuse_store();
            // while func.as_mut().remove_unuse_def() {
            //     func.as_mut().remove_unuse_store();
            // }
        }
    }

    pub fn rm_inst_suf_update_array_offset(
        &mut self,
        pool: &mut BackendPool,
        used_but_not_saveds: &HashMap<String, HashSet<Reg>>,
    ) {
        for (_, func) in self.name_func.iter().filter(|(_, f)| !f.is_extern) {
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
