use super::*;

//定义中转者
enum TmpHolder {
    Reg(Reg),
    StackOffset(i32),
}

static HANDLE_CALL_ACTIONS_PATH: &str = "handle_call_actions.txt";

// 把一个寄存器的值抛出到中转者手中

impl Func {
    /// 把寄存器的值分裂到栈上
    fn split_to_stack(
        &mut self,
        reg_to_split: &Reg,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
        pool: &mut BackendPool,
        split_maps: &mut HashMap<Reg, TmpHolder>,
        phisic_mems: &mut HashMap<Reg, StackSlot>,
    ) {
        debug_assert!(!split_maps.contains_key(reg_to_split));
        let sst = if let Some(sst) = phisic_mems.get(reg_to_split) {
            *sst
        } else {
            let back = self.stack_addr.back().unwrap();
            let new_pos = back.get_pos() + back.get_size();
            let new_sst = StackSlot::new(new_pos, ADDR_SIZE);
            self.stack_addr.push_back(new_sst);
            phisic_mems.insert(*reg_to_split, new_sst);
            new_sst
        };
        let pos = sst.get_pos();
        let sd_inst = LIRInst::build_storetostack_inst(reg_to_split, pos);
        config::record_caller_save_sl(&self.label, "", sd_inst.to_string().as_str());
        new_insts.push(pool.put_inst(sd_inst));
        split_maps.insert(*reg_to_split, TmpHolder::StackOffset(pos));
    }

    ///把寄存器中的值从栈上恢复
    fn load_back_from_certain_pos(
        reg: &Reg,
        pos: i32,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
        pool: &mut BackendPool,
    ) {
        let ld_inst = LIRInst::build_loadstack_inst(reg, pos);
        config::record_caller_save_sl("", "", ld_inst.to_string().as_str());
        new_insts.push(pool.put_inst(ld_inst));
    }

    ///把寄存器的值分裂到空余寄存器里
    fn split_to_reg(
        &mut self,
        reg_to_split: &Reg,
        tmp_holder_reg: &Reg,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
        pool: &mut BackendPool,
        split_maps: &mut HashMap<Reg, TmpHolder>,
        tmp_holder_regs: &mut HashMap<Reg, Reg>,
    ) {
        //
        //如果能够找到同类寄存器做中转
        let mv_inst = LIRInst::build_mv(&reg_to_split, &tmp_holder_reg);
        config::record_caller_save_sl(&self.label, "", mv_inst.to_string().as_str());
        new_insts.push(pool.put_inst(mv_inst));
        tmp_holder_regs.insert(*tmp_holder_reg, *reg_to_split);
        split_maps.insert(*reg_to_split, TmpHolder::Reg(*tmp_holder_reg));
    }

    fn mv_back(
        reg_splited: &Reg,
        tmp_holder_reg: &Reg,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
        pool: &mut BackendPool,
    ) {
        let mv_inst = LIRInst::build_mv(tmp_holder_reg, reg_splited);
        config::record_caller_save_sl("", "", mv_inst.to_string().as_str());
        new_insts.push(pool.put_inst(mv_inst));
    }

