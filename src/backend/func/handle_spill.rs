use crate::log_file_uln;

use super::*;

static SPILL_ACTIONS_PATH: &str = "spill_actions.txt";

/// handle spill v3实现
impl Func {
    ///为handle spill 计算寄存器活跃区间
    /// 会认为zero,ra,sp,tp,gp,s0在所有块中始终活跃
    pub fn calc_live_for_handle_spill(&self) {
        self.calc_live_base();
        //把sp和ra寄存器加入到所有的块的live out,live in中，表示这些寄存器永远不能在函数中自由分配使用
        for bb in self.blocks.iter() {
            //0:zero, 1:ra, 2:sp 3:gp 4:tp
            for id in 0..=4 {
                bb.as_mut().live_in.insert(Reg::new(id, ScalarType::Int));
                bb.as_mut().live_out.insert(Reg::new(id, ScalarType::Int));
            }
            //加入s0,避免在handle spill中使用了s0
            bb.as_mut().live_in.insert(Reg::new(8, ScalarType::Int));
            bb.as_mut().live_out.insert(Reg::new(8, ScalarType::Int));
        }
    }

    ///精细化的handle spill:
    ///
    ///遇到spilling寄存器的时候:
    /// * 优先使用available的寄存器
    ///     其中,优先使用caller save的寄存器
    ///     ,再考虑使用callee save的寄存器.
    /// * 如果要使用unavailable的寄存器,才需要进行spill操作来保存和恢复原值
    ///     优先使用caller save的寄存器,
    /// * 一定要spill到内存上的时候,使用递增的slot,把slot记录到数组的表中,等待重排
    pub fn handle_spill(&mut self, pool: &mut BackendPool) {
        self.calc_live_for_handle_spill();
        //先分配空间
        //对于spillings用到的空间直接一人一个
        let regs = self.draw_all_virtual_regs();
        debug_assert!(regs.len() == self.reg_alloc_info.spillings.len());
        for spilling_reg in self.reg_alloc_info.spillings.iter() {
            debug_assert!(
                regs.contains(&Reg::new(*spilling_reg, ScalarType::Int))
                    || regs.contains(&Reg::new(*spilling_reg, ScalarType::Float))
            );
            let last = self.stack_addr.back().unwrap();
            let new_pos = last.get_pos() + last.get_size();
            let new_stack_slot = StackSlot::new(new_pos, ADDR_SIZE);
            let if_i_reg = Reg::new(*spilling_reg, ScalarType::Int);
            let if_f_reg = Reg::new(*spilling_reg, ScalarType::Float);
            debug_assert!(!(regs.contains(&&if_i_reg) && regs.contains(&if_f_reg)));
            let spilling_reg = if regs.contains(&if_i_reg) {
                if_i_reg
            } else {
                if_f_reg
            };
            self.spill_stack_map.insert(spilling_reg, new_stack_slot);
            self.stack_addr.push_back(new_stack_slot);
        }
        // Func::print_func(ObjPtr::new(&self), "mm.txt");
        //为物理寄存器相关的借还开辟空间

        let mut phisic_mems = HashMap::new();
        for reg in Reg::get_all_not_specials() {
            let last = self.stack_addr.back().unwrap();
            let new_pos = last.get_pos() + last.get_size();
            let new_stack_slot = StackSlot::new(new_pos, ADDR_SIZE);
            self.stack_addr.push_back(new_stack_slot);
            phisic_mems.insert(reg, new_stack_slot);
        }

        self.print_live_interval(
            format!("live_interval_before_handle_spill_{}.txt", self.label).as_str(),
        );
        // debug_assert!();
        let to_process = self.blocks.iter().cloned().collect::<Vec<ObjPtr<BB>>>();
        // Func::print_func(ObjPtr::new(&self), "before_handle_spill.txt");
        log_file!(SPILL_ACTIONS_PATH, "\n\nfunc:{}", self.label);
        for bb in to_process.iter() {
            if bb.insts.len() == 0 {
                continue;
            }
            self.handle_spill_for_block(&phisic_mems, bb, pool);
        }
        // self.remove_inst_suf_spill(pool);
        Func::print_func(
            ObjPtr::new(&self),
            format!("after_handle_spill_{}.txt", self.label).as_str(),
        );

        debug_assert!(self.draw_all_virtual_regs().len() == 0);
    }

