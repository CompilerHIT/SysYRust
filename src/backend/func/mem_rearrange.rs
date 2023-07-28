use core::time;

use super::*;

// rearrange slot实现 ,for module-build v3
impl Func {
    ///分析函数的栈空间的作用区间  (得到liveuse,livedef, live in,live out)
    /// 在handle overflow前使用,仅仅对于spill的指令进行分析
    pub fn calc_stackslot_interval(
        func: &Func,
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
        for bb in func.blocks.iter() {
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
            for bb in func.blocks.iter() {
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
            for bb in func.blocks.iter() {
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
        func: &Func,
        rearrangables: &HashSet<StackSlot>,
        live_outs: &HashMap<ObjPtr<BB>, HashSet<StackSlot>>,
    ) -> HashMap<StackSlot, HashSet<StackSlot>> {
        let mut interef = HashMap::new();
        for bb in func.blocks.iter() {
            let live_out = live_outs.get(bb).unwrap();
            let mut live_now = HashSet::new();
            live_out.iter().for_each(|sst| {
                if rearrangables.contains(sst) {
                    if !interef.contains_key(sst) {
                        interef.insert(*sst, HashSet::new());
                    }
                    live_now.insert(*sst);
                }
            });
            for inst in bb.insts.iter().rev() {
                match inst.get_type() {
                    InstrsType::LoadFromStack => {
                        let sst = inst.get_stackslot_with_addr_size();

                        if rearrangables.contains(&sst) && !live_now.contains(&sst) {
                            if !interef.contains_key(&sst) {
                                interef.insert(sst, HashSet::new());
                            }
                            for live in live_now.iter() {
                                interef.get_mut(&sst).unwrap().insert(*live);
                                interef.get_mut(live).unwrap().insert(sst);
                            }
                            live_now.insert(sst);
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
    ///统计指定栈空间使用次数
    pub fn rearrange_stack_slot(&mut self) {
        let mut all_rearrangable_stackslot: HashSet<StackSlot> = HashSet::new();
        //首先统计所有的可重排地址
        while self.stack_addr.back().unwrap().get_pos() != 0 {
            all_rearrangable_stackslot.insert(self.stack_addr.pop_back().unwrap());
        }
        let (_, _, _, live_outs) = Func::calc_stackslot_interval(self);

        let interef = Func::calc_stackslot_interef_with_rearrangable_set(
            self,
            &all_rearrangable_stackslot,
            &live_outs,
        );
        let mut times: HashMap<StackSlot, i32> = HashMap::new();
        //分析虚拟栈单元使用次数
        analyse_each_inst(&self, &mut |inst| match inst.get_type() {
            InstrsType::LoadFromStack | InstrsType::StoreToStack => {
                let sst = inst.get_stackslot_with_addr_size();
                if all_rearrangable_stackslot.contains(&sst) {
                    let new_times = times.get(&sst).unwrap_or(&0) + 1;
                    times.insert(sst, new_times);
                }
            }
            _ => (),
        });
        //对于虚拟栈单元,按照使用次数从高到低进行排序
        let mut ordered_ssts: Vec<StackSlot> = all_rearrangable_stackslot.iter().cloned().collect();
        ordered_ssts.sort_by_cached_key(|sst| -times.get(sst).unwrap());
        //进行重分配
        self.spill_stack_map.clear();

        return;
    }
}

pub fn analyse_each_inst(func: &Func, analyser: &mut dyn FnMut(&ObjPtr<LIRInst>)) {
    for bb in func.blocks.iter() {
        for inst in bb.insts.iter() {
            analyser(inst);
        }
    }
}
