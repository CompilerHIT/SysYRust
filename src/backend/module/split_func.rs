use std::fmt::format;

use crate::backend::regalloc::structs::RegUsedStat;

use super::*;

impl AsmModule {
    pub fn split_func_v4(&mut self, pool: &mut BackendPool) {
        //建立寄存器使用情况表,使用一个reg_use_stat的序列化方法得到
        let main_func = self.name_func.get("main").unwrap();
        let mut new_name_func = HashMap::new();
        new_name_func.insert("main".to_string(), *main_func);

        let mut base_splits: HashMap<String, HashMap<RegUsedStat, String>> = HashMap::new();
        for (name, ff) in self.name_func.iter() {
            if ff.is_extern {
                new_name_func.insert(name.clone(), *ff);
                continue;
            }
            base_splits.insert(name.clone(), HashMap::new());
        }

        //找到新函数,然后进行优先级别分配(不断减少约束)
        let mut to_process: LinkedList<ObjPtr<Func>> = LinkedList::new();
        to_process.push_back(*main_func);
        while !to_process.is_empty() {
            let caller = to_process.pop_front().unwrap();
            AsmModule::analyse_inst_with_live_now(&caller, &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    return;
                }
                let func = inst.get_func_name().unwrap();
                let func_name = &func;
                let func = self.name_func.get(func_name).unwrap();
                if func.is_extern {
                    return;
                }
                let mut live_now = live_now.clone();
                if let Some(def_reg) = inst.get_def_reg() {
                    live_now.remove(&def_reg);
                }

                //分析当前需要保存的callee saved寄存器,或者分析当前需要建立的约束
                //我需要尽量减少任何的callee saved的发生,所有我会优先分配不造成影响的callee saved
                //然后再分配该处不造成影响的caller saved
                //然后再分配该处造成影响的callee saved
                //最后再分配该处会造成影响的caller saved
                let mut all_callers = Reg::get_all_callers_saved();
                let mut all_callees = Reg::get_all_callees_saved();
                //处理论外寄存器以及专用寄存器
                all_callers.remove(&Reg::get_ra());
                all_callees.remove(&&Reg::get_sp());

                let mut bad_callees = all_callees.clone();
                bad_callees.retain(|reg| live_now.contains(reg));
                let mut bad_callers = all_callers.clone();
                bad_callers.retain(|reg| live_now.contains(reg));
                let mut good_callees = all_callees.clone();
                good_callees.retain(|reg| !bad_callees.contains(reg));
                let mut good_callers = all_callers.clone();
                good_callers.retain(|reg| !bad_callers.contains(reg));

                //判断对应函数是否已经存在
                let mut constraint = RegUsedStat::init_unspecial_regs_without_s0();
                bad_callees
                    .iter()
                    .for_each(|reg| constraint.use_reg(reg.get_color()));
                bad_callers
                    .iter()
                    .for_each(|reg| constraint.use_reg(reg.get_color()));

                assert!(bad_callees.remove(&Reg::get_s0()));
                assert!(!constraint.is_available_reg(Reg::get_s0().get_color()));
                let splits = base_splits.get(func_name).unwrap();
                if let Some(new_func) = splits.get(&constraint) {
                    inst.as_mut().replace_label(new_func.clone());
                    return;
                }

                //
                let mut ord_regs: Vec<Reg> = Vec::new();
                ord_regs.extend(good_callees.iter());
                ord_regs.extend(good_callers.iter());
                ord_regs.extend(bad_callers.iter());
                ord_regs.extend(bad_callees.iter());
                debug_assert!(ord_regs.len() == 58, "{}", ord_regs.len());

                //按照顺序进行分配,分配确定之后,再之后不会再改变函数内的寄存器组成

                //因为分配在handle spill之后,所以只能够求一个完美分配,
                //获得原函数的一个深度clone
                let new_func = func.real_deep_clone(pool);

                new_func.as_mut().alloc_reg_with_priority(ord_regs);
                let new_constraint = new_func.draw_phisic_regs();

                if let Some(new_func_name) = splits.get(&new_constraint) {
                    inst.as_mut().replace_label(new_func_name.clone());
                    return;
                }

                //分析分配后的寄存器使用结果,根据它自身used的 caller saved寄存器和callee 寄存器给它建模
                let used = new_func.draw_phisic_regs();
                let sufix_mark = used.draw_code_mark();
                let bb_sufix = format!("_hitsz_{}_{}", func_name.clone(), sufix_mark.clone());
                let new_func_name = format!("{}_hitsz_{}", func_name.clone(), sufix_mark.clone());

                new_func.as_mut().set_name(&new_func_name);
                new_func.as_mut().suffix_bb(&bb_sufix);

                //判断新函数是否是第一个
                if splits.len() == 0 {
                    new_func.as_mut().is_header = true;
                } else {
                    new_func.as_mut().is_header = false;
                }
                new_name_func.insert(new_func_name.clone(), new_func);
                base_splits
                    .get_mut(func_name)
                    .unwrap()
                    .insert(constraint, new_func_name.clone());
                //new func name 同时还可以存在它自己刚好存在的constraint中,
                base_splits
                    .get_mut(func_name)
                    .unwrap()
                    .insert(new_constraint, new_func_name.clone());
                inst.as_mut().replace_label(new_func_name);
                to_process.push_back(new_func);
            });
        }
        self.name_func = new_name_func;

        self.base_splits = base_splits;
    }
}
