use rand::seq::index;

use crate::{
    backend::simulator::program_stat::ProgramStat, frontend::irgen::Process, utility::ObjPool,
};

use super::*;

//关于无用指令消除的实现
impl Func {
    ///移除无用指令
    pub fn remove_unuse_inst(&mut self) {
        //TOCHECK
        // 移除mv va va 类型指令
        self.remove_self_mv();
        // 移除无用def
        self.remove_unuse_def();
    }

    ///v2p 后的移除无用指令
    pub fn remove_unuse_inst_suf_v2p(&mut self, pool: &mut BackendPool) {
        self.remove_self_mv();
        // self.remove_unuse_load_after_v2p(pool);
        // while self.remove_self_mv() {
        //     self.remove_unuse_load_after_v2p(pool);
        // }
        self.short_cut_const_count();
        self.remove_unuse_def();
        self.short_cut_mv();
        self.remove_unuse_def();
        self.short_cut_complex_expr();
        self.remove_unuse_def();
    }

    //移除
    pub fn remove_self_mv(&mut self) -> bool {
        // 移除mv va va 类型指令
        let mut if_rm = false;
        for bb in self.blocks.iter() {
            bb.as_mut().insts.retain(|inst| match inst.get_type() {
                InstrsType::OpReg(SingleOp::Mv) => {
                    let to_reg = inst.get_dst().drop_reg();
                    let from_reg = inst.get_lhs().drop_reg();
                    if to_reg == from_reg {
                        if_rm = true;
                        return false;
                    }
                    true
                }
                _ => true,
            })
        }
        if_rm
    }

    //无用的load指令
    //当且仅当v2p后能够使用
    pub fn remove_unuse_load_after_v2p(&mut self, pool: &mut BackendPool) {
        // return;
        //找到一个load指令,先往前寻找,判断是否能够块内找到中间有能够使用的物理寄存器以及store指令
        //如果找到的话,就替换该指令
        //块内部分
        self.remove_unuse_load_in_block(pool);
        // // // //块间部分,块间消除load,要找到前继块中所有的对应store,使用mv操作代替store和load操作
        self.remove_unuse_load_between_blocks(pool);
        self.remove_self_mv();
        // Func::print_func(ObjPtr::new(&self), "./pre_rm_unuse_store.txt");
        self.remove_unuse_store();
        // Func::print_func(ObjPtr::new(&self), "./suf_rm_unuse_store.txt");
        // self.remove_unuse_def();
        // Func::print_func(ObjPtr::new(&self), "after_rm_load.txt");
    }

