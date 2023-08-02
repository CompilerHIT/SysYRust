use rand::seq::index;

use crate::{frontend::irgen::Process, utility::ObjPool};

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
        self.remove_unuse_load_after_v2p(pool);
        while self.remove_self_mv() {
            self.remove_unuse_load_after_v2p(pool);
        }
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
        //找到一个load指令,先往前寻找,判断是否能够块内找到中间有能够使用的物理寄存器以及store指令
        //如果找到的话,就替换该指令
        //块内部分
        self.calc_live_base();

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
                let new_to_mv = LIRInst::build_mv(&tmp_reg, &tmp_reg);
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

        // //块间部分,块间消除load,要找到前继块中所有的对应store,使用mv操作代替store和load操作
        Func::print_func(ObjPtr::new(&self), "before_rm_load.txt");
        self.calc_live_base();
        self.remove_self_mv();
        // let mut unchangable
        //找到每个块前面的第一个load指令,找到块的前继块的store指令,如果前继块的store指令
        for bb in self.blocks.iter() {
            let mut loads: Vec<(usize, RegUsedStat)> = Vec::new();
            let mut reg_use_stat = RegUsedStat::init_unspecial_regs();
            bb.live_in
                .iter()
                .for_each(|reg| reg_use_stat.use_reg(reg.get_color()));
            for (index, inst) in bb.insts.iter().enumerate() {
                match inst.get_type() {
                    InstrsType::LoadFromStack => {
                        loads.push((index, reg_use_stat));
                        break;
                    }
                    _ => {}
                }
                for reg in inst.get_regs() {
                    reg_use_stat.use_reg(reg.get_color());
                }
            }

            if loads.len() == 0 {
                continue;
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

            // debug_assert!(stores.len() <= bb.in_edge.len());
            if stores.len() != bb.in_edge.len() {
                continue;
            }
            let load_inst = bb.insts.get(*load_index).unwrap();
            let to_reg = load_inst.get_def_reg().unwrap();
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
                continue;
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
        }

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
                            loads.push((index, reg_use_stat));
                            break;
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
            self.remove_unuse_store();
            self.calc_live_base();
            if finish_flag {
                break;
            }
        }

        // self.remove_self_mv();
        // Func::print_func(ObjPtr::new(&self), "pre_rm_unuse_store.txt");
        self.remove_unuse_store();
        // Func::print_func(ObjPtr::new(&self), "suf_rm_unuse_store.txt");
        // self.remove_unuse_def();
        // Func::print_func(ObjPtr::new(&self), "after_rm_load.txt");
    }

    //移除无用的store指令(有store但无use的指令)
    pub fn remove_unuse_store(&mut self) {
        //根据sst图进行无用store指令删除
        let (_, _, _, live_outs) = Func::calc_stackslot_interval(self);
        for bb in self.blocks.iter() {
            let mut livenow: HashSet<StackSlot> = live_outs.get(bb).unwrap().clone();
            let mut to_rm: HashSet<ObjPtr<LIRInst>> = HashSet::new();
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                match inst.get_type() {
                    InstrsType::StoreToStack => {
                        let sst = inst.get_stackslot_with_addr_size();
                        if !livenow.contains(&sst) && sst.get_pos() >= 0 {
                            println!("{}-{}-{}", bb.label, index, inst.as_ref());
                            to_rm.insert(*inst);
                        } else {
                            livenow.remove(&sst);
                        }
                    }
                    InstrsType::LoadFromStack => {
                        let sst = inst.get_stackslot_with_addr_size();
                        livenow.insert(sst);
                    }
                    _ => (),
                }
            }
            bb.as_mut().insts.retain(|inst| !to_rm.contains(inst));
        }
    }

    //针对mv的值短路
    pub fn short_cut_mv(&mut self) {
        use crate::backend::simulator::structs::Value;
        //维护每个寄存器当前的值
        //维护每个值先后出现的次数
        let mut val_occurs: HashMap<Value, LinkedList<Reg>> = HashMap::new();
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
            self.remove_unuse_store();
        }
    }
}