    /// callers used 为函数内以及其递归调用的函数们内使用到的caller saved寄存器
    /// callees used 为非external 函数内 以及其递归调用的非external 函数内使用到的callee saved寄存器
    /// callees
    pub fn handle_call(
        &mut self,
        pool: &mut BackendPool,
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_used: &HashMap<String, HashSet<Reg>>,
        callees_be_saved: &HashMap<String, HashSet<Reg>>,
    ) {
        self.calc_live_for_handle_call();

        //来到handle call的时候代码中不存在虚拟寄存器
        debug_assert!(self.draw_all_virtual_regs().len() == 0, "{}", self.label);

        let mut available_tmp_regs: RegUsedStat = RegUsedStat::init_unavailable();
        if self.label != "main" {
            for reg in callees_used.get(self.label.as_str()).unwrap() {
                available_tmp_regs.release_reg(reg.get_color());
            }
            for reg in callers_used.get(self.label.as_str()).unwrap() {
                available_tmp_regs.release_reg(reg.get_color());
            }
        } else {
            for reg in Reg::get_all_not_specials() {
                available_tmp_regs.release_reg(reg.get_color());
            }
        }
        for reg in Reg::get_all_specials_with_s0() {
            available_tmp_regs.use_reg(reg.get_color());
        }

        let available_tmp_regs = available_tmp_regs;

        let mut phisic_mems: HashMap<Reg, StackSlot> = HashMap::new();

        //遇到使用了的callers_used寄存器,就要保存保存到栈上或者保存到一个临时可用寄存器中
        //当遇到了临时可用寄存器的使用者,或者遇到这个值要使用的时候才把这个寄存器的值归还回来
        // 记录寄存器遇到的下一次使用情况
        let bbs: Vec<ObjPtr<BB>> = self.blocks.iter().cloned().collect();
        for bb in bbs.iter() {
            let mut next_occurs = Func::build_next_occurs(bb);
            let mut split_maps: HashMap<Reg, TmpHolder> = HashMap::new(); // to_saved-> tmp holder
            let mut tmp_holder_regs: HashMap<Reg, Reg> = HashMap::new(); //中转使用的reg

            let mut index = 0;
            let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::with_capacity(bb.insts.len());
            //初始化live now
            while index < bb.insts.len() {
                let inst = bb.insts.get(index).unwrap();
                match inst.get_type() {
                    InstrsType::Branch(_) | InstrsType::Jump => {
                        break;
                    }
                    _ => (),
                }

                //遇到指令中使用到在暂存表中的寄存器的情况,则从暂存者手中把值恢复过来
                for reg in inst.get_reg_use() {
                    if let Some(tmp_holder) = split_maps.remove(&reg) {
                        match tmp_holder {
                            TmpHolder::Reg(tmp_holder_reg) => {
                                Func::mv_back(&reg, &tmp_holder_reg, &mut new_insts, pool);
                            }
                            TmpHolder::StackOffset(pos) => {
                                Func::load_back_from_certain_pos(&reg, pos, &mut new_insts, pool);
                            }
                        };
                    }
                }
                let mut to_give_up: Vec<Reg> = Vec::new();
                for (reg, _) in split_maps.iter() {
                    if let Some(next_occur) = next_occurs.get(reg) {
                        let next_occur = next_occur.front().unwrap();
                        if next_occur.1 {
                            to_give_up.push(*reg);
                        }
                    } else {
                        to_give_up.push(*reg);
                    }
                }
                for reg in to_give_up.iter() {
                    let tmp_holder = split_maps.remove(reg).unwrap();
                    match tmp_holder {
                        TmpHolder::Reg(tmp_holder_reg) => {
                            assert!(tmp_holder_regs.remove(&tmp_holder_reg).is_some());
                        }
                        _ => (),
                    }
                }

                //更新next occur
                Func::refresh_next_occurs(&mut next_occurs, index);

                //如果遇到要归还的寄存器自身的def,则放弃对该寄存器的保存
                //如果遇到指令def的寄存器就是正在用来暂存的寄存器的时候,则需要把值归还
                for reg in inst.get_reg_def() {
                    if let Some(tmp_holder) = split_maps.remove(&reg) {
                        unreachable!();
                    }
                }
                for reg in inst.get_reg_def() {
                    if let Some(reg_splited) = tmp_holder_regs.remove(&reg) {
                        Func::mv_back(&reg_splited, &reg, &mut new_insts, pool);
                        split_maps.remove(&reg_splited);
                    }
                }
                if inst.get_type() != InstrsType::Call {
                    new_insts.push(*inst);
                    index += 1;
                    continue;
                }

                // 遇到call指令, (判断该call 指令是否会影响借用的物理寄存器)

                let func_name = inst.get_func_name().unwrap();

                let caller_used = callers_used.get(&func_name).unwrap();
                let callee_used = callees_used.get(func_name.as_str()).unwrap();
                let mut callee_used_but_not_saved = callee_used.clone();
                callee_used_but_not_saved
                    .retain(|reg| !callees_be_saved.get(&func_name).unwrap().contains(reg));
                let mut used_but_not_saved = callee_used_but_not_saved;
                used_but_not_saved.extend(caller_used.iter());
                let used_but_not_saved = used_but_not_saved;
                for reg in used_but_not_saved {
                    if let Some(reg_splited) = tmp_holder_regs.remove(&reg) {
                        split_maps.remove(&reg_splited).unwrap();
                        Func::mv_back(&reg_splited, &reg, &mut new_insts, pool);
                    }
                }
                //从next_occurs表生成 live now
                let mut live_now: HashSet<Reg> = HashSet::new();
                for (reg, next_occurs) in next_occurs.iter() {
                    if let Some(next_occur) = next_occurs.front() {
                        //如果下次出现非def则为活
                        if !next_occur.1 {
                            live_now.insert(*reg);
                        }
                    } else {
                        unreachable!();
                    }
                }
                live_now.extend(inst.get_reg_use());
                //记录需要保存的caller saved寄存器
                let mut to_saved = live_now.clone();
                to_saved.retain(|reg| caller_used.contains(reg));
                to_saved.retain(|reg| !split_maps.contains_key(reg));
                for reg in inst.get_reg_def() {
                    to_saved.remove(&reg);
                }
                // 把寄存器恢复到本身
                // 进行寄存器的保存操作,
                // 首先在可用寄存器表中查询不在live now中也不在caller used中 callee used中的 available_tmp regs
                let mut tmp_holder_regs_choicess = available_tmp_regs;
                caller_used
                    .iter()
                    .for_each(|reg| tmp_holder_regs_choicess.use_reg(reg.get_color()));
                let mut callee_used = callees_used.get(func_name.as_str()).unwrap().clone();
                callee_used.retain(|reg| !callees_be_saved.get(&func_name).unwrap().contains(reg));
                callee_used
                    .iter()
                    .for_each(|reg| tmp_holder_regs_choicess.use_reg(reg.get_color()));
                live_now
                    .iter()
                    .for_each(|reg| tmp_holder_regs_choicess.use_reg(reg.get_color()));

                //然后已经作为中转的寄存器不能够再租借了
                for (p_reg_holder, _) in tmp_holder_regs.iter() {
                    tmp_holder_regs_choicess.use_reg(p_reg_holder.get_color());
                }
                for (p_split, _) in split_maps.iter() {
                    debug_assert!(!tmp_holder_regs_choicess.is_available_reg(p_split.get_color()));
                }
                for reg in inst.get_regs() {
                    debug_assert!(!tmp_holder_regs_choicess.is_available_reg(reg.get_color()));
                }
                assert!(!tmp_holder_regs_choicess.is_available_reg(Reg::get_ra().get_color()));
                assert!(!to_saved.contains(&Reg::get_ra()));
                //首先为寄存器寻找租借者
                for reg in to_saved {
                    if let Some(color) = tmp_holder_regs_choicess.get_available_reg(reg.get_type())
                    {
                        let tmp_holder = Reg::from_color(color);
                        tmp_holder_regs_choicess.use_reg(color);
                        debug_assert!(!Reg::get_all_specials().contains(&tmp_holder));
                        self.split_to_reg(
                            &reg,
                            &tmp_holder,
                            &mut new_insts,
                            pool,
                            &mut split_maps,
                            &mut tmp_holder_regs,
                        );
                        continue;
                    }
                    // 否则使用栈空间做中转,对于同一物理寄存器使用同一栈空间
                    self.split_to_stack(
                        &reg,
                        &mut new_insts,
                        pool,
                        &mut split_maps,
                        &mut phisic_mems,
                    );
                }
                new_insts.push(*inst);
                // 对于当前寄存器def的寄存器,可以让它们从表中清除
                index += 1;
            }

            for (reg, tmp_holder) in split_maps {
                match tmp_holder {
                    TmpHolder::Reg(tmp_holder_reg) => {
                        Func::mv_back(&reg, &tmp_holder_reg, &mut new_insts, pool);
                    }
                    TmpHolder::StackOffset(pos) => {
                        Func::load_back_from_certain_pos(&reg, pos, &mut new_insts, pool);
                    }
                }
            }
            //然后加回跳转指令
            while index < bb.insts.len() {
                let inst = bb.insts.get(index).unwrap();
                debug_assert!(inst.get_type() != InstrsType::Call);
                new_insts.push(*inst);
                index += 1;
            }
            bb.as_mut().insts = new_insts;
        }
    }
}

