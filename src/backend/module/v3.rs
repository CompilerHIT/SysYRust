use super::*;

/// build v3:
/// 1. 实现 函数分裂, 优化callee的保存恢复
/// 2. 指令级 上下文 caller 选择
/// 3. 对spill use和caller use的栈空间 紧缩
/// 4. 删除无用函数模板(可选)
impl AsmModule {
    ///处理spillings的虚拟寄存器的临时物理寄存器借用
    pub fn handle_spill_v3(&mut self, pool: &mut BackendPool) {
        self.name_func.iter().for_each(|(_, func)| {
            if func.is_extern {
                return;
            }
            func.as_mut().handle_spill_v3(pool);
        });
    }

    ///对于caller save 和 handle spill  使用到的栈空间 进行紧缩
    pub fn rearrange_stack_slot(&mut self) {
        self.name_func
            .iter()
            .filter(|(_, f)| !f.is_extern)
            .for_each(|(_, func)| func.as_mut().rearrange_stack_slot());
    }

    ///处理 函数调用前后的保存和恢复
    /// 1. 插入保存和恢复caller save的指令
    pub fn handle_call_v3(&mut self, pool: &mut BackendPool) {
        // 分析并刷新每个函数的call指令前后需要保存的caller save信息,以及call内部的函数需要保存的callee save信息
        // 对于 handle call
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.as_mut()
                .handle_call_v3(pool, &self.caller_regs_to_saveds);
        }
    }

    ///加入外部函数,
    pub fn add_external_func(&mut self, pool: &mut BackendPool) {
        // debug_assert!(self.name_func.contains_key("putint"));
        //加入外部函数
        let build_external_func =
            |module: &mut AsmModule, name: &str, pool: &mut BackendPool| -> ObjPtr<Func> {
                let external_context = pool.put_context(Context::new());
                let external_func = pool.put_func(Func::new(name, external_context));
                external_func.as_mut().is_extern = true;
                module.name_func.insert(name.to_string(), external_func);
                external_func
            };
        // let extern_func_path = "extern_func.txt";
        //补充外部函数 memset 和memcpy
        let extern_funcs = vec![
            "memset@plt",
            "memcpy@plt",
            "putint",
            "getint",
            "getarray",
            "putarray",
            "getch",
            "putch",
            "getfloat",
            "putfloat",
            "getfarray",
            "putfarray",
            "putf",
            "_sysy_starttime",
            "_sysy_stoptime",
            "hitsz_thread_init",
            "hitsz_thread_create",
            "hitsz_thread_join",
            "hitsz_thread_exit",
            "hitsz_get_thread_num",
        ];
        for name in extern_funcs.iter() {
            build_external_func(self, &name, pool);
        }
    }

    /// 计算栈空间,进行ra,sp,callee 的保存和恢复
    pub fn build_stack_info(&mut self, f: &mut File) {
        for (name, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            if func.label == "main" {
                func.as_mut().callee_saved.clear(); // main函数不需要保存任何callee saved
            } else {
                let callees = self.callee_regs_to_saveds.get_mut(name).unwrap();
                callees.remove(&Reg::get_sp()); //sp虽然是callee saved但不需要通过栈方式restore
                func.as_mut().callee_saved.extend(callees.iter());
            }
            func.as_mut().save_callee(f);
        }
    }

    ///删除进行函数分裂后的剩余无用函数
    pub fn remove_external_func(&mut self) {
        self.name_func.retain(|_, f| !f.is_extern);
    }

    pub fn update_array_offset(&mut self, pool: &mut BackendPool) {
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            func.as_mut().update_array_offset(pool);
        }
    }
}