    pub fn remove_unuse_load_in_block(&mut self, pool: &mut BackendPool) {
        // self.calc_live_base();
        // Func::print_func(ObjPtr::new(&self), "pre_rm_load_in_block.txt");
        //块内删除
        for bb in self.blocks.iter() {
            let mut to_process: Vec<(usize, usize)> = Vec::new();
            let mut loads: HashMap<i32, usize> = HashMap::new();
            bb.insts
                .iter()
                .enumerate()
                .rev()
                .for_each(|(index, inst)| match inst.get_type() {
                    InstrsType::StoreToStack => {
                        let store_to = inst.get_stack_offset().get_data();
                        if loads.contains_key(&store_to) {
                            let store_index = index;
                            let load_inex = loads.get(&store_to).unwrap();
                            to_process.push((store_index, *load_inex));
                            loads.remove(&store_to);
                        }
                    }
                    InstrsType::LoadFromStack => {
                        let pos = inst.get_stack_offset().get_data();
                        loads.insert(pos, index);
                    }
                    _ => (),
                });
            // 对于to_process,按照开头下标进行排序,每次进行一轮替换之后进行更新
            to_process.sort_by_key(|item| item.0);
            let mut index = 0;
            while index < to_process.len() {
                let (store_index, load_index) = to_process.get(index).unwrap().clone();
                let available: RegUsedStat =
                    Func::draw_available_of_certain_area(bb.as_ref(), store_index, load_index);
                let from_inst = bb.insts.get(store_index).unwrap();
                let to_inst = bb.insts.get(load_index).unwrap();
                let from_reg = from_inst.get_dst().drop_reg();
                let to_reg = to_inst.get_dst().drop_reg();
                debug_assert!(to_reg.get_type() == from_reg.get_type());
                let tmp_reg = || -> Option<Reg> {
                    if available.is_available_reg(to_reg.get_color()) {
                        return Some(to_reg);
                    } else if available.is_available_reg(from_reg.get_color()) {
                        return Some(from_reg);
                    } else if let Some(available) = available.get_available_reg(to_reg.get_type()) {
                        let tmp_reg = Reg::from_color(available);
                        return Some(tmp_reg);
                    }
                    return None;
                }();
                if tmp_reg.is_none() {
                    index += 1;
                    continue;
                }
                let tmp_reg = tmp_reg.unwrap();
                let new_from_mv = LIRInst::build_mv(&from_reg, &tmp_reg);
                let new_to_mv = LIRInst::build_mv(&tmp_reg, &to_reg);
                bb.as_mut()
                    .insts
                    .insert(store_index, pool.put_inst(new_from_mv));
                *bb.insts.get(load_index + 1).unwrap().as_mut() = new_to_mv;
                //更新完后更新下标
                //首先对于所有原本下标大于等于store_index的,下标+1
                //然后对于所有原本下标大于等于load_index的,下标+2
                let mut i = index + 1;
                while i < to_process.len() {
                    let (next_store_index, next_load_index) = to_process.get_mut(i).unwrap();
                    *next_store_index += 1;
                    *next_load_index += 1;
                    i += 1;
                }
                index += 1;
            }
        }

        self.remove_self_mv();
    }

    pub fn remove_unuse_load_between_blocks(&mut self, pool: &mut BackendPool) {
        // Func::print_func(ObjPtr::new(&self), "before_rm_load.txt");
        self.calc_live_base();
        self.remove_self_mv();

        let mut rm_each =
            |bb: &ObjPtr<BB>, unchangable: &mut HashSet<(ObjPtr<BB>, ObjPtr<LIRInst>)>| -> bool {
                let mut loads: Vec<(usize, RegUsedStat)> = Vec::new();
                let mut reg_use_stat = RegUsedStat::init_unspecial_regs();
                bb.live_in
                    .iter()
                    .for_each(|reg| reg_use_stat.use_reg(reg.get_color()));
                for (index, inst) in bb.insts.iter().enumerate() {
                    match inst.get_type() {
                        InstrsType::LoadFromStack => {
                            if !unchangable.contains(&(*bb, *inst)) {
                                loads.push((index, reg_use_stat));
                                break;
                            }
                        }
                        _ => {}
                    }
                    for reg in inst.get_regs() {
                        reg_use_stat.use_reg(reg.get_color());
                    }
                }

                if loads.len() == 0 {
                    return false;
                }
                let (load_index, mut available) = loads.get(0).unwrap();
                let mut stores: Vec<(ObjPtr<BB>, usize, RegUsedStat)> = Vec::new();
                let pos = bb
                    .insts
                    .get(*load_index)
                    .unwrap()
                    .get_stack_offset()
                    .get_data();
                //对于stores
                for in_bb in bb.in_edge.iter() {
                    Func::analyse_inst_with_regused_and_index_backorder_until(
                        &in_bb,
                        &mut |inst, index, rus| match inst.get_type() {
                            InstrsType::StoreToStack => {
                                let this_pos = inst.get_stack_offset().get_data();
                                if this_pos == pos {
                                    stores.push((*in_bb, index, *rus));
                                }
                            }
                            _ => (),
                        },
                        &|_| -> bool {
                            return false;
                        },
                    )
                }

                let load_inst = bb.insts.get(*load_index).unwrap();
                let to_reg = load_inst.get_def_reg().unwrap();
                // debug_assert!(stores.len() <= bb.in_edge.len());
                if stores.len() != bb.in_edge.len() {
                    unchangable.insert((*bb, *load_inst));
                    return false;
                }
                let mid_reg = || -> Option<Reg> {
                    let mut in_available = RegUsedStat::init_unspecial_regs();
                    for (_, _, in_a) in stores.iter() {
                        in_available.merge(in_a);
                    }
                    if available.is_available_reg(to_reg.get_color())
                        && in_available.is_available_reg(to_reg.get_color())
                    {
                        return Some(*to_reg);
                    }
                    available.merge(&in_available);
                    let color = available.get_available_reg(to_reg.get_type());
                    if let Some(color) = color {
                        return Some(Reg::from_color(color));
                    }
                    None
                }();
                if mid_reg.is_none() {
                    //把该指令加入无法使用表
                    unchangable.insert((*bb, *load_inst));
                    return false;
                }
                let mid_reg = mid_reg.unwrap();
                debug_assert!(!bb.live_in.contains(&mid_reg));
                bb.as_mut().live_in.insert(mid_reg);
                *load_inst.as_mut() = LIRInst::build_mv(&mid_reg, to_reg);
                for (in_bb, store_index, _) in stores {
                    debug_assert!(!in_bb.live_out.contains(&mid_reg));
                    in_bb.as_mut().live_out.insert(mid_reg);
                    //删除无用的store指令
                    let store_inst = in_bb.insts.get(store_index).unwrap();
                    let from_reg = store_inst.get_dst().drop_reg();
                    let from_mv_inst = LIRInst::build_mv(&from_reg, &mid_reg);
                    in_bb
                        .as_mut()
                        .insts
                        .insert(store_index, pool.put_inst(from_mv_inst));
                }
                return true;
            };

        let mut unchangable: HashSet<(ObjPtr<BB>, ObjPtr<LIRInst>)> = HashSet::new();
        loop {
            let mut finish_flag = true;
            for bb in self.blocks.iter() {
                while rm_each(bb, &mut unchangable) {
                    finish_flag = false;
                }
            }
            // self.remove_unuse_store();
            self.calc_live_base();
            if finish_flag {
                break;
            }
        }
    }

