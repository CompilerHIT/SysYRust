use super::*;

/// handle spill2: handle spill过程中对spill寄存器用到的栈进行重排
/// 当前func的spill不能够与v1的spill完美替换
impl Func {
    /// 为spilling 寄存器预先分配空间 的 handle spill
    pub fn handle_spill_v2(&mut self, pool: &mut BackendPool) {
        // 首先给这个函数分配spill的空间
        self.calc_live_for_handle_spill();
        self.assign_stack_slot_for_spill();
        let this = pool.put_func(self.clone());
        for block in self.blocks.iter() {
            block
                .as_mut()
                .handle_spill_v2(this, &self.reg_alloc_info.spillings, pool);
        }
    }

    /// 为了分配spill的虚拟寄存器所需的栈空间使用的而构建冲突图
    fn build_interferench_for_assign_stack_slot_for_spill(&mut self) -> HashMap<Reg, HashSet<Reg>> {
        let mut out: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        self.calc_live_for_alloc_reg();
        for bb in self.blocks.iter() {
            //
            let bb = *bb;
            let mut live_now: HashSet<Reg> = HashSet::new();
            for reg in bb.live_out.iter() {
                if !self.reg_alloc_info.spillings.contains(&reg.get_id()) {
                    continue;
                }
                if !out.contains_key(reg) {
                    out.insert(*reg, HashSet::new());
                }
                for live in live_now.iter() {
                    if live == reg {
                        continue;
                    }
                    out.get_mut(live).unwrap().insert(*reg);
                    out.get_mut(reg).unwrap().insert(*live);
                }
                live_now.insert(*reg);
            }
            for inst in bb.insts.iter().rev() {
                for reg in inst.get_reg_def() {
                    live_now.remove(&reg);
                }
                for reg in inst.get_reg_use() {
                    if !self.reg_alloc_info.spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    if !out.contains_key(&reg) {
                        out.insert(reg, HashSet::new());
                    }
                    for live in live_now.iter() {
                        if live == &reg {
                            continue;
                        }
                        out.get_mut(live).unwrap().insert(reg);
                        out.get_mut(&reg).unwrap().insert(*live);
                    }
                    live_now.insert(reg);
                }
            }
        }
        out
    }