    pub fn handle_spill_tmp(&mut self, pool: &mut BackendPool) {
        self.calc_live_for_handle_spill();
        //先分配空间
        //对于spillings用到的空间直接一人一个
        let mut spill_stack_map: HashMap<i32, StackSlot> = HashMap::new();
        for spilling_reg in self.reg_alloc_info.spillings.iter() {
            let last = self.stack_addr.back().unwrap();
            let new_pos = last.get_pos() + last.get_size();
            let new_stack_slot = StackSlot::new(new_pos, ADDR_SIZE);
            spill_stack_map.insert(*spilling_reg, new_stack_slot);
            self.stack_addr.push_back(new_stack_slot);
        }

        Func::print_func(ObjPtr::new(&self), "before_handle_spill.txt");
        let to_process = self.blocks.iter().cloned().collect::<Vec<ObjPtr<BB>>>();
        for bb in to_process.iter() {
            if bb.insts.len() == 0 {
                continue;
            }
            Func::handle_spill_of_block_tmp(bb, pool, &spill_stack_map);
        }
        // rm inst suf spill对于hf32存在bug
        // self.remove_inst_suf_spill(pool);
    }
}

///tmp handle spill的实现
impl Func {
    /// 认为有三个临时寄存器可以使用
    fn handle_spill_of_block_tmp(
        bb: &ObjPtr<BB>,
        pool: &mut BackendPool,
        spill_stack_map: &HashMap<i32, StackSlot>,
    ) {
        //直接保存恢复保存恢复 (使用t0-t2三个寄存器)
        let mut new_insts = Vec::new();
        let mut tmp_available = RegUsedStat::init_unavailable();
        for i in 5..=7 {
            tmp_available.release_reg(i);
        }
        for i in 18..=20 {
            tmp_available.release_reg(i + FLOAT_BASE);
        }
        let tmp_available = tmp_available;
        for inst in bb.insts.iter() {
            let mut tmp_use_stat = tmp_available;
            let defed = inst.get_reg_def();
            let regs = inst.get_regs();
            for r in inst.get_regs() {
                if r.is_physic() {
                    assert!(!tmp_available.is_available_reg(r.get_color()), "{r}");
                }
            }
            let mut borrows = HashMap::new();
            for reg in regs.iter() {
                if reg.is_physic() {
                    continue;
                }
                let tmp_reg = tmp_use_stat.get_available_reg(reg.get_type()).unwrap();
                tmp_use_stat.use_reg(tmp_reg);
                let tmp_reg = Reg::from_color(tmp_reg);
                //把值存到物理寄存器
                //从栈上加载值
                let pos = spill_stack_map.get(&reg.get_id()).unwrap().get_pos();
                let ld_inst = LIRInst::build_loadstack_inst(&tmp_reg, pos);
                new_insts.push(pool.put_inst(ld_inst));
                inst.as_mut().replace_reg(&reg, &tmp_reg);
                config::record_spill(
                    "",
                    "",
                    format!("从栈{}把{}值取入{}", pos, reg, tmp_reg).as_str(),
                );
                borrows.insert(reg, tmp_reg);
            }
            new_insts.push(*inst);
            for reg in defed.iter() {
                if !reg.is_physic() {
                    let pos = spill_stack_map.get(&reg.get_id()).unwrap().get_pos();
                    let borrowed = borrows.get(&reg).unwrap();
                    let sd_inst = LIRInst::build_storetostack_inst(&borrowed, pos);
                    new_insts.push(pool.put_inst(sd_inst));
                    config::record_spill(
                        "",
                        "",
                        format!("把{}值从{}存入栈{}", reg, borrowed, pos).as_str(),
                    );
                }
            }
        }
        bb.as_mut().insts = new_insts;
    }
}

