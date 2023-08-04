use crate::{
    backend::{func, opt, regalloc::perfect_alloc},
    ir::CallMap,
};

use super::*;

/// build v4:
/// 1.寄存器重分配:针对call上下文调整函数寄存器组成
/// 2.针对函数是否为main调整寄存器组成
impl AsmModule {
    ///建立函数间的直接调用表
    pub fn build_own_call_map(&mut self) {
        let mut call_map = HashMap::new();

        //首先建立直接函数调用表
        for (name, func) in self.name_func.iter() {
            let mut callee_funcs: HashSet<String> = HashSet::new();
            if func.is_extern {
                call_map.insert(name.clone(), callee_funcs);
                continue;
            }
            for bb in func.blocks.iter() {
                bb.insts
                    .iter()
                    .filter(|inst| inst.get_type() == InstrsType::Call)
                    .for_each(|inst| {
                        let func_name = inst.get_func_name().unwrap();
                        callee_funcs.insert(func_name.clone());
                        // calls.push((name.clone(), func_name));
                    });
            }
            call_map.insert(name.clone(), callee_funcs);
        }

        self.call_map = call_map;
        //然后建立函数调用族
        let func_group = AsmModule::build_func_groups(&self.call_map);
        self.func_groups = func_group;
    }

    pub fn p2v(&mut self) {
        self.name_func
            .iter()
            .filter(|(_, f)| !f.is_extern)
            .for_each(|(_, f)| {
                f.as_mut()
                    .p2v_pre_handle_call(Reg::get_all_recolorable_regs());
            });
    }

    ///v4的analyse for handle call 依赖于前文调用build call map构建的call map
    pub fn anaylyse_for_handle_call_v4(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        self.analyse_callee_regs_to_saved();
        self.analyse_caller_regs_to_saved();
    }

