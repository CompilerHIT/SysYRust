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
        let this = pool.put_func(self.clone());
        for bb in self.blocks.iter() {
            bb.as_mut().handle_spill_v3(this, pool);
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