    /// 分析spill空间之间的冲突关系,进行紧缩
    fn assign_stack_slot_for_spill(&mut self) {
        let path = "assign_mem.txt";

        // 给spill的寄存器空间,如果出现重复的情况,则说明后端可能空间存在冲突
        // 建立spill寄存器之间的冲突关系(如果两个spill的寄存器之间是相互冲突的,则它们不能够共享相同内存)
        let mut spill_coes: HashMap<i32, i32> = HashMap::new();
        let mut id_to_regs: HashMap<i32, Reg> = HashMap::new();
        let spillings = &self.reg_alloc_info.spillings;
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_reg_use() {
                    if reg.is_physic() {
                        continue;
                    }
                    if !spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    id_to_regs.insert(reg.get_id(), reg);
                    spill_coes.insert(
                        reg.get_id(),
                        spill_coes.get(&reg.get_id()).unwrap_or(&0) + 1,
                    );
                }
                for reg in inst.get_reg_def() {
                    if reg.is_physic() {
                        continue;
                    }
                    if !spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    id_to_regs.insert(reg.get_id(), reg);
                    spill_coes.insert(
                        reg.get_id(),
                        spill_coes.get(&reg.get_id()).unwrap_or(&0) + 1,
                    );
                }
            }
        }
        // 桶排序
        let mut buckets: HashMap<i32, LinkedList<Reg>> = HashMap::new();
        let mut coe_orders: BiHeap<i32> = BiHeap::new();
        for id in spillings {
            debug_assert!(spill_coes.contains_key(id) && id_to_regs.contains_key(id));
            let coe = spill_coes.get(id).unwrap();
            let reg = id_to_regs.get(id).unwrap();
            if !buckets.contains_key(coe) {
                coe_orders.push(*coe);
                buckets.insert(*coe, LinkedList::new());
            }
            buckets.get_mut(coe).unwrap().push_back(*reg);
        }
        log_file!(path, "{:?}", spillings);
        // 使用一个表记录之前使用过的空间,每次分配空间的时候可以复用之前使用过的空间,只要没有冲突
        // 如果有冲突则 需要开辟新的空间
        let mut slots: LinkedList<StackSlot> = LinkedList::new();
        let inter_graph: HashMap<Reg, HashSet<Reg>> =
            self.build_interferench_for_assign_stack_slot_for_spill();
        // 优先给使用次数最多的spill寄存器分配内存空间
        while !coe_orders.is_empty() {
            let spill_coe = coe_orders.pop_max().unwrap();
            let lst = buckets.get_mut(&spill_coe).unwrap();
            while !lst.is_empty() {
                let toassign = lst.pop_front().unwrap();
                log_file!(path, "assign:{}", toassign);
                if self.spill_stack_map.contains_key(&toassign) {
                    unreachable!()
                }
                // 首先在已经分配的空间里面寻找可复用的空间
                // 首先记录冲突的空间
                let mut inter_slots: HashSet<StackSlot> = HashSet::new();
                for reg in inter_graph.get(&toassign).unwrap() {
                    if !self.spill_stack_map.contains_key(reg) {
                        continue;
                    }
                    let stack_slot = self.spill_stack_map.get(reg).unwrap();
                    inter_slots.insert(*stack_slot);
                }

                // 然后遍历已经分配的空间,寻找到第一个可以分配的空间
                let mut num = slots.len();
                let mut slot_for_toassign: Option<StackSlot> = Option::None;
                while num > 0 {
                    num -= 1;
                    let old_slot = slots.pop_front().unwrap();
                    slots.push_back(old_slot);
                    if inter_slots.contains(&old_slot) {
                        continue;
                    }
                    log_file!(path, "reuse one times!,{}-{:?}", toassign, old_slot);
                    slot_for_toassign = Some(old_slot);
                    break;
                }
                if slot_for_toassign.is_none() {
                    let last_slot = self.stack_addr.back().unwrap();
                    let pos = last_slot.get_pos() + last_slot.get_size();
                    let stack_slot = StackSlot::new(pos, ADDR_SIZE);
                    self.stack_addr.push_back(stack_slot);
                    slot_for_toassign = Some(stack_slot);
                    slots.push_back(stack_slot);
                }
                self.spill_stack_map
                    .insert(toassign, slot_for_toassign.unwrap());
            }
        }

        log!("func:{}\n{:?}", self.label, self.spill_stack_map);
    }
}

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
        self.build_reg_intervals();
        //先分配空间
        self.assign_stack_slot_for_spill();

        //为物理寄存器相关的借还开辟空间
        let mut phisic_mems = HashMap::new();
        for reg in Reg::get_all_not_specials() {
            if !reg.is_special() {
                let last = self.stack_addr.back().unwrap();
                let new_pos = last.get_pos() + last.get_size();
                let new_stack_slot = StackSlot::new(new_pos, ADDR_SIZE);
                self.stack_addr.push_back(new_stack_slot);
                phisic_mems.insert(reg, new_stack_slot);
            }
        }

        for bb in self.blocks.iter() {
            Func::handle_spill_of_block(
                bb,
                pool,
                &self.reg_alloc_info.spillings,
                &self.spill_stack_map,
                &phisic_mems,
            );
        }
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