impl Func {
    fn handle_spill_for_inst(
        index: usize,
        inst: &ObjPtr<LIRInst>,
        choose_borrow: &dyn Fn(
            &Reg,
            &HashMap<Reg, LinkedList<(usize, bool)>>,
            &HashMap<Reg, Reg>,
            &HashMap<Reg, Reg>,
            RegUsedStat,
        ) -> Reg,
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        phisic_mems: &HashMap<Reg, StackSlot>,
        next_occurs: &mut HashMap<Reg, LinkedList<(usize, bool)>>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        //TODO,选择合适的启发函数选择要借用的寄存器
        // let choose_borrow = Func::choose_borrow_2;
        // let choose_borrow = Func::choose_borrow_1;
        //首先根据当前下标更新next occurs
        Func::refresh_next_occurs(next_occurs, index);
        let used = inst.get_reg_use();
        let defed = inst.get_reg_def();
        //归还必须归还的物理寄存器
        for reg in used.iter().filter(|reg| reg.is_physic()) {
            match holders.get(reg) {
                Some(holder) => {
                    let holder = *holder;
                    let holder = &holder;
                    if holder != reg {
                        //物理寄存器不在持有者手上,则需要进行归还
                        Func::return_reg(
                            holder,
                            reg,
                            spill_stack_map,
                            rentors,
                            holders,
                            pool,
                            new_insts,
                        );
                        // 然后加载回物理寄存器的原值
                        Func::load_back_phy(reg, &phisic_mems, rentors, holders, pool, new_insts);
                    }
                }
                _ => {
                    // 加载回物理寄存器的原值
                    Func::load_back_phy(reg, &phisic_mems, rentors, holders, pool, new_insts);
                }
            }
        }
        for reg in defed.iter().filter(|reg| reg.is_physic()) {
            if let Some(holder) = holders.get(reg) {
                // 如果持有者不是本人则需要归还
                let rentor = *holder;
                if holder != reg {
                    Func::return_reg(
                        &rentor,
                        reg,
                        spill_stack_map,
                        rentors,
                        holders,
                        pool,
                        new_insts,
                    );
                }
            }
            holders.insert(*reg, *reg);
        }

        let mut availables: RegUsedStat = RegUsedStat::init_unspecial_regs_without_s0();
        let mut regs = inst.get_regs();
        //记录不能够使用的寄存器
        for reg in regs.iter() {
            if let Some(borrowed) = rentors.get(reg) {
                availables.use_reg(borrowed.get_color());
            } else if reg.is_physic() {
                availables.use_reg(reg.get_color());
            }
        }
        regs.retain(|reg| !reg.is_physic());
        let regs = regs;

        let used: HashSet<Reg> = used.iter().cloned().collect();
        //给没有选中物理寄存器租借的虚拟寄存器寻找适合租借的物理寄存器
        for new_rentor in regs.iter() {
            if rentors.contains_key(&new_rentor) {
                continue;
            }
            let to_borrow = choose_borrow(new_rentor, &next_occurs, &rentors, &holders, availables);
            availables.use_reg(to_borrow.get_color());
            // 如果目标要借用的物理寄存器有主人,则free
            if holders.contains_key(&to_borrow) {
                Func::free_preg(
                    &to_borrow,
                    rentors,
                    holders,
                    spill_stack_map,
                    phisic_mems,
                    pool,
                    new_insts,
                );
            }

            if used.contains(new_rentor) {
                Func::load_back_virtual(
                    new_rentor,
                    &to_borrow,
                    rentors,
                    holders,
                    spill_stack_map,
                    pool,
                    new_insts,
                );
            } else {
                holders.insert(to_borrow, *new_rentor);
                rentors.insert(*new_rentor, to_borrow);
            }
        }

        // log_file_uln!("")
        //把虚拟寄存器替换为它们租借的物理寄存器
        for reg in regs.iter() {
            debug_assert!(rentors.contains_key(reg));
            let borrow = rentors.get(reg).unwrap();
            inst.as_mut().replace_reg(reg, borrow);
        }

        debug_assert!(
            inst.get_regs()
                .iter()
                .filter(|reg| !reg.is_physic())
                .count()
                == 0
        );
        //根据next occur更新rentor和holder
        Func::refresh_rentors_and_holders_with_next_occur(rentors, holders, &next_occurs);
        new_insts.push(*inst);
        log_file!(SPILL_ACTIONS_PATH, "{}", inst.to_string());
    }

