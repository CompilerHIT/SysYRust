use super::*;

impl AsmModule {
    ///build constraints with caller_used
    pub fn build_constraints_with_caller_used(
        callers_used: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        // AsmModule::analyse_inst_with_live_now(&main_func, &mut |inst, live_now| {
        //     if inst.get_type() != InstrsType::Call {
        //         return;
        //     }
        //     //对于call指令来说,不需要保存和恢复在call指令的时候定义的寄存器
        //     let mut live_now = live_now.clone();
        //     if let Some(def_reg) = inst.get_def_reg() {
        //         live_now.remove(&def_reg);
        //     }
        //     let live_now = live_now;

        //     //对于 call指令,分析上下文造成的依赖关系
        //     let func_name = inst.get_func_name().unwrap();
        //     let func = self.name_func.get(func_name.as_str()).unwrap();
        //     if func.is_extern {
        //         //遇到 is_extern的情况,不能节省,也不应节省
        //         return;
        //     } else {
        //         let callee_used = callee_used.get(func.label.as_str()).unwrap();
        //         for reg in live_now.iter() {
        //             if reg.is_physic() {
        //                 continue;
        //             }

        //             if !callee_constraints.contains_key(reg) {
        //                 callee_constraints.insert(*reg, callee_used.clone());
        //             } else {
        //                 callee_constraints.get_mut(reg).unwrap().extend(callee_used);
        //             }
        //         }
        //     }
        // });
        todo!()
    }

    ///build constraints with callee_used
    pub fn build_constraints_with_callee_used(
        callees_used: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        todo!()
    }
    ///build constraints with all used
    pub fn build_constraints(
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_used: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        todo!()
    }

    ///build contraints with used_but_not_saved
    pub fn build_constraints_with_used_but_not_saved(
        used_but_not_saved: &HashMap<String, HashSet<Reg>>,
    ) -> HashMap<Reg, HashSet<Reg>> {
        todo!()
    }
}