    //移除无用的store指令(有store但无use的指令)
    pub fn remove_unuse_store(&mut self) {
        //根据sst图进行无用store指令删除
        let (_, _, _, live_outs) = Func::calc_stackslot_interval(self);
        for bb in self.blocks.iter() {
            let mut livenow: HashSet<StackSlot> = live_outs.get(bb).unwrap().clone();
            let mut to_rm: HashSet<ObjPtr<LIRInst>> = HashSet::new();
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                // log_file!(
                //     "cc_rm_store.txt",
                //     "{}-{}-{:?}",
                //     bb.label,
                //     index,
                //     inst.as_ref()
                // );
                match inst.get_type() {
                    InstrsType::StoreToStack => {
                        let sst = inst.get_stackslot_with_addr_size();
                        // log!("{}-{}:", self.label, bb.label);
                        // livenow.iter().for_each(|sst| log!("{:?}", sst));
                        if !livenow.contains(&sst) && sst.get_pos() >= 0 {
                            // log_file!(
                            //     "./rm_for_store.txt",
                            //     "{}-{}-{:?}",
                            //     bb.label,
                            //     index,
                            //     inst.as_ref()
                            // );
                            to_rm.insert(*inst);
                        } else {
                            livenow.remove(&sst);
                        }
                    }
                    InstrsType::LoadFromStack => {
                        let sst = inst.get_stackslot_with_addr_size();
                        livenow.insert(sst);
                        debug_assert!(livenow.contains(&sst));
                    }
                    _ => (),
                }
            }
            bb.as_mut().insts.retain(|inst| !to_rm.contains(inst));
        }
    }

    //针对mv的值短路
    //会把对已经存在的数值的使用,改为从最早寄存器获取
    pub fn short_cut_mv(&mut self) {
        use crate::backend::simulator::structs::Value;
        Func::print_func(ObjPtr::new(&self), "before_short_cut_mv.txt");
        //维护每个寄存器当前的值
        //维护每个值先后出现的次数
        let mut val_occurs: HashMap<Value, LinkedList<Reg>> = HashMap::new();
        for bb in self.blocks.iter() {
            let mut program_stat = ProgramStat::new();
            for inst in bb.insts.iter() {
                //获取该指令涉及的寄存器,判断该指令后目的寄存器是否是常数
                for reg in inst.get_reg_use() {
                    let val = program_stat.get_val_from_reg(&reg);
                    if let Some(val) = val {
                        if val_occurs.contains_key(&val) {
                            let occurs = val_occurs.get_mut(&val).unwrap();
                            while !occurs.is_empty() {
                                let pre = occurs.front().unwrap();
                                let pre_val = program_stat.get_val_from_reg(pre);
                                if pre_val.is_none() {
                                    unreachable!();
                                }
                                let pre_val = pre_val.unwrap();
                                if pre_val != val {
                                    occurs.pop_front();
                                    continue;
                                }
                                if pre == &reg {
                                    break;
                                }
                                //否则找到了最早使用的寄存器,直接使用该寄存器
                                inst.as_mut().replace_only_use_reg(&reg, pre);
                                break;
                            }
                        }
                    }
                }
                program_stat.consume_inst(inst);
                //判断下一条指令是否需要的值在当前就已经存在了,而且存在某个寄存器里面
                //判断def 是否能够化简
                if let Some(def_reg) = inst.get_def_reg() {
                    let def_reg = *def_reg;
                    //判断当前值是否是常数,如果是常数,修改为li指令
                    //ps,def之后一定有值
                    let val = program_stat.get_val_from_reg(&def_reg).unwrap();
                    //判断当前值是否是之前已经出现过的值,如果是,使用过最早出现的值
                    if !val_occurs.contains_key(&val) {
                        val_occurs.insert(val.clone(), LinkedList::new());
                    }
                    let occurs = val_occurs.get_mut(&val).unwrap();
                    occurs.push_back(def_reg);
                    while !occurs.is_empty() {
                        let front = occurs.front().unwrap();
                        debug_assert!(
                            program_stat.get_val_from_reg(front).is_some(),
                            "{}:{},{}-{}",
                            self.label,
                            bb.label,
                            inst.as_ref(),
                            front
                        );
                        let pre_val = program_stat.get_val_from_reg(front).unwrap();
                        if pre_val != val {
                            occurs.pop_front();
                            continue;
                        }
                        if front == &def_reg {
                            break;
                        }
                        *inst.as_mut() = LIRInst::build_mv(front, &def_reg);
                        break;
                    }
                }
            }
        }
    }

    //针对常数计算的值短路, (优先改成mv,其次改成直接li)
    pub fn short_cut_const_count(&mut self) {}

    //针对特殊表达式进行的值短路
    pub fn short_cut_complex_expr(&mut self) {}

    ///移除无用def指令
    pub fn remove_unuse_def(&mut self) {
        //TODO,等待前端修改main的ret指令的类型为ScarlarType::Int
        // 循环删除无用def
        loop {
            self.calc_live_base();
            let mut finish_flag = true;
            for bb in self.blocks.iter() {
                let mut new_insts = Vec::new();
                Func::analyse_inst_with_live_now_backorder(*bb, &mut |inst, live_now| {
                    match inst.get_type() {
                        InstrsType::Call | InstrsType::Ret(_) => {
                            new_insts.push(inst);
                            return;
                        }
                        _ => (),
                    }
                    let def_reg = inst.get_def_reg();
                    if def_reg.is_none() {
                        new_insts.push(inst);
                        return;
                    }
                    let def_reg = def_reg.unwrap();
                    if !live_now.contains(def_reg) {
                        finish_flag = false;
                        return;
                    }
                    new_insts.push(inst);
                });
                new_insts.reverse();
                bb.as_mut().insts = new_insts;
            }
            if finish_flag {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_gg() {
        let mut m: Vec<i32> = Vec::new();
        m.push(33);
        m.push(44);
        m.push(555);
        let mut cindex = m.len() - 1;
        for (index, v) in m.iter().enumerate().rev() {
            assert!(cindex == index);
            if cindex == 0 {
                break;
            }
            cindex -= 1;
        }
    }
}
