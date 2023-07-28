use super::*;

///handle call v3的实现
impl Func {
    ///calc_live for handle call v3
    /// 仅仅对6个特殊寄存器x0-x4以及s0认为始终活跃
    /// 其他寄存器都动态分析
    pub fn calc_live_for_handle_call(&self) {
        //TODO, 去除allocable限制!
        self.calc_live_base();
        //把 特殊寄存器 (加入自己的in 和 out)
        for bb in self.blocks.iter() {
            //0:zero, 1:ra, 2:sp 3:gp ,4,tp
            for id in 0..=4 {
                bb.as_mut().live_in.insert(Reg::new(id, ScalarType::Int));
                bb.as_mut().live_out.insert(Reg::new(id, ScalarType::Int));
            }
            //加入s0,handle call发生在handle overflow之前,把s0标记为无限存活以避免被使用
            bb.as_mut().live_in.insert(Reg::new(8, ScalarType::Int));
            bb.as_mut().live_out.insert(Reg::new(8, ScalarType::Int));
        }
    }

    /// 在handle spill之后调用
    /// 里面的 callee saved传入的是 函数模板对应内部使用到的寄存器
    pub fn analyse_for_handle_call(
        &self,
        callee_saved: &HashMap<String, HashSet<Reg>>,
    ) -> Vec<(ObjPtr<LIRInst>, HashSet<Reg>)> {
        let mut new_funcs: Vec<(ObjPtr<LIRInst>, HashSet<Reg>)> = Vec::new();
        self.calc_live_for_handle_call();
        for bb in self.blocks.iter() {
            let mut livenow: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                debug_assert!(reg.is_physic());
                livenow.insert(*reg);
            });
            //然后倒序
            for inst in bb.insts.iter().rev() {
                for reg in inst.get_reg_def() {
                    debug_assert!(livenow.contains(&reg), "{}", reg);
                    livenow.remove(&reg);
                }
                //分析如果该指令为call指令的时候上下文中需要保存的callee saved
                if inst.get_type() == InstrsType::Call {
                    let func_label = inst.get_func_name().unwrap();
                    //如果是当前活跃并且在func used列表中的寄存器的callee saved寄存器 才是需要保存的寄存器
                    let callees_saved_now: HashSet<Reg> = callee_saved
                        .get(&func_label)
                        .unwrap()
                        .iter()
                        .cloned()
                        .filter(|reg| livenow.contains(reg))
                        .collect();
                    new_funcs.push((*inst, callees_saved_now));
                }
                for reg in inst.get_reg_use() {
                    livenow.insert(reg);
                }
            }
        }
        new_funcs
    }

    pub fn set_name(&mut self, new_name: &String) {
        self.label = new_name.clone();
        for bb in self.blocks.iter() {
            bb.as_mut().func_label = new_name.clone();
        }
    }
    /// 给label改名,加上指定后缀
    pub fn suffix_bb(&mut self, suffix: &String) {
        //记录bb,遇到指令进行替换
        let mut old_new = HashMap::new();
        for bb in self.blocks.iter() {
            let mut new_bb_label = bb.label.clone();
            new_bb_label.push_str(&suffix);
            old_new.insert(bb.as_mut().label.clone(), new_bb_label.clone());
            bb.as_mut().label = new_bb_label;
        }
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                let old = inst.get_bb_label();
                if old.is_none() {
                    continue;
                }
                let new = old_new.get(&old.unwrap()).unwrap().clone();
                inst.as_mut().replace_label(new);
            }
        }
    }

    /// 给局部静态数组改名,加上指定后缀
    // pub fn suffix_local_arr(&mut self, suffix: &String) {
    //     todo!();
    // }

    ///函数分裂用到的函数的真实深度克隆
    pub fn real_deep_clone(&self, pool: &mut BackendPool) -> ObjPtr<Func> {
        let context = pool.put_context(Context::new());
        let mut new_func = Func::new("default", context);
        new_func.blocks = Vec::new();
        let mut old_to_new_bbs: HashMap<ObjPtr<BB>, ObjPtr<BB>> = HashMap::new();
        let mut old_to_new_insts: HashMap<ObjPtr<LIRInst>, ObjPtr<LIRInst>> = HashMap::new();
        //复制bb 的内容
        for bb in self.blocks.iter() {
            let mut new_bb = BB::new(&bb.label.clone(), &new_func.label);
            new_bb.showed = bb.showed;
            new_bb.insts = Vec::new();
            for inst in bb.insts.iter() {
                let new_inst = inst.as_ref().clone();
                let new_inst = pool.put_inst(new_inst);
                new_bb.insts.push(new_inst);
                old_to_new_insts.insert(*inst, new_inst);
            }
            let new_bb = pool.put_block(new_bb);
            old_to_new_bbs.insert(*bb, new_bb);
            new_func.blocks.push(new_bb);
        }
        //复制bb 的 出入关系
        for bb in self.blocks.iter() {
            let new_bb = old_to_new_bbs.get(bb).unwrap();
            bb.in_edge.iter().for_each(|in_bb| {
                let new_in_bb = old_to_new_bbs.get(in_bb).unwrap();
                new_bb.as_mut().in_edge.push(*new_in_bb);
            });
            bb.out_edge.iter().for_each(|out_bb| {
                let new_out_bb = old_to_new_bbs.get(out_bb).unwrap();
                new_bb.as_mut().out_edge.push(*new_out_bb);
            })
        }

        new_func.entry = Some(*old_to_new_bbs.get(&self.entry.unwrap()).unwrap());
        new_func.is_extern = self.is_extern;
        new_func.is_header = self.is_header;
        new_func.param_cnt = self.param_cnt;
        // new_func.params
        new_func.stack_addr = self.stack_addr.iter().cloned().collect();
        new_func.spill_stack_map = self.spill_stack_map.clone();
        new_func.const_array = self.const_array.clone();
        new_func.float_array = self.float_array.clone();
        new_func.callee_saved = self.callee_saved.iter().cloned().collect();
        // new_func.caller_saved = self.caller_saved.clone();
        // new_func.caller_saved_len = self.caller_saved_len; //TODO,修改
        new_func.array_slot = self.array_slot.iter().cloned().collect();
        // 对 array inst 进行复制
        new_func.array_inst.clear();
        for inst in self.array_inst.iter() {
            let new_inst = old_to_new_insts.get(inst).unwrap();
            new_func.array_inst.push(*new_inst);
        }
        pool.put_func(new_func)
    }

    ///配合v3系列的module.build
    /// 实现了自适应函数调用
    /// callers_used 为  (func name, the caller saved reg this func used)
    pub fn handle_call_v3(
        &mut self,
        pool: &mut BackendPool,
        callers_used: &HashMap<String, HashSet<Reg>>,
    ) {
        self.calc_live_for_handle_call();
        let mut slots_for_caller_saved: Vec<StackSlot> = Vec::new();
        // self.print_func();
        for bb in self.blocks.iter() {
            let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
            let mut live_now: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                live_now.insert(*reg);
            });
            for inst in bb.insts.iter().rev() {
                for reg in inst.get_reg_def() {
                    debug_assert!(
                        live_now.contains(&reg),
                        "blocak:{},inst:{},reg:{}",
                        bb.label,
                        inst.as_ref(),
                        reg
                    );
                    live_now.remove(&reg);
                }

                if inst.get_type() == InstrsType::Call {
                    // 找出 caller saved
                    let mut to_saved: Vec<Reg> = Vec::new();
                    for reg in live_now.iter() {
                        //需要注意ra寄存器虽然是caller saved,但是不需要用栈空间方式进行restore
                        if reg.is_caller_save() && reg.get_id() != 1 {
                            to_saved.push(*reg);
                        }
                    }
                    //TODO to_check, 根据指令判断是否使用
                    let func_name = inst.get_func_name().unwrap();
                    let callers_used = callers_used.get(&func_name).unwrap();
                    to_saved = to_saved
                        .iter()
                        .cloned()
                        .filter(|reg| callers_used.contains(reg))
                        .collect();
                    //根据调用的函数的情况,判断这个函数使用了哪些caller save寄存器
                    // 准备栈空间
                    while slots_for_caller_saved.len() < to_saved.len() {
                        let last_slot = self.stack_addr.back().unwrap();
                        let new_pos = last_slot.get_pos() + last_slot.get_size();
                        let new_slot = StackSlot::new(new_pos, ADDR_SIZE);
                        self.stack_addr.push_back(new_slot);
                        slots_for_caller_saved.push(new_slot);
                    }
                    //产生一条指令
                    let build_ls = |reg: Reg, offset: i32, kind: InstrsType| -> LIRInst {
                        debug_assert!(
                            (kind == InstrsType::LoadFromStack || kind == InstrsType::StoreToStack)
                        );
                        let mut ins = LIRInst::new(
                            kind,
                            vec![Operand::Reg(reg), Operand::IImm(IImm::new(offset))],
                        );
                        ins.set_double();
                        ins
                    };
                    // 插入恢复指令
                    for (index, reg) in to_saved.iter().enumerate() {
                        let pos = slots_for_caller_saved.get(index).unwrap().get_pos();
                        let load_inst = build_ls(*reg, pos, InstrsType::LoadFromStack);
                        let load_inst = pool.put_inst(load_inst);
                        new_insts.push(load_inst);
                        config::record_caller_save_sl(
                            &self.label,
                            &bb.label,
                            format!("load{reg}").as_str(),
                        );
                    }
                    new_insts.push(*inst); //插入call指令
                                           //插入保存指令
                    for (index, reg) in to_saved.iter().enumerate() {
                        let pos = slots_for_caller_saved.get(index).unwrap().get_pos();
                        let store_inst = build_ls(*reg, pos, InstrsType::StoreToStack);
                        let store_inst = pool.put_inst(store_inst);
                        new_insts.push(store_inst);
                        config::record_caller_save_sl(
                            &self.label,
                            &bb.label,
                            format!("store{reg}").as_str(),
                        );
                    }
                } else {
                    new_insts.push(*inst);
                }
                for reg in inst.get_reg_use() {
                    live_now.insert(reg);
                }
            }
            new_insts.reverse();
            bb.as_mut().insts = new_insts;
        }
        // self.print_func();
    }

    // pub fn handle_call_v4(
    //     &mut self,
    //     pool: &mut BackendPool,
    //     callers_used: &HashMap<String, HashSet<Reg>>,
    // ) {
    //     //根据上下文决定对函数能够使用哪些
    // }
}
