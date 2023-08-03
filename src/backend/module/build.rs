use super::*;

impl AsmModule {
    /// build v4:
    /// 1. 实现 函数分裂, 优化callee的保存恢复
    /// 2. 指令级 上下文 caller 选择
    /// 3. 对spill use和caller use的栈空间 紧缩
    /// 4. 寄存器重分配:针对call上下文调整函数寄存器组成
    /// 5. 针对函数是否为main调整寄存器组成
    pub fn build_v4(&mut self, f: &mut File, _f2: &mut File, pool: &mut BackendPool, is_opt: bool) {
        let obj_module = ObjPtr::new(self);
        self.build_lir(pool);
        if is_opt {
            BackendPass::new(obj_module).block_pass_pre_clear(pool);
        }

        // self.print_func();
        self.remove_unuse_inst_pre_alloc();
        // self.print_func();

        //检查是否有存在name func里面没有,但是被调用了的函数

        if is_opt {
            // // gep偏移计算合并
            // BackendPass::new(obj_module).opt_gep();

            // 设置一些寄存器为临时变量
            self.cal_tmp_var();

            // 对非临时寄存器进行分配
            self.allocate_reg();
            // 将非临时寄存器映射到物理寄存器
            self.map_v_to_p();
            // 代码调度，列表调度法
            self.list_scheduling_tech();

            // 为临时寄存器分配寄存器
            self.clear_tmp_var();
            self.allocate_reg();
            self.map_v_to_p();
        } else {
            self.allocate_reg();
            self.map_v_to_p();
        }

        self.print_func();
        self.remove_unuse_inst_suf_alloc();
        self.print_func();

        //加入外部函数
        self.add_external_func(pool);

        // //建立调用表
        self.build_own_call_map();
        // //寄存器重分配,重分析

        // self.print_func();

        self.realloc_reg_with_priority();

        self.remove_unuse_inst_suf_alloc();
        // self.print_func();
        self.handle_spill_v3(pool);
        // self.print_func();

        self.remove_unuse_inst_suf_alloc();

        // // self.anaylyse_for_handle_call_v3_pre_split();
        self.anaylyse_for_handle_call_v4();

        let is_opt = true;
        if is_opt {
            self.split_func(pool);
            self.build_own_call_map();
            // self.anaylyse_for_handle_call_v4();
        }

        self.reduce_caller_to_saved_after_func_split();
        self.analyse_caller_regs_to_saved();

        //此后栈空间大小以及 caller saved和callee saved都确定了
        let callers_used = self.build_caller_used();
        let callees_used = self.build_callee_used();
        let callees_be_saved = &self.callee_regs_to_saveds.clone();
        let used_but_not_saved =
            AsmModule::count_used_but_not_saveds(&callers_used, &callees_used, callees_be_saved);
        self.handle_call_v4(pool, &callers_used, &callees_used, callees_be_saved);
        self.remove_external_func(); //在handle call之前调用,删掉前面往name func中加入的external func

        self.rm_inst_suf_handle_call(pool, &used_but_not_saved);
        self.rearrange_stack_slot();
        self.update_array_offset(pool);

        //
        self.rm_inst_suf_update_array_offset(pool, &used_but_not_saved);

        self.build_stack_info(f);
        // self.print_func();
        //删除无用的函数
    }
}