impl Func {
    ///考虑有临时寄存器可以用
    /// 该操作应该在p2v之后进行,认为遇到的虚拟寄存器都是临时寄存器
    fn handle_spill_of_block(
        bb: &ObjPtr<BB>,
        pool: &mut BackendPool,
        spillings: &HashSet<i32>,
        spill_stack_map: &HashMap<Reg, StackSlot>,
        phisic_mem: &HashMap<Reg, StackSlot>,
    ) {
        //优先使用临时寄存器,然后使用其他空余寄存器
        //使用越密集的寄存器倾向于分配更远的虚拟寄存器
        //抽象 （index,v_reg) 获取一个物理寄存器造成的代价
        // (index,v_reg,p_reg)
        //遇到spllings的时候选择一个归还日期最接近该spilling寄存器的块内终结日期的寄存器
        //ps 理论上一个块内一个寄存器只可能存在一次定义,但是可能存在若干次使用
        let mut ends_of_spilling_reg_in_block: HashMap<Reg, ObjPtr<LIRInst>> = HashMap::new();
        for inst in bb.insts.iter().rev() {
            for reg in inst.get_reg_use() {
                if !reg.is_physic() {
                    debug_assert!(spillings.contains(&reg.get_id()));
                    if ends_of_spilling_reg_in_block.contains_key(&reg) {
                        continue;
                    }
                    ends_of_spilling_reg_in_block.insert(reg, *inst);
                }
            }
        }

        //维护一个表,记录当前各个物理寄存器的持有者
        let mut next_occurs: HashMap<Reg, LinkedList<(usize, bool)>> = HashMap::new();
        let mut holders: HashMap<Reg, Reg> = HashMap::new();
        //初始化holder
        bb.live_in.iter().for_each(|reg| {
            if reg.is_physic() {
                holders.insert(*reg, *reg);
                next_occurs.insert(*reg, LinkedList::new());
            } else {
                //对于虚拟寄存器先不给它们分配要用的物理寄存器,等到要借的时候再分配
                next_occurs.insert(*reg, LinkedList::new());
            }
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
        //对于其他的没有加入到表中的寄存器,也添加列表
        for reg in Reg::get_all_specials() {
            if next_occurs.contains_key(&reg) {
                continue;
            }
            next_occurs.insert(reg, LinkedList::new());
        }

        //然后对于不在live out中的但是insts中出现的所有寄存器,直接全部加上一个超长长度,
        for (_, next_occur) in next_occurs.iter_mut() {
            next_occur.push_back((bb.insts.len() * 2, false));
        }

        //准备用于进行选择选择要借用的寄存器的函数
        let choose_borrow = |inst: &ObjPtr<LIRInst>,
                             rentor: &Reg,
                             next_occurs: &mut HashMap<Reg, LinkedList<(usize, bool)>>,
                             rentors: &HashMap<Reg, Reg>,
                             holders: &HashMap<Reg, Reg>|
         -> Reg {
            debug_assert!(!rentors.contains_key(rentor));
            //获取所有非特殊寄存器
            let mut regs = RegUsedStat::init_unspecial_regs();
            //然后禁止当前指令使用到的物理寄存器,以及当前指令涉及的虚拟寄存器使用到的寄存器
            for reg in inst.get_regs() {
                if reg.is_physic() {
                    regs.use_reg(reg.get_color())
                } else if let Some(br) = rentors.get(&reg) {
                    regs.use_reg(br.get_color());
                }
            }

            //禁止种类不同的寄存器
            regs.merge(&RegUsedStat::init_for_reg(rentor.get_type()));
            let mut choices: Vec<(Reg, usize)> = Vec::new();
            //然后建立可用寄存器列表
            for reg in Reg::get_all_not_specials() {
                if !regs.is_available_reg(reg.get_color()) {
                    continue;
                }
                let old_holder = holders.get(&reg);
                let old_holder = if old_holder.is_some() {
                    old_holder.unwrap()
                } else {
                    &reg
                };
                let next_occur = next_occurs.get(old_holder).unwrap().front().unwrap();
                let (index, if_def) = next_occur;
                //因为def的情况代价更小更适合选,所以相同前置的情况下先设置为1,
                let next_occur = index << 1 | (if *if_def { 1 } else { 0 });
                choices.push((*old_holder, next_occur));
            }
            //对 order 进行排序
            choices.sort_by_key(|item| item.1);
            //获取该虚拟寄存器的下一次出现
            debug_assert!(choices.len() != 0);
            let to_borrow_from = choices.get(0).unwrap().0;
            if to_borrow_from.is_physic() {
                to_borrow_from
            } else {
                rentors.get(&to_borrow_from).unwrap().clone()
            }
        };
        let borrow = |rentor: &Reg,
                      borrowed: &Reg,
                      inst: &ObjPtr<LIRInst>,
                      next_occurs: &mut HashMap<Reg, LinkedList<(usize, bool)>>,
                      rentors: &mut HashMap<Reg, Reg>,
                      holders: &mut HashMap<Reg, Reg>,
                      pool: &mut BackendPool,
                      new_insts: &mut Vec<ObjPtr<LIRInst>>| {
            //首先判断是否需要进行寄存器的归还
            match holders.get(borrowed) {
                Some(holder) => {
                    if !holder.is_physic() {
                        //判断是否需要把该寄存器的值还回去
                        let if_turn_back = next_occurs.get(holder).unwrap().front().unwrap().1;
                        if if_turn_back {
                            let pos = spill_stack_map.get(holder).unwrap().get_pos();
                            let back_inst = LIRInst::build_storetostack_inst(borrowed, pos);
                            new_insts.push(pool.put_inst(back_inst));
                        }
                        rentors.remove(holder);
                        holders.remove(borrowed);
                    }
                }
                None => (),
            };
            //然后判断是否需要拿回rentor寄存器原本的值
            //需要
            if inst.get_reg_use().contains(rentor) {
                let pos = spill_stack_map.get(rentor).unwrap().get_pos();
                let load_back_inst = LIRInst::build_loadstack_inst(borrowed, pos);
                new_insts.push(pool.put_inst(load_back_inst));
            }
            //修改 rent hold表
            holders.insert(*borrowed, *rentor);
            rentors.insert(*rentor, *borrowed);
        };
        //寄存器归还逻辑
        let return_reg = |rentor: &Reg,
                          borrowed: &Reg,
                          rentors: &mut HashMap<Reg, Reg>,
                          holders: &mut HashMap<Reg, Reg>,
                          pool: &mut BackendPool,
                          new_insts: &mut Vec<ObjPtr<LIRInst>>| {
            debug_assert!(spillings.contains(&rentor.get_id()));
            debug_assert!(rentors.get(rentor).unwrap() == borrowed);
            debug_assert!(holders.get(borrowed).unwrap() == rentor);
            let pos = spill_stack_map.get(rentor).unwrap().get_pos();
            //把spilling寄存器的值还回栈上
            let self_back_inst = LIRInst::build_storetostack_inst(&borrowed, pos);
            new_insts.push(pool.put_inst(self_back_inst));
            //把物理寄存器的值取回
            let owner_pos = phisic_mem.get(&borrowed).unwrap().get_pos();
            let return_inst = LIRInst::build_loadstack_inst(&borrowed, owner_pos);
            new_insts.push(pool.put_inst(return_inst));
            //更新rentor 和rentor的状态
            rentors.remove(rentor);
            holders.insert(*borrowed, *borrowed);
        };
        //归还物理寄存器的逻辑
        let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
        let mut rentors: HashMap<Reg, Reg> = HashMap::new();
        //正式分配流程,
        let mut index = 0;
        while index < bb.insts.len() {
            let inst = bb.insts.get(index).unwrap();
            //更新next occur表
            for reg in inst.get_regs() {
                let next_occur = next_occurs.get_mut(&reg).unwrap();
                while !next_occur.is_empty() {
                    let front = next_occur.front().unwrap();
                    if front.0 <= index {
                        next_occur.pop_front();
                        continue;
                    }
                    break;
                }
            }

            //先归还
            for reg in inst.get_regs() {
                //判断是否有需要归还的寄存器 (把值取回物理寄存器,此处需要一个物理寄存器相关的空间)
                if reg.is_physic() && holders.contains_key(&reg) {
                    //遇到的物理寄存器一定有持有者
                    let rentor = holders.get(&reg).unwrap();
                    if &reg != rentor {
                        //如果寄存器不在当前phisicreg 手上,则进行归还
                        //统一归还操作为归还到栈空间上,无用访存指令后面会删除
                        let rentor = *rentor;
                        return_reg(
                            &rentor,
                            &reg,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                }
            }

            //再租借
            for reg in inst.get_regs() {
                if !reg.is_physic() {
                    if !rentors.contains_key(&reg) {
                        let to_borrow =
                            choose_borrow(inst, &reg, &mut next_occurs, &rentors, &holders);
                        borrow(
                            &reg,
                            &to_borrow,
                            inst,
                            &mut next_occurs,
                            &mut rentors,
                            &mut holders,
                            pool,
                            &mut new_insts,
                        );
                    }
                }
            }
            //判断是否有需要把值存回栈上的寄存器
            //然后对该指令进行寄存器替换
            for reg in inst.get_regs() {
                if !reg.is_physic() {
                    debug_assert!(spillings.contains(&reg.get_id()));
                    let borrowed = rentors.get(&reg).unwrap();
                    inst.as_mut().replace_reg(&reg, borrowed);
                }
            }
            //然后加入替换后的指令
            new_insts.push(*inst);

            //根据当前只有表进行试探归还
            let mut to_relase = Vec::new();
            for (p_reg, holder) in holders.iter() {
                let next_occur = next_occurs.get(holder).unwrap();
                if next_occur.front().unwrap().0 > bb.insts.len() {
                    to_relase.push((*holder, *p_reg));
                }
            }
            for (rentor, borrowed) in to_relase.iter() {
                holders.remove(borrowed);
                rentors.remove(rentor);
            }
            index += 1;
        }
        //在块的最后,判断是否有哪些寄存器还没有归还到主人手里,但是应该归还
        for (rentor, borrow) in rentors {
            //如果spillings寄存器值需要归还
            if bb.live_out.contains(&rentor) {
                let pos = spill_stack_map.get(&rentor).unwrap().get_pos();
                let return_inst = LIRInst::build_storetostack_inst(&borrow, pos);
                new_insts.push(pool.put_inst(return_inst));
            }
            //如果对应物理寄存器值应该取回
            if bb.live_out.contains(&borrow) {
                let pos = phisic_mem.get(&borrow).unwrap().get_pos();
                let get_back_inst = LIRInst::build_loadstack_inst(&borrow, pos);
                new_insts.push(pool.put_inst(get_back_inst));
            }
        }
        bb.as_mut().insts = new_insts;
        // unimplemented!()
    }
}
