use super::*;

impl AsmModule {
    /// build v3:
    /// 1. 实现 函数分裂, 优化callee的保存恢复
    /// 2. 指令级 上下文 caller 选择
    /// 3. 对spill use和caller use的栈空间 紧缩
    /// 4. 删除无用函数模板(可选)
    pub fn build_v3(&mut self, f: &mut File, _f2: &mut File, pool: &mut BackendPool, is_opt: bool) {
        self.build_lir(pool);
        self.remove_unuse_inst_pre_alloc();

        // self.generate_row_asm(_f2, pool); //generate row  asm可能会造成bug

        if is_opt {
            // 设置一些寄存器为临时变量
            self.cal_tmp_var();

            self.allocate_reg();
            self.map_v_to_p();
            // 代码调度，列表调度法
            self.list_scheduling_tech();

            // 为临时寄存器分配寄存器
            self.clear_tmp_var();
            self.allocate_reg();
            self.map_v_to_p();
        } else {
            // self.generate_row_asm(_f2, pool);
            self.allocate_reg();
            // self.generate_row_asm(_f2, pool);
            // self.map_v_to_p();
            // self.generate_row_asm(_f2, pool);
            // // ///重分配
            // self.name_func.iter().for_each(|(_, func)| {
            //     func.as_mut()
            //         .p2v_pre_handle_call(Reg::get_all_recolorable_regs())
            // });
            // // self.generate_row_asm(_f2, pool);
            // self.allocate_reg();
            self.map_v_to_p();
        }

        self.handle_spill_v3(pool);
        self.remove_unuse_inst_suf_alloc();

        self.add_external_func(pool); //加入外部函数以进行分析
        self.anaylyse_for_handle_call_v3_pre_split();

        if is_opt {
            self.split_func(pool);
        }

        self.remove_useless_func();
        self.handle_call_v3(pool);
        self.rearrange_stack_slot();
        self.update_array_offset(pool);
        self.build_stack_info(f);
        // self.print_func();
        //删除无用的函数
    }

    /// build v4:
    /// 1.寄存器重分配:针对call上下文调整函数寄存器组成
    /// 2.针对函数是否为main调整寄存器组成
    pub fn build_v4(&mut self, f: &mut File, _f2: &mut File, pool: &mut BackendPool, is_opt: bool) {
        self.build_lir(pool);
        self.remove_unuse_inst_pre_alloc();

        // self.generate_row_asm(_f2, pool); //generate row  asm可能会造成bug

        if is_opt {
            // gep偏移计算合并
            BackendPass::new(ObjPtr::new(self)).opt_gep();

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
            // self.generate_row_asm(_f2, pool);
            self.allocate_reg();
            // self.generate_row_asm(_f2, pool);
            // self.map_v_to_p();
            // self.generate_row_asm(_f2, pool);
            // 重分配
            // self.name_func.iter().for_each(|(_, func)| {
            //     func.as_mut()
            //         .p2v_pre_handle_call(Reg::get_all_recolorable_regs())
            // });
            // // self.generate_row_asm(_f2, pool);
            // self.allocate_reg();
            self.map_v_to_p();
        }
        self.remove_unuse_inst_suf_alloc();
        //在寄存器分配后跑两遍寄存器接合
        // for i in 0..2 {
        //     self.p2v();
        //     self.allocate_reg();
        //     self.map_v_to_p();
        //     self.remove_unuse_inst_suf_alloc();
        // }

        //加入外部函数
        self.add_external_func(pool);

        //建立调用表
        self.build_own_call_map();
        //寄存器重分配,重分析

        // self.realloc_reg_with_priority();

        self.handle_spill_v3(pool);
        self.remove_unuse_inst_suf_alloc();

        // self.anaylyse_for_handle_call_v3_pre_split();
        self.anaylyse_for_handle_call_v4();

        if is_opt {
            self.split_func(pool);
            self.build_own_call_map();
            // self.anaylyse_for_handle_call_v3();
        }
        // self.reduce_caller_to_saved_after_func_split();

        self.remove_useless_func(); //在handle call之前调用,删掉前面往name func中加入的external func
        self.handle_call_v3(pool);

        self.rearrange_stack_slot();
        self.update_array_offset(pool);
        self.build_stack_info(f);

        // self.print_func();
        //删除无用的函数
    }
}
