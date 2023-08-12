use super::*;

/// handle spill v3实现
impl Func {
    ///为handle spill 计算寄存器活跃区间
    /// 会认为zero,ra,sp,tp,gp在所有块中始终活跃
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
    pub fn handle_spill_v3(&mut self, pool: &mut BackendPool) {
        self.calc_live_for_handle_spill();
        //先分配空间
        //对于spillings用到的空间直接一人一个
        let regs = self.draw_all_virtual_regs();
        for spilling_reg in regs.iter() {
            let spilling_reg = &spilling_reg.get_id();
            debug_assert!(
                regs.contains(&Reg::new(*spilling_reg, ScalarType::Int))
                    || regs.contains(&Reg::new(*spilling_reg, ScalarType::Float))
            );
            let last = self.stack_addr.back().unwrap();
            let new_pos = last.get_pos() + last.get_size();
            let new_stack_slot = StackSlot::new(new_pos, ADDR_SIZE);
            let spilling_reg = if regs.contains(&Reg::new(*spilling_reg, ScalarType::Int)) {
                debug_assert!(!regs.contains(&Reg::new(*spilling_reg, ScalarType::Float)));
                Reg::new(*spilling_reg, ScalarType::Int)
            } else {
                debug_assert!(regs.contains(&Reg::new(*spilling_reg, ScalarType::Float)));
                Reg::new(*spilling_reg, ScalarType::Float)
            };
            debug_assert!(!self.spill_stack_map.contains_key(&spilling_reg));
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

        Func::print_func(ObjPtr::new(&self), "before_handle_spill.txt");
        // debug_assert!();
        let to_process = self.blocks.iter().cloned().collect::<Vec<ObjPtr<BB>>>();
        // Func::print_func(ObjPtr::new(&self), "before_handle_spill.txt");
        for bb in to_process.iter() {
            if bb.insts.len() == 0 {
                continue;
            }
            self.handle_spill_for_block(bb, pool);
        }
        // self.remove_inst_suf_spill(pool);
        Func::print_func(ObjPtr::new(&self), "after_handle_spill.txt");
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

    ///在handle spill之后调用
    /// 返回该函数使用了哪些callee saved的寄存器
    pub fn draw_used_callees(&self) -> HashSet<Reg> {
        let mut callees: HashSet<Reg> = HashSet::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_callee_save() {
                        callees.insert(reg);
                    }
                }
            }
        }
        callees
    }

    /// 该函数应该在vtop之后调用
    /// 获取该函数使用到的caller save寄存器
    pub fn draw_used_callers(&self) -> HashSet<Reg> {
        let mut callers: HashSet<Reg> = HashSet::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_caller_save() {
                        callers.insert(reg);
                    }
                }
            }
        }
        callers
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
    //不需要给物理寄存器分配空间,因为每个块中都会为物理寄存器临时分配空间
    pub fn handle_spill_for_block(&mut self, bb: &ObjPtr<BB>, pool: &mut BackendPool) {
        let mut phisic_mems = HashMap::new();
        for reg in Reg::get_all_regs() {
            let back = self.stack_addr.back().unwrap();
            let new_pos = back.get_pos() + back.get_size();
            let new_sst = StackSlot::new(new_pos, ADDR_SIZE);
            self.stack_addr.push_back(new_sst);
            phisic_mems.insert(reg, new_sst);
        }
        //
        let spill_stack_map = &self.spill_stack_map;
        let mut next_occurs = Func::build_next_occurs(bb);

        let choose_borrow = Func::choose_borrow;
        let borrow = Func::borrow;
        let refresh_rentors_and_holders_with_next_occur =
            Func::refresh_rentors_and_holders_with_next_occur;

        let mut new_insts = Vec::new();
        let mut rentors: HashMap<Reg, Reg> = HashMap::new();
        let mut holders: HashMap<Reg, Reg> = HashMap::new();
        bb.live_in
            .iter()
            .filter(|reg| reg.is_physic())
            .for_each(|reg| {
                holders.insert(*reg, *reg);
            });
        //遇到结尾分支跳转语句前的处理
        let mut process_one = |index: usize, inst: &ObjPtr<LIRInst>| {
            //首先根据当前下标更新next occurs
            Func::refresh_next_occurs(&mut next_occurs, index);
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
                            Func::return_reg(holder, reg, spill_stack_map, pool, &mut new_insts);
                            Func::load_back(reg, &phisic_mems, pool, &mut new_insts);
                            rentors.remove(holder);
                            holders.insert(*reg, *reg);
                        }
                    }
                    _ => {
                        Func::load_back(reg, &phisic_mems, pool, &mut new_insts);
                        holders.insert(*reg, *reg);
                    }
                }
            }
            for reg in defed.iter().filter(|reg| reg.is_physic()) {
                match holders.get(reg) {
                    Some(holder) => {
                        let holder = *holder;
                        let holder = &holder;
                        if holder != reg {
                            //物理寄存器不在持有者手上,则需要进行归还
                            Func::return_reg(holder, reg, spill_stack_map, pool, &mut new_insts);
                            rentors.remove(holder);
                        }
                    }
                    _ => (),
                }
                holders.insert(*reg, *reg);
            }

            let mut availables: RegUsedStat = RegUsedStat::init_unspecial_regs_without_s0();
            let mut regs = inst.get_regs();
            //记录不能够使用的寄存器
            regs.retain(|reg| !reg.is_physic());
            for reg in regs.iter() {
                if let Some(borrowed) = rentors.get(reg) {
                    availables.use_reg(borrowed.get_color());
                } else if reg.is_physic() {
                    availables.use_reg(reg.get_color());
                }
            }

            //选择并租借物理寄存器
            for reg in regs.iter() {
                if rentors.contains_key(&reg) {
                    continue;
                }
                let to_borrow = choose_borrow(reg, &next_occurs, &rentors, &holders, availables);
                availables.use_reg(to_borrow.get_color());
                borrow(
                    reg,
                    &to_borrow,
                    &mut rentors,
                    &mut holders,
                    spill_stack_map,
                    &phisic_mems,
                    pool,
                    &mut new_insts,
                );
            }
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
            refresh_rentors_and_holders_with_next_occur(&mut rentors, &mut holders, &next_occurs);
            new_insts.push(*inst);
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

        //归还所有物理寄存器
        let to_give_back: Vec<Reg> = rentors.iter().map(|(_, r)| *r).collect();
        for reg in to_give_back {
            let rentor = *holders.get(&reg).unwrap();
            if bb.live_out.contains(&rentor) {
                Func::return_reg(&rentor, &reg, spill_stack_map, pool, &mut new_insts);
            }
            //根据物理寄存器是否在live out 中判断是否要加载回来
            if bb.live_out.contains(&reg) {
                Func::load_back(&reg, &phisic_mems, pool, &mut new_insts);
            }
            rentors.remove(&rentor);
            holders.insert(reg, reg);
        }

        // println!("{index}");
        //对结尾跳转语句的处理(使用临时寄存器,不在live out里面的内容不去保存)
        while index < bb.insts.len() {
            let inst = bb.insts.get(index).unwrap();
            match inst.get_type() {
                InstrsType::Branch(_) | InstrsType::Jump => (),
                _ => {
                    debug_assert!(
                        inst.get_regs()
                            .iter()
                            .filter(|reg| !reg.is_physic())
                            .count()
                            == 0
                    );
                    new_insts.push(*inst);
                    index += 1;
                    continue;
                }
            };
            //使用临时寄存器作为借用
            let mut regs = inst.get_regs();
            let mut availables = RegUsedStat::init_unavailable();
            for reg in Reg::get_all_tmps() {
                availables.release_reg(reg.get_color());
            }
            regs.retain(|reg| !reg.is_physic());
            for reg in regs {
                let available = availables.get_available_reg(reg.get_type()).unwrap();
                availables.use_reg(available);
                let tmp_reg = Reg::from_color(available);
                Func::borrow(
                    &reg,
                    &tmp_reg,
                    &mut rentors,
                    &mut holders,
                    spill_stack_map,
                    &phisic_mems,
                    pool,
                    &mut new_insts,
                );
                inst.as_mut().replace_reg(&reg, &tmp_reg);
            }
            //borrow结束后rentor中的寄存器应该都是临时寄存器
            for (_, br) in rentors.iter() {
                debug_assert!(Reg::get_all_tmps().contains(br));
            }
            rentors.clear();

            new_insts.push(*inst);
            index += 1;
        }

        bb.as_mut().insts = new_insts;
    }

    fn refresh_next_occurs(
        next_occurs: &mut HashMap<Reg, LinkedList<(usize, bool)>>,
        cur_index: usize,
    ) {
        let mut to_free = Vec::new();
        for (reg, next_occurs) in next_occurs.iter_mut() {
            while !next_occurs.is_empty() && next_occurs.front().unwrap().0 <= cur_index {
                next_occurs.pop_front();
            }
            if next_occurs.len() == 0 {
                to_free.push(*reg);
            }
        }
        for reg in to_free {
            next_occurs.remove(&reg);
        }
    }

    ///建立下次出现表,依赖于上次calc live的结果
    fn build_next_occurs(bb: &BB) -> HashMap<Reg, LinkedList<(usize, bool)>> {
        let mut next_occurs: HashMap<Reg, LinkedList<(usize, bool)>> = HashMap::new();
        //初始化holder
        bb.live_in.iter().for_each(|reg| {
            next_occurs.insert(*reg, LinkedList::new());
        });
        // 维护一个物理寄存器的作用区间队列,每次的def和use压入栈中 (先压入use,再压入def)
        // 每个链表元素为(reg,if_def)
        for (index, inst) in bb.insts.iter().enumerate() {
            for reg in inst.get_reg_use() {
                if !next_occurs.contains_key(&reg) {
                    next_occurs.insert(reg, LinkedList::new());
                }
                next_occurs.get_mut(&reg).unwrap().push_back((index, false));
            }
            for reg in inst.get_reg_def() {
                if !next_occurs.contains_key(&reg) {
                    next_occurs.insert(reg, LinkedList::new());
                }
                next_occurs.get_mut(&reg).unwrap().push_back((index, true));
            }
        }
        bb.live_out.iter().for_each(|reg| {
            next_occurs
                .get_mut(reg)
                .unwrap()
                .push_back((bb.insts.len(), false));
        });
        for (_, b) in next_occurs.iter() {
            debug_assert!(b.len() >= 1);
        }
        next_occurs
    }

    fn return_reg(
        rentor: &Reg,
        owner: &Reg,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        //
        debug_assert!(!rentor.is_physic());
        //先把rentor的值存回栈上
        let pos = spill_stack_map.get(rentor).unwrap().get_pos();
        let store_inst = LIRInst::build_storetostack_inst(owner, pos);
        new_insts.push(pool.put_inst(store_inst));
        config::record_spill(
            "",
            "",
            format!("把{}值从{}存回栈{}上", rentor, owner, pos).as_str(),
        );
    }

    ///加载回物理寄存器的原值
    fn load_back(
        p_reg: &Reg,
        phisic_mems: &HashMap<Reg, StackSlot>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        let pos = phisic_mems.get(p_reg).unwrap().get_pos();
        let load_inst = LIRInst::build_loadstack_inst(p_reg, pos);
        new_insts.push(pool.put_inst(load_inst));
        config::record_spill(
            "",
            "",
            format!("从栈{}上加载回物理寄存器{}原值", pos, p_reg).as_str(),
        );
    }

    ///choose last
    fn choose_borrow(
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
        if self_next_occur.is_none() {
            let to_borrow = availables.get_available_reg(rentor.get_type()).unwrap();
            let to_borrow = Reg::from_color(to_borrow);
            return to_borrow;
        }

        //如果自己有下次出现,记录自己最后一次出现的下标
        let self_last_occur = self_next_occur.unwrap().back().unwrap().0;
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
                (self_last_occur * 2 + 1, true)
            } else {
                next_occur.unwrap().front().unwrap().clone()
            };
            //因为def的情况代价更小更适合选,所以相同前置的情况下先设置为1,
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

    ///如果下次rentor是def,则直接丢弃rentor
    fn refresh_rentors_and_holders_with_next_occur(
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        next_occurs: &HashMap<Reg, LinkedList<(usize, bool)>>,
    ) {
        let mut to_process = Vec::new();
        for (holder, reg) in holders.iter() {
            if holder == reg {
                to_process.push(*reg);
            }
        }
        for reg in to_process {
            debug_assert!(holders.get(&reg).unwrap() == &reg);
            let next_occur = next_occurs.get(&reg);
            //说明这个物理寄存器之后不再出现了,说明可以抛弃
            if next_occur.is_none() {
                holders.remove(&reg);
                continue;
            }
            let then_def = next_occur.unwrap().front().unwrap().1;
            if then_def {
                holders.remove(&reg);
            }
        }

        let to_process: Vec<Reg> = rentors.iter().map(|(r, _)| *r).collect();
        for rentor in to_process {
            let reg_rent = *rentors.get(&rentor).unwrap();
            let next_occur = next_occurs.get(&rentor);
            if next_occur.is_none() {
                rentors.remove(&rentor);
                holders.remove(&reg_rent);
                continue;
            }
            let then_def = next_occur.unwrap().front().unwrap().1;
            if then_def {
                rentors.remove(&rentor);
                holders.remove(&reg_rent);
            }
        }
    }

    fn borrow(
        rentor: &Reg,
        to_borrow: &Reg,
        rentors: &mut HashMap<Reg, Reg>,
        holders: &mut HashMap<Reg, Reg>,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        phisic_mems: &HashMap<Reg, StackSlot>,
        pool: &mut BackendPool,
        new_insts: &mut Vec<ObjPtr<LIRInst>>,
    ) {
        //如果要借用的物理寄存器有持有者,则把持有者的值保存到对应栈上
        if let Some(holder) = holders.get(to_borrow) {
            //判断下次使用是读还是写,如果是写的话,就不用保存
            let pos = if holder.is_physic() {
                phisic_mems.get(holder).unwrap().get_pos()
            } else {
                spill_stack_map.get(holder).unwrap().get_pos()
            };
            config::record_spill(
                "",
                "",
                format!("把{}值暂存到栈{}是上", holder, pos).as_str(),
            );
            let store_inst = LIRInst::build_storetostack_inst(to_borrow, pos);
            new_insts.push(pool.put_inst(store_inst));
            rentors.remove(holder);
        }
        //借用物理寄存器 (先把物理寄存器原值保存到栈上)
        let pos = spill_stack_map.get(rentor).unwrap().get_pos();
        let load_inst = LIRInst::build_loadstack_inst(to_borrow, pos);
        new_insts.push(pool.put_inst(load_inst));
        config::record_spill(
            "",
            "",
            format!("从栈{}加载{}的值到{}上", pos, rentor, to_borrow).as_str(),
        );
        rentors.insert(*rentor, *to_borrow);
        holders.insert(*to_borrow, *rentor);
    }
}
