use crate::config;

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
        config::record_event("finish build lir");

        // self.print_asm("asm_abastract.txt");
        // let is_opt = true;
        if is_opt {
            config::record_event("start block_pass_pre_clear");
            BackendPass::new(obj_module).block_pass_pre_clear(pool);
            config::record_event("finish block_pass_pre_clear");
        }

        self.print_asm("after_block_opt.log");

        self.remove_unuse_inst_pre_alloc();
        config::record_event("finish rm pre first alloc");

        if is_opt {
            // // gep偏移计算合并
            // BackendPass::new(obj_module).opt_gep();
            config::record_event("start schedule");
            // 设置一些寄存器为临时变量
            self.cal_tmp_var();

            // 对非临时寄存器进行分配
            self.allocate_reg();
            // 将非临时寄存器映射到物理寄存器
            self.map_v_to_p();
            // 窥孔
            // BackendPass::new(obj_module).fuse_tmp_regs();
            // 代码调度，列表调度法
            // self.list_scheduling_tech();

            // // 为临时寄存器分配寄存器
            self.clear_tmp_var();

            self.allocate_reg();
            self.map_v_to_p();
            config::record_event("finish schedule");
        } else {
            self.allocate_reg();
            self.map_v_to_p();
        }
        self.remove_unuse_inst_suf_alloc();
        config::record_event("finish rm inst suf first alloc");
        //加入外部函数
        self.add_external_func(pool);
        // //建立调用表
        self.build_own_call_map();
        // //寄存器重分配,重分析
        if is_opt {
            self.realloc_pre_split_func();
            config::record_event("finish realloc pre spilit func");
        }
        config::record_event("start handle spill");
        if false {
            self.handle_spill_v3(pool);
        } else {
            self.handle_spill_tmp(pool);
        }
        config::record_event("finish handle spill");
        // if is_opt {
        //     //似乎存在bug,并且目前没有收益,暂时放弃
        //     self.split_func(pool);
        //     self.build_own_call_map();
        // }
        //此后栈空间大小以及 caller saved和callee saved都确定了

        if is_opt {
            let callers_used = self.build_caller_used();
            let callees_used = self.build_callee_used();
            self.caller_regs_to_saveds = callers_used.clone();
            self.callee_regs_to_saveds = callees_used.clone();
        } else {
            self.anaylyse_for_handle_call_v4();
        }
        config::record_event("finish analyse for handle call");
        let callers_used = self.build_caller_used();
        let callees_used = self.build_callee_used();
        let callees_be_saved = &self.callee_regs_to_saveds.clone();
        let used_but_not_saved =
            AsmModule::build_used_but_not_saveds(&callers_used, &callees_used, callees_be_saved);
        config::record_event("start handle call");
        if is_opt {
            self.handle_call(pool, &callers_used, &callees_used, callees_be_saved);
        } else {
            self.handle_call_tmp(pool);
        }
        config::record_event("finish handle call");
        let is_opt = true;
        if is_opt && config::get_rest_secs() > 120 {
            println!("{}", config::get_rest_secs());
            assert!(config::get_rest_secs() > 60);
            config::record_event("start rm before rearrange");
            self.rm_inst_before_rearrange(pool, &used_but_not_saved);
            config::record_event("finish rm before rearrange");
            config::record_event("start mem rearrange");
            self.rearrange_stack_slot();
            config::record_event("finish mem rearrange");
        }
        self.update_array_offset(pool);
        config::record_event("finish update_array_offset");
        if true {
            self.rm_inst_suf_update_array_offset(pool, &used_but_not_saved);
            config::record_event("finish rm suf update array offset");
        }
        //检查代码中是否会def sp
        self.build_stack_info(f);
    }
}
