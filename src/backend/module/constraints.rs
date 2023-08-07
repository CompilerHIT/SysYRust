use super::*;

impl AsmModule {
    ///build constraints with caller_used
    pub fn build_constraints_with_caller_used(
        &mut self,
        callers_used: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        //统计所有寄存器相关的冲突情况(包括物理寄存器)
        let mut constraints = HashMap::new();
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            AsmModule::analyse_inst_with_live_now(func.as_ref(), &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let func_name = inst.get_func_name().unwrap();
                let func_name = &func_name;
                let mut live_now = live_now.clone();
                if let Some(def_reg) = inst.get_def_reg() {
                    live_now.remove(&def_reg);
                }
                let callers_used = callers_used.get(func_name).unwrap();
                let mut callers_need_to_saved = live_now.clone();
                callers_need_to_saved.retain(|reg| callers_used.contains(reg));
                for live_reg in live_now {
                    if live_reg.is_physic() {
                        continue;
                    }
                    if !constraints.contains_key(&live_reg) {
                        constraints.insert(live_reg, HashSet::new());
                    }
                    constraints
                        .get_mut(&live_reg)
                        .unwrap()
                        .extend(callers_need_to_saved.iter());
                }
            });
        }
        constraints
    }

    ///build constraints with callee_used
    pub fn build_constraints_with_callee_used(
        &mut self,
        callees_used: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        let mut constraints = HashMap::new();
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            AsmModule::analyse_inst_with_live_now(func.as_ref(), &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let func_name = inst.get_func_name().unwrap();
                let func_name = &func_name;
                let mut live_now = live_now.clone();
                if let Some(def_reg) = inst.get_def_reg() {
                    live_now.remove(&def_reg);
                }
                let callees_used = callees_used.get(func_name).unwrap();
                let mut callees_need_to_saved = live_now.clone();
                callees_need_to_saved.retain(|reg| callees_used.contains(reg));
                for live_reg in live_now {
                    if live_reg.is_physic() {
                        continue;
                    }
                    if !constraints.contains_key(&live_reg) {
                        constraints.insert(live_reg, HashSet::new());
                    }
                    constraints
                        .get_mut(&live_reg)
                        .unwrap()
                        .extend(callees_need_to_saved.iter());
                }
            });
        }
        constraints
    }
    ///build constraints with all used
    pub fn build_constraints(
        &mut self,
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_used: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        let callees_constraints = self.build_constraints_with_callee_used(callees_used);
        let callers_constraints = self.build_constraints_with_caller_used(callers_used);
        let mut constraints = callees_constraints;
        for (reg, constraint) in callers_constraints {
            if !constraints.contains_key(&reg) {
                constraints.insert(reg, constraint);
            } else {
                constraints.get_mut(&reg).unwrap().extend(constraint.iter());
            }
        }
        constraints
    }

    ///build contraints with used_but_not_saved
    pub fn build_constraints_with_used_but_not_saved(
        &mut self,
        used_but_not_saved: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        let mut constraints = HashMap::new();
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            AsmModule::analyse_inst_with_live_now(func.as_ref(), &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let func_name = inst.get_func_name().unwrap();
                let func_name = &func_name;
                let mut live_now = live_now.clone();
                if let Some(def_reg) = inst.get_def_reg() {
                    live_now.remove(&def_reg);
                }
                let mut regs_need_saved = used_but_not_saved.get(func_name).unwrap().clone();
                regs_need_saved.retain(|reg| live_now.contains(reg));
                let live_regs_need_saved = regs_need_saved;
                for live_reg in live_now {
                    if live_reg.is_physic() {
                        continue;
                    }
                    if !constraints.contains_key(&live_reg) {
                        constraints.insert(live_reg, HashSet::new());
                    }
                    constraints
                        .get_mut(&live_reg)
                        .unwrap()
                        .extend(live_regs_need_saved.iter());
                }
            });
        }
        constraints
    }
}