//tmp handle call
impl Func {
    ///完全保存所有caller saved,然后直接恢复,使用固定空间
    pub fn handle_call_tmp(&mut self, pool: &mut BackendPool) {
        let mut p_mems = HashMap::new();
        let mut build_tmp_slot = |func: &mut Func, reg: &Reg| -> i32 {
            if let Some(pos) = p_mems.get(reg) {
                return *pos;
            }
            let back = func.stack_addr.back().unwrap();
            let pos = back.get_pos() + back.get_size();
            let new_stack_slot = StackSlot::new(pos, ADDR_SIZE);
            func.stack_addr.push_back(new_stack_slot);
            let new_pos = new_stack_slot.get_pos();
            p_mems.insert(*reg, new_pos);
            new_pos
        };
        self.calc_live_for_handle_call();
        let bb_to_process: Vec<ObjPtr<BB>> = self
            .blocks
            .iter()
            .filter(|bb| bb.insts.len() != 0)
            .cloned()
            .collect();
        for bb in bb_to_process {
            let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
            Func::analyse_inst_with_live_now_backorder(bb, &mut |inst, live_now| {
                if inst.get_type() != InstrsType::Call {
                    new_insts.push(inst);
                    return;
                }
                //不需要保存instdef地寄存器
                let mut live_now = live_now.clone();
                live_now.retain(|reg| reg.is_caller_save() && reg != &Reg::get_ra());
                for reg in inst.get_reg_def() {
                    live_now.remove(&reg);
                }
                //恢复
                let live_now: Vec<Reg> = live_now.iter().cloned().collect();
                debug_assert!(live_now.iter().filter(|reg| !reg.is_physic()).count() == 0);
                for reg in live_now.iter() {
                    let pos = build_tmp_slot(self, reg);
                    let load_inst = LIRInst::build_loadstack_inst(reg, pos);
                    new_insts.push(pool.put_inst(load_inst));
                }
                new_insts.push(inst);
                for reg in live_now.iter() {
                    let pos = build_tmp_slot(self, reg);
                    let load_inst = LIRInst::build_storetostack_inst(reg, pos);
                    new_insts.push(pool.put_inst(load_inst));
                }
                //暂存
            });
            new_insts.reverse();
            bb.as_mut().insts = new_insts;
        }
    }
}
