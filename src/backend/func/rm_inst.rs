use crate::utility::ObjPool;

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
    pub fn remove_unuse_inst_suf_v2p(&mut self) {
        self.remove_self_mv();
        self.replace_unuse_load_after_v2p();
        while self.remove_self_mv() {
            self.replace_unuse_load_after_v2p();
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
    pub fn replace_unuse_load_after_v2p(&mut self) {
        //找到一个load指令,先往前寻找,判断是否能够块内找到中间有能够使用的物理寄存器以及store指令
        //如果找到的话,就替换该指令
        //块内部分
        self.calc_live_base();
        //使用n**2算法
        for bb in self.blocks.iter() {
            let mut to_process: Vec<(usize, usize)> = Vec::new();
            let mut loads: HashMap<i32, usize> = HashMap::new();
            Func::analyse_inst_with_live_now_and_index_backorder(
                *bb,
                &mut |inst, index, live_now| match inst.get_type() {
                    InstrsType::StoreToStack => {
                        let pos = inst.get_stack_offset().get_data();
                        if loads.contains_key(&pos) {
                            let load_index = loads.get(&pos).unwrap();
                            to_process.push((index, *load_index));
                        }
                    }
                    InstrsType::LoadFromStack => {
                        let pos = inst.get_stack_offset().get_data();
                        loads.insert(pos, index);
                    }
                    _ => (),
                },
            );
            //处理替换过程
            for (store_index, load_index) in to_process {
                let available =
                    Func::draw_available_of_certain_area(bb.as_ref(), store_index, load_index);
                let from_inst = bb.insts.get(store_index).unwrap();
                let to_inst = bb.insts.get(load_index).unwrap();
                let from_reg = from_inst.get_dst().drop_reg();
                let to_reg = to_inst.get_dst().drop_reg();
                if available.is_available_reg(from_reg.get_color()) {
                    *from_inst.as_mut() = LIRInst::build_mv(&from_reg, &from_reg);
                    *to_inst.as_mut() = LIRInst::build_mv(&from_reg, &to_reg);
                    continue;
                }
                if available.is_available_reg(to_reg.get_color()) {
                    *from_inst.as_mut() = LIRInst::build_mv(&from_reg, &to_reg);
                    *to_inst.as_mut() = LIRInst::build_mv(&to_reg, &to_reg);
                    continue;
                }
                let available = available.get_available_reg(to_reg.get_type());
                if let Some(available) = available {
                    let tmp_reg = Reg::from_color(available);
                    *from_inst.as_mut() = LIRInst::build_mv(&from_reg, &tmp_reg);
                    *to_inst.as_mut() = LIRInst::build_mv(&tmp_reg, &to_reg);
                }
                //否则替换失败
            }
        }
        //块间部分,块间消除load,要找到前继块中所有的对应store,使用mv操作代替store和load操作
        self.calc_live_base();
        loop {
            let mut finish_flag = true;
            //找到每个块前面的第一个load指令,找到块的前继块的store指令,如果前继块的store指令
            for bb in self.blocks.iter() {
                let mut loads: Vec<(ObjPtr<LIRInst>, RegUsedStat)> = Vec::new();
                let mut reg_use_stat = RegUsedStat::init_unspecial_regs();
                bb.live_in
                    .iter()
                    .for_each(|reg| reg_use_stat.use_reg(reg.get_color()));
                for inst in bb.insts.iter() {
                    match inst.get_type() {
                        InstrsType::LoadFromStack => {
                            loads.push((*inst, reg_use_stat));
                        }
                        _ => {
                            for reg in inst.get_reg_def() {
                                reg_use_stat.use_reg(reg.get_color());
                            }
                        }
                    }
                }

                let mut reg_used_for_replace = RegUsedStat::new();
                for (load_inst, mut available) in loads {
                    available.merge(&reg_used_for_replace);
                    let mut stores: Vec<(ObjPtr<LIRInst>, RegUsedStat)> = Vec::new();
                    let pos = load_inst.get_stack_offset().get_data();
                    for in_bb in bb.in_edge.iter() {
                        let mut if_find = false;
                        let mut tmp_pool: ObjPool<bool> = ObjPool::new();
                        let if_find = tmp_pool.put(if_find);
                        Func::analyse_inst_with_regused_backorder_until(
                            in_bb,
                            &mut |inst, rus| {
                                match inst.get_type() {
                                    InstrsType::StoreToStack => {
                                        let this_pos = inst.get_stack_offset().get_data();
                                        if this_pos == pos {
                                            stores.push((inst, *rus));
                                            *if_find.as_mut() = true;
                                        }
                                    }
                                    _ => (),
                                };
                            },
                            &|inst| {
                                return *if_find.as_ref();
                            },
                        );
                        tmp_pool.free_all();
                    }
                    //找到前继块中所有的对应store的位置
                    if stores.len() != bb.in_edge.len() {
                        continue;
                    }

                    //若可以更新,进行更新,并且更新live in live out
                    //首先找到通用的reg
                    let to_reg = load_inst.get_dst().drop_reg();
                    let mut from_available = RegUsedStat::init_unspecial_regs();
                    for (_, in_available) in stores.iter() {
                        from_available.merge(in_available);
                    }
                    let tmp_reg = || -> Option<Reg> {
                        if available.is_available_reg(to_reg.get_color())
                            && from_available.is_available_reg(to_reg.get_color())
                        {
                            return Some(to_reg);
                        }
                        available.merge(&from_available);
                        let tr = available.get_available_reg(to_reg.get_type());
                        if let Some(color) = tr {
                            return Some(Reg::from_color(color));
                        }
                        return None;
                    }();
                    if tmp_reg.is_none() {
                        continue;
                    }
                    let tmp_reg = tmp_reg.unwrap();
                    reg_used_for_replace.use_reg(tmp_reg.get_color());
                    debug_assert!(!bb.live_in.contains(&tmp_reg));
                    bb.as_mut().live_in.insert(tmp_reg);
                    *load_inst.as_mut() = LIRInst::build_mv(&tmp_reg, &to_reg);
                    for (store_inst, _) in stores {
                        let from_reg = store_inst.get_dst().drop_reg();
                        *store_inst.as_mut() = LIRInst::build_mv(&from_reg, &to_reg);
                    }
                    for in_bb in bb.in_edge.iter() {
                        debug_assert!(!in_bb.live_out.contains(&tmp_reg));
                        in_bb.as_mut().live_out.insert(tmp_reg);
                    }
                }
            }
            if finish_flag {
                break;
            }
        }
    }

    //移除无用的store指令
    pub fn remove_unuse_store(&mut self) {}

    //针对mv的值短路
    pub fn short_cut_mv(&mut self) {}

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