    //不需要给物理寄存器分配空间,因为每个块中都会为物理寄存器临时分配空间
    pub fn handle_spill_for_block(
        &mut self,
        phisic_mems: &HashMap<Reg, StackSlot>,
        bb: &ObjPtr<BB>,
        pool: &mut BackendPool,
    ) {
        log_file!(SPILL_ACTIONS_PATH, "\nblock:{}", bb.label);
        let spill_stack_map = &self.spill_stack_map;
        let mut next_occurs = Func::build_next_occurs(bb);
        let mut new_insts = Vec::new();
        let mut rentors: HashMap<Reg, Reg> = HashMap::new();
        let mut holders: HashMap<Reg, Reg> = HashMap::new();
        bb.live_in
            .iter()
            .filter(|reg| reg.is_physic())
            .for_each(|reg| {
                holders.insert(*reg, *reg);
            });
        let mut process_one = |index: usize, inst: &ObjPtr<LIRInst>| {
            Func::handle_spill_for_inst(
                index,
                inst,
                &Func::choose_borrow_1,
                &mut rentors,
                &mut holders,
                spill_stack_map,
                phisic_mems,
                &mut next_occurs,
                pool,
                &mut new_insts,
            );
        };
        let mut index = 0;
        while index < bb.insts.len() {
            let inst = bb.insts.get(index).unwrap();
            match inst.get_type() {
                InstrsType::Branch(_) | InstrsType::Jump => {
                    break;
                }
                _ => (),
            }
            process_one(index, inst);
            index += 1;
        }

        //归还所有被spilling寄存器借用得的物理寄存器
        let to_give_back: Vec<Reg> = rentors.iter().map(|(_, r)| *r).collect();
        for reg in to_give_back {
            let rentor = *holders.get(&reg).unwrap();
            debug_assert!(!rentor.is_physic());
            if bb.live_out.contains(&rentor) {
                Func::return_reg(
                    &rentor,
                    &reg,
                    spill_stack_map,
                    &mut rentors,
                    &mut holders,
                    pool,
                    &mut new_insts,
                );
            }
        }
        // 对于live out中的寄存器可以先split到栈上,然后再根据需要判断是否需要归还
        // 加载回物理寄存器的原值(所有在live out中的值都要加载回来)
        bb.live_out
            .iter()
            .filter(|reg| reg.is_physic())
            .for_each(|reg| {
                if holders.contains_key(reg) {
                    let holder = *holders.get(reg).unwrap();
                    if holder != *reg {
                        Func::return_reg(
                            &holder,
                            reg,
                            spill_stack_map,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    } else {
                        return;
                    }
                }
                Func::load_back_phy(
                    reg,
                    phisic_mems,
                    &mut rentors,
                    &mut holders,
                    pool,
                    &mut new_insts,
                );
                debug_assert!(holders.get(reg).unwrap() == reg);
            });
        // 使用的不同启发函数
        let mut process_one = |index: usize, inst: &ObjPtr<LIRInst>| {
            Func::handle_spill_for_inst(
                index,
                inst,
                &Func::choose_borrow_2,
                &mut rentors,
                &mut holders,
                spill_stack_map,
                phisic_mems,
                &mut next_occurs,
                pool,
                &mut new_insts,
            );
        };
        //对结尾跳转语句的处理(使用临时寄存器,不在live out里面的内容不去保存)
        while index < bb.insts.len() {
            let inst = bb.insts.get(index).unwrap();
            match inst.get_type() {
                InstrsType::Branch(_) | InstrsType::Jump => (),
                _ => {
                    unreachable!("{}", inst.as_ref());
                    continue;
                }
            };
            process_one(index, inst);
            index += 1;
        }

        bb.as_mut().insts = new_insts;
    }

    ///把spilling寄存器的值从借来的物理寄存器上转回栈上，归还物理寄存器
    /// 并且修改rentors表和holders表,清除rentors和holders表上对应记录
    fn return_reg(
        rentor: &Reg,
        rented_reg: &Reg,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        //
        debug_assert!(!rentor.is_physic());
        debug_assert!(rented_reg.is_physic());
        debug_assert!(holders.get(rented_reg).unwrap() == rentor);
        debug_assert!(rentors.get(rentor).unwrap() == rented_reg);
        //先把rentor的值存回栈上
        let pos = spill_stack_map.get(rentor).unwrap().get_pos();
        let store_inst = LIRInst::build_storetostack_inst(rented_reg, pos);
        new_insts.push(pool.put_inst(store_inst));
        rentors.remove(rentor);
        holders.remove(rented_reg);
        config::record_spill(
            "",
            "",
            format!("把{}值从{}存回栈{}上", rentor, rented_reg, pos).as_str(),
        );
    }

    ///加载回物理寄存器的原值
    fn load_back_phy(
        p_reg: &Reg,
        phisic_mems: &HashMap<Reg, StackSlot>,
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        debug_assert!(!holders.contains_key(p_reg));
        let pos = phisic_mems.get(p_reg).unwrap().get_pos();
        let load_inst = LIRInst::build_loadstack_inst(p_reg, pos);
        new_insts.push(pool.put_inst(load_inst));
        holders.insert(*p_reg, *p_reg);
        config::record_spill(
            "",
            "",
            format!("从栈{}上加载回物理寄存器{}原值", pos, p_reg).as_str(),
        );
    }

    /// 只有使用虚拟寄存器的值的时候,才需要取回虚拟寄存器的原值
    fn load_back_virtual(
        v_reg: &Reg,
        to_borrow: &Reg,
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        // 需要确保当前没有别的寄存器持有该物理寄存器
        debug_assert!(!holders.contains_key(to_borrow));
        //并且当前虚拟寄存器没有借用到任何物理寄存器
        debug_assert!(!rentors.contains_key(v_reg));
        let pos = spill_stack_map.get(v_reg).unwrap().get_pos();
        let load_inst = LIRInst::build_loadstack_inst(to_borrow, pos);
        config::record_spill("", "", format!("{}", load_inst).as_str());
        new_insts.push(pool.put_inst(load_inst));
        rentors.insert(*v_reg, *to_borrow);
        holders.insert(*to_borrow, *v_reg);
    }

    /// 释放某个物理寄存器 (需要把原值保存到对应区域)
    fn free_preg(
        p_reg: &Reg,
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        phisic_mems: &HashMap<Reg, StackSlot>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        debug_assert!(holders.contains_key(p_reg));
        let holder = *holders.get(p_reg).unwrap();
        let pos = if holder.is_physic() {
            debug_assert!(holder == *p_reg);
            holders.remove(&holder);
            phisic_mems
        } else {
            holders.remove(p_reg);
            rentors.remove(&holder);
            spill_stack_map
        }
        .get(&holder)
        .unwrap()
        .get_pos();
        let split_to_stack_inst = LIRInst::build_storetostack_inst(p_reg, pos);
        config::record_spill("", "", format!("{}", split_to_stack_inst).as_str());
        new_insts.push(pool.put_inst(split_to_stack_inst));
        debug_assert!(!holders.contains_key(p_reg));
    }

    ///如果下次rentor是def,则直接丢弃rentor
    fn refresh_rentors_and_holders_with_next_occur(
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        next_occurs: &HashMap<Reg, LinkedList<(usize, bool)>>,
    ) {
        // 对于物理寄存器的持有者进行分析
        let to_process: Vec<Reg> = holders.iter().map(|(_, holder)| *holder).collect();
        // 如果持有者之后不出现或者下次不def,就可以结束持有
        for holder in to_process {
            if !next_occurs.contains_key(&holder)
                || (next_occurs.get(&holder).unwrap().front().unwrap().1)
            {
                if holder.is_physic() {
                    debug_assert!(!rentors.contains_key(&holder));
                    holders.remove(&holder);
                } else {
                    let reg_holded = rentors.remove(&holder).unwrap();
                    holders.remove(&reg_holded);
                }
            }
        }
    }
}

impl Func {
    ///choose last
    fn choose_borrow_1(
        rentor: &Reg,
        next_occurs: &HashMap<Reg, LinkedList<(usize, bool)>>,
        rentors: &HashMap<Reg, Reg>,
        holders: &HashMap<Reg, Reg>,
        availables: RegUsedStat,
    ) -> Reg {
        let mut availables = availables;
        availables.merge(&RegUsedStat::init_for_reg(rentor.get_type()));

        //获取自己的下次出现,如果自己没有下次出现,则 (use 完就结束了,也就是最短使用了,也就随便选一个能用的寄存器就最好)
        let self_next_occur = next_occurs.get(rentor);
        let self_last_occur = if self_next_occur.is_none() {
            0
        } else {
            self_next_occur.unwrap().back().unwrap().0
        };
        let mut choices: Vec<(Reg, usize, bool)> = Vec::new();
        //然后建立可用寄存器列表
        for reg in Reg::get_all_not_specials() {
            if !availables.is_available_reg(reg.get_color()) {
                continue;
            }
            let old_holder = holders.get(&reg);
            let old_holder = if old_holder.is_some() {
                old_holder.unwrap()
            } else {
                &reg
            };
            let next_occur = next_occurs.get(old_holder);
            //如果next occur下次没有出现(则可以直接给他一个最大值,也就是自己的出现位置+2)
            let (index, if_def) = if next_occur.is_none() {
                (self_last_occur + 10000, true)
            } else {
                next_occur.unwrap().front().unwrap().clone()
            };
            choices.push((*old_holder, index, if_def));
        }

        //对 order 进行排序
        choices.sort_by_key(|item| item.1);

        //优先选择刚好下一次出现在自己最后一次出现后的一个
        //首先寻找第一个下标大于自身的下次def自由寄存器,如果有自由寄存器,优先自由寄存器 (代价1)
        // 然后寻找第一个下标大于自身的下次use自由寄存器,如果有自由寄存器,优先自由寄存器 (代价2)
        //然后寻找第一个下次出现大于自身的下次use 可抢寄存器,代价3
        //最后最大下标寄存器
        let mut first_free_def: Option<Reg> = None;
        let mut first_free_use: Option<Reg> = None;
        let mut first_borrowable_use: Option<Reg> = None;
        for (reg, _, if_def) in choices.iter().filter(|item| item.1 > self_last_occur) {
            if reg.is_physic() && *if_def && first_free_def.is_none() {
                first_free_def = Some(*reg);
                break;
            }
        }
        for (reg, _, if_def) in choices.iter().filter(|item| item.1 > self_last_occur) {
            if reg.is_physic() && !*if_def && first_free_use.is_none() {
                first_free_use = Some(*reg);
                break;
            }
        }
        for (reg, _, if_def) in choices.iter().filter(|item| item.1 > self_last_occur) {
            if !reg.is_physic() && first_borrowable_use.is_none() {
                debug_assert!(!*if_def);
                first_borrowable_use = Some(*reg);
                break;
            }
        }

        if let Some(reg) = first_free_def {
            log_file!("1.txt", "1");
            return reg;
        } else if let Some(reg) = first_free_use {
            log_file!("2.txt", "2");
            return reg;
        } else if let Some(rentor) = first_borrowable_use {
            log_file!("3.txt", "3");
            return *rentors.get(&rentor).unwrap();
        }
        log_file!("4.txt", "4");

        //获取该虚拟寄存器的下一次出现
        debug_assert!(choices.len() != 0);
        let to_borrow_from = choices.last().unwrap().0;
        let to_borrow = if to_borrow_from.is_physic() {
            to_borrow_from
        } else {
            rentors.get(&to_borrow_from).unwrap().clone()
        };
        debug_assert!(to_borrow.is_physic());
        to_borrow
    }

    ///只选择临时寄存器
    fn choose_borrow_2(
        rentor: &Reg,
        next_occurs: &HashMap<Reg, LinkedList<(usize, bool)>>,
        rentors: &HashMap<Reg, Reg>,
        holders: &HashMap<Reg, Reg>,
        availables: RegUsedStat,
    ) -> Reg {
        let mut availables = availables;
        availables.merge(&RegUsedStat::init_for_reg(rentor.get_type()));
        for reg in Reg::get_all_tmps() {
            if availables.is_available_reg(reg.get_color()) {
                return reg;
            }
        }
        debug_assert!(false);
        let color = availables.get_available_reg(rentor.get_type()).unwrap();
        Reg::from_color(color)
    }
}