    ///精确分析caller regs to saved
    pub fn analyse_caller_regs_to_saved(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        let caller_used = self.build_caller_used();
        self.caller_regs_to_saveds.clear();
        for (name, _) in self.name_func.iter() {
            self.caller_regs_to_saveds
                .insert(name.clone(), HashSet::new());
        }
        for (_, func) in self.name_func.iter().filter(|(_, f)| !f.is_extern) {
            func.calc_live_for_handle_call();
            AsmModule::analyse_inst_with_live_now(func, &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                //对于call指令来说,不需要保存和恢复在call指令的时候定义的寄存器
                let mut live_now = live_now.clone();
                if let Some(def_reg) = inst.get_def_reg() {
                    live_now.remove(&def_reg);
                }
                let live_now = live_now;

                let callee_func_name = &inst.get_func_name().unwrap();
                let mut to_saved = live_now.clone();
                to_saved.retain(|reg| caller_used.get(callee_func_name).unwrap().contains(reg));
                self.caller_regs_to_saveds
                    .get_mut(callee_func_name)
                    .unwrap()
                    .extend(to_saved.iter());
            });
        }
    }
    ///精确分析callee regs to saved
    pub fn analyse_callee_regs_to_saved(&mut self) {
        //对于name func里面的东西,根据上下文准备对应内容
        let callee_used = self.build_callee_used();
        self.callee_regs_to_saveds.clear();
        for (name, _) in self.name_func.iter() {
            self.callee_regs_to_saveds
                .insert(name.clone(), HashSet::new());
        }
        for (_, func) in self.name_func.iter().filter(|(_, f)| !f.is_extern) {
            func.calc_live_for_handle_call();
            AsmModule::analyse_inst_with_live_now(func, &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                //对于call指令来说,不需要保存和恢复在call指令的时候定义的寄存器
                let mut live_now = live_now.clone();
                if let Some(def_reg) = inst.get_def_reg() {
                    live_now.remove(&def_reg);
                }
                let live_now = live_now;

                let callee_func_name = &inst.get_func_name().unwrap();
                //刷新callee svaed
                if self.name_func.get(callee_func_name).unwrap().is_extern {
                    return;
                }
                let mut to_saved = live_now.clone();
                to_saved.retain(|reg| callee_used.get(callee_func_name).unwrap().contains(reg));
                self.callee_regs_to_saveds
                    .get_mut(callee_func_name)
                    .unwrap()
                    .extend(to_saved);
            });
        }
    }

    ///最后得到的表中不会包含sp
    pub fn build_callee_used(&self) -> HashMap<String, HashSet<Reg>> {
        let mut calleed_useds = HashMap::new();
        for (_, func) in self.name_func.iter() {
            let mut callees_used = self.draw_callee_used(*func);
            callees_used.remove(&Reg::get_sp());
            calleed_useds.insert(func.label.clone(), callees_used);
        }
        calleed_useds
    }

    //最后得到的表中不会包含ra
    pub fn build_caller_used(&self) -> HashMap<String, HashSet<Reg>> {
        let mut caller_useds = HashMap::new();
        for (_, func) in self.name_func.iter() {
            let mut callers_used = self.draw_caller_used(*func);
            callers_used.remove(&Reg::get_ra());
            caller_useds.insert(func.label.clone(), callers_used);
        }
        caller_useds
    }

    ///重新分析出一个函数递归地影响到的callee saved的寄存器的组成
    /// 它只会统计该函数用到的callee saved以及它调用的非外部函数用到的callee saved寄存器
    pub fn draw_callee_used(&self, func: ObjPtr<Func>) -> HashSet<Reg> {
        if func.is_extern {
            return HashSet::new();
        }
        let mut new_callee_uesd: HashSet<Reg> = func.draw_used_callees();
        // 首先递归地找到这个函数内部调用过地所有函数集合
        let mut callee_funcs: HashSet<ObjPtr<Func>> = HashSet::new();
        // let call_map = AsmModule::build_call_map(name_func);
        for func in self.call_map.get(func.label.as_str()).unwrap() {
            let func = self.name_func.get(func).unwrap();
            if func.is_extern {
                continue;
            }
            callee_funcs.insert(*func);
        }
        //处理多重递归调用的情况
        loop {
            let mut break_flag = true;
            let mut callee_to_add = HashSet::new();
            for func in callee_funcs.iter() {
                let func = *func;
                for func in self.call_map.get(func.label.as_str()).unwrap() {
                    let func = *self.name_func.get(func).unwrap();
                    if func.is_extern {
                        continue;
                    }
                    if !callee_funcs.contains(&func) && !callee_to_add.contains(&func) {
                        callee_to_add.insert(func);
                        break_flag = false;
                    }
                }
            }
            callee_funcs.extend(callee_to_add);
            if break_flag {
                break;
            }
        }
        for func_called in callee_funcs.iter() {
            debug_assert!(!func_called.is_extern);
            let callee_used = func_called.draw_used_callees();
            new_callee_uesd.extend(callee_used);
        }
        new_callee_uesd
    }

    ///递归分析一个函数调用影响到的caller saved寄存器=
    pub fn draw_caller_used(&self, func: ObjPtr<Func>) -> HashSet<Reg> {
        let mut new_callers_used: HashSet<Reg> = func.draw_used_callers();
        // 首先递归地找到这个函数内部调用过地所有函数集合
        let mut callee_funcs: HashSet<ObjPtr<Func>> = HashSet::new();
        if func.is_extern {
            return Reg::get_all_callers_saved();
        }

        for func in self.call_map.get(func.label.as_str()).unwrap() {
            let func = self.name_func.get(func).unwrap();
            callee_funcs.insert(*func);
            if func.is_extern {
                return Reg::get_all_callers_saved();
            }
        }
        //处理多重递归调用的情况
        loop {
            let mut break_flag = true;
            let mut callee_to_add = HashSet::new();
            for func in callee_funcs.iter() {
                let func = *func;
                for func in self.call_map.get(func.label.as_str()).unwrap() {
                    let func = *self.name_func.get(func).unwrap();
                    if func.is_extern {
                        return Reg::get_all_callers_saved();
                    }
                    if !callee_funcs.contains(&func) && !callee_to_add.contains(&func) {
                        callee_to_add.insert(func);
                        break_flag = false;
                    }
                }
            }
            callee_funcs.extend(callee_to_add);
            if break_flag {
                break;
            }
        }
        for func in callee_funcs.iter() {
            debug_assert!(!func.is_extern);
            let caller_used = func.draw_used_callers();
            new_callers_used.extend(caller_used);
        }
        new_callers_used
    }
}
