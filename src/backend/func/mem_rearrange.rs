use super::*;

// rearrange slot实现 ,for module-build v3
impl Func {
    ///分析函数的栈空间的作用区间  (得到liveuse,livedef, live in,live out)
    /// 在handle overflow前使用,仅仅对于spill的指令进行分析
    pub fn calc_stackslot_interval(
        &self,
    ) -> (
        HashMap<ObjPtr<BB>, HashSet<StackSlot>>,
        HashMap<ObjPtr<BB>, HashSet<StackSlot>>,
        HashMap<ObjPtr<BB>, HashSet<StackSlot>>,
        HashMap<ObjPtr<BB>, HashSet<StackSlot>>,
    ) {
        //计算使用的内存地址的活跃区间
        let mut live_ins: HashMap<ObjPtr<BB>, HashSet<StackSlot>> = HashMap::new();
        let mut live_outs: HashMap<ObjPtr<BB>, HashSet<StackSlot>> = HashMap::new();
        let mut live_defs: HashMap<ObjPtr<BB>, HashSet<StackSlot>> = HashMap::new();
        let mut live_uses: HashMap<ObjPtr<BB>, HashSet<StackSlot>> = HashMap::new();

        //计算stackslot的 live use, live def
        for bb in self.blocks.iter() {
            let mut live_def: HashSet<StackSlot> = HashSet::new();
            let mut live_use: HashSet<StackSlot> = HashSet::new();
            for inst in bb.insts.iter().rev() {
                match inst.get_type() {
                    InstrsType::LoadFromStack => {
                        let offset = inst.get_stack_offset().get_data();
                        let stackslot = StackSlot::new(offset, ADDR_SIZE);
                        live_def.remove(&stackslot);
                        live_use.insert(stackslot);
                    }
                    InstrsType::StoreToStack => {
                        let offset = inst.get_stack_offset().get_data();
                        let stackslot = StackSlot::new(offset, ADDR_SIZE);

                        live_use.insert(stackslot);
                        live_def.remove(&stackslot);
                    }
                    _ => (),
                }
            }
            live_defs.insert(*bb, live_def);
            live_uses.insert(*bb, live_use.clone());
            live_ins.insert(*bb, live_use);
            live_outs.insert(*bb, HashSet::new());
        }
        //计算live in 和 live out
        loop {
            let mut finish_flag = true;
            // new in =  (old_out-def)+old_in
            // new out= [out_edge:uses]
            //更新live in
            for bb in self.blocks.iter() {
                let live_in = live_ins.get_mut(bb).unwrap();
                let def = live_defs.get(bb).unwrap();
                let mut new_in = live_outs.get(bb).unwrap().clone();
                new_in.retain(|sst| def.contains(sst));
                new_in.extend(live_in.iter());
                if new_in.len() > live_in.len() {
                    finish_flag = false;
                    *live_in = new_in;
                }
            }
            //更新 live out
            for bb in self.blocks.iter() {
                let live_out = live_outs.get_mut(bb).unwrap();
                let mut new_live_out = live_out.clone();
                for out_bb in bb.out_edge.iter() {
                    new_live_out.extend(live_ins.get(out_bb).unwrap().iter());
                }
                if new_live_out.len() > live_out.len() {
                    *live_out = new_live_out;
                    finish_flag = false;
                }
            }
            if finish_flag {
                break;
            }
        }

        (live_uses, live_defs, live_ins, live_outs)
    }
    ///分析函数用到的栈空间的冲突,传入
    pub fn calc_stackslot_interef_with_rearrangable_set(
        &mut self,
        rearrangables: &HashSet<StackSlot>,
        live_outs: &HashMap<ObjPtr<BB>, HashSet<StackSlot>>,
    ) -> HashMap<(StackSlot, StackSlot), i32> {
        let mut interef = HashMap::new();
        for bb in self.blocks.iter() {
            let live_out = live_outs.get(bb).unwrap();
            let mut live_now = HashSet::new();
            live_out.iter().for_each(|sst| {
                if rearrangables.contains(sst) {
                    live_now.insert(*sst);
                }
            });
            for inst in bb.insts.iter().rev() {
                match inst.get_type() {
                    InstrsType::LoadFromStack => {
                        let sst = inst.get_stackslot_with_addr_size();
                        if rearrangables.contains(&sst) && !live_now.contains(&sst) {
                            for live in live_now.iter() {
                                let key = if sst.get_pos() < live.get_pos() {
                                    (sst, *live)
                                } else {
                                    (*live, sst)
                                };
                                let new_times = interef.get(&key).unwrap_or(&0) + 1;
                                interef.insert(key, new_times);
                            }
                        }
                    }
                    InstrsType::StoreToStack => {
                        let sst = inst.get_stackslot_with_addr_size();
                        live_now.remove(&sst);
                    }
                    _ => continue,
                };
            }
        }
        //获取用于rearrange需要的栈空间
        interef
    }

    pub fn rearrange_stack_slot(&mut self) {
        let mut all_rearrangable_stackslot: HashSet<StackSlot> = HashSet::new();
        //首先统计所有的可重排地址
        for stackslot in self.stack_addr.iter().rev() {
            if stackslot.get_pos() == 0 {
                break;
            }
            all_rearrangable_stackslot.insert(*stackslot);
        }
        let (_, _, _, live_outs) = self.calc_stackslot_interval();
        let interef = self
            .calc_stackslot_interef_with_rearrangable_set(&all_rearrangable_stackslot, &live_outs);

        //根据interef统计stackslot的使用次数,
        //对于使用次数最多的stackslot,给它分配最新的stackslot
        let mut new_slots: LinkedList<StackSlot> = LinkedList::new();
        new_slots.push_back(StackSlot::new(0, 0));

        return;
        //定位使用到的栈空间(计算它们之间的依赖关系)

        //分析栈空间的读写的传递
    }
}
