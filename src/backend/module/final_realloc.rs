use core::time;

use crate::backend::regalloc;

use super::*;

impl AsmModule {
    //进行最终的重分配
    pub fn final_realloc(&mut self, pool: &mut BackendPool) {
        // let used_but_not_saved=self.builduse

        let callers_used = self.build_caller_used();
        let callees_used = self.build_callee_used();

        let callees_saved = &self.callee_regs_to_saveds;
        let mut reg_used_but_not_saved =
            AsmModule::build_used_but_not_saveds(&callers_used, &callees_used, callees_saved);

        //禁止在函数调用前后使用s0
        for (_, used_but_not_saved) in reg_used_but_not_saved.iter_mut() {
            used_but_not_saved.insert(Reg::get_s0());
        }
        let reg_used_but_not_saved = reg_used_but_not_saved;

        let mut to_realloc: Vec<ObjPtr<Func>> = self.name_func.iter().map(|(_, f)| *f).collect();
        to_realloc.retain(|f| !f.is_extern);

        self.print_asm("before_final_alloc.txt");
        for func in to_realloc.iter() {
            let name = &func.label;
            // let callers_used = callers_used.get(name).unwrap().clone();
            // let callees_used = callees_used.get(name).unwrap().clone();
            let callers_used = func.draw_used_callers();
            let callees_used = func.draw_used_callees();

            let mut used = callers_used.clone();
            used.extend(callees_used);
            used.insert(Reg::get_s0());
            let availables = used;
            //before alloc
            //记录alloc前的改变
            // let path = format!("{}_{}.txt", name, times);
            // self.print_asm(&path);
            // times += 1;

            // if func.label == "main" {
            //     continue;
            // }
            // if func.label == "params_mix" {
            //     continue;
            // }
            if func.label == "params_fa40" {
                continue;
            }
            // if func.label == "params_f40_i24" {
            //     continue;
            // }

            // 每次
            while regalloc::merge::merge_reg_with_constraints(
                func.as_mut(),
                &availables,
                &reg_used_but_not_saved,
            ) {
                break;
            }
        }

        self.print_asm("before_final_realloc_p.txt");

        // for func in to_realloc {
        //     let name = &func.label;
        //     // let callers_used = callers_used.get(name).unwrap().clone();
        //     // let callees_used = callees_used.get(name).unwrap().clone();
        //     let callers_used = func.draw_used_callers();
        //     let callees_used = func.draw_used_callees();

        //     let mut used = callers_used.clone();
        //     used.extend(callees_used);
        //     used.insert(Reg::get_s0());
        //     let availables = used;
        //     if func.label != "params_fa40" {
        //         continue;
        //     }
        //     // 每次
        //     while regalloc::merge::merge_reg_with_constraints(
        //         func.as_mut(),
        //         &availables,
        //         &reg_used_but_not_saved,
        //     ) {
        //         break;
        //     }
        // }
        self.print_asm("after_final_realloc_p.txt");
        self.print_asm("after_final_realloc.txt");
        // //检查,寄存器合并结束后不应该影响原本的保存需求
        // let new_caller_used=self.build_caller_used();
        // let new_callee_used=self.build_callee_used();
        // let
    }
}
