use rand::seq::index;

use crate::{
    backend::simulator::{execute_stat::ExecuteStat, program_stat::ProgramStat, structs::Value},
    frontend::irgen::Process,
    utility::ObjPool,
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

    pub fn rm_inst_suf_update_array_offset(
        &mut self,
        pool: &mut BackendPool,
        regs_used_but_not_saved: &HashMap<String, HashSet<Reg>>,
    ) {
        self.remove_meaningless_def(regs_used_but_not_saved);
        self.remove_unuse_def();

        self.short_cut_mv(regs_used_but_not_saved);

        while self.remove_self_mv() || self.remove_unuse_def() {
            self.short_cut_mv(regs_used_but_not_saved);
        }
        self.short_cut_const();
        while self.remove_unuse_def() {
            self.short_cut_const();
        }
        self.short_cut_mv(regs_used_but_not_saved);
        while self.remove_self_mv() || self.remove_unuse_def() {
            self.short_cut_mv(regs_used_but_not_saved);
        }
        while self.remove_unuse_store() {
            self.remove_unuse_def();
        }
        self.remove_self_mv();
        self.remove_meaningless_def(regs_used_but_not_saved);
        self.remove_unuse_def();
        self.remove_self_mv();
        Func::print_func(ObjPtr::new(&self), "after_rm_suf_update_array_offset.txt");
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
                        log_file!(
                            "remove_self_mv.txt",
                            "{}:{},{:?}",
                            bb.label,
                            inst.as_ref(),
                            inst.as_ref()
                        );
                        return false;
                    }
                    return true;
                }
                _ => true,
            })
        }
        if_rm
    }

    //移除无意义的def
    pub fn remove_meaningless_def(
        &mut self,
        regs_used_but_not_saved: &HashMap<String, HashSet<Reg>>,
    ) {
        //遇到program stat前后值不改变的情况,则删除无用的def指令,除非这条def指令是call指令
        for bb in self.blocks.iter() {
            let mut program_stat = ProgramStat::new();
            let mut to_rm = HashSet::new();
            for inst in bb.insts.iter() {
                let reg_def = inst.get_def_reg();
                if reg_def.is_none() {
                    program_stat.consume_inst(inst);
                    continue;
                }

                let reg_def = reg_def.as_ref().unwrap();
                let old_val = program_stat.reg_val.get(reg_def);
                if old_val.is_none() {
                    program_stat.consume_inst(inst);
                    continue;
                }
                let old_val = old_val.unwrap().clone();

                //记录需要保存的值

                if inst.get_type() == InstrsType::Call {
                    let mut to_saveds = HashMap::new();
                    let func = inst.get_func_name().unwrap();
                    let func = func.as_str();
                    let reg_used_but_not_saved = regs_used_but_not_saved.get(func).unwrap();
                    for (reg, val) in program_stat.reg_val.iter() {
                        if reg_used_but_not_saved.contains(reg) {
                            continue;
                        }
                        to_saveds.insert(*reg, val.clone());
                    }
                    program_stat.consume_inst(inst);
                    for (reg, val) in to_saveds {
                        program_stat.reg_val.insert(reg, val);
                    }
                    continue;
                }

                program_stat.consume_inst(inst);

                let new_val = program_stat.reg_val.get(reg_def).unwrap();
                if new_val == &old_val {
                    // println!("{}", inst.as_ref());
                    to_rm.insert(*inst);
                }
            }
            bb.as_mut().insts.retain(|inst| !to_rm.contains(inst));
        }
    }

    //移除无用的store指令(有store但无use的指令)
    pub fn remove_unuse_store(&mut self) -> bool {
        let mut out = false;
        //根据sst图进行无用store指令删除
        config::record_event(format!("start build liveout for {}", self.label).as_str());
        let (_, _, _, live_outs) = Func::calc_stackslot_interval(self);
        config::record_event(format!("finish build liveout for {}", self.label).as_str());
        for bb in self.blocks.iter() {
            let mut livenow: HashSet<StackSlot> = live_outs.get(bb).unwrap().clone();
            let mut to_rm: HashSet<ObjPtr<LIRInst>> = HashSet::new();
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                match inst.get_type() {
                    InstrsType::StoreToStack => {
                        let sst = inst.get_stackslot_with_addr_size();
                        if !livenow.contains(&sst) && sst.get_pos() >= 0 {
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
            if to_rm.len() != 0 {
                out = true;
            }
            bb.as_mut().insts.retain(|inst| !to_rm.contains(inst));
        }
        out
    }

    ///移除无用def指令
    pub fn remove_unuse_def(&mut self) -> bool {
        //TODO,等待前端修改main的ret指令的类型为ScarlarType::Int
        // 循环删除无用def
        let mut out = false;
        loop {
            self.calc_live_base();
            let mut finish_flag = true;

            for bb in self.blocks.iter() {
                let mut to_rm: HashSet<ObjPtr<LIRInst>> = HashSet::new();

                Func::analyse_inst_with_live_now_backorder(*bb, &mut |inst, live_now| {
                    match inst.get_type() {
                        InstrsType::Call => {
                            return;
                        }
                        _ => (),
                    };
                    let def_reg = inst.get_def_reg();
                    if def_reg.is_none() {
                        return;
                    }
                    let def_reg = def_reg.unwrap();
                    let def_reg = &def_reg;
                    // println!("{def_reg}");
                    if !live_now.contains(def_reg) {
                        to_rm.insert(inst);
                        return;
                    }
                });
                bb.as_mut().insts.retain(|inst| !to_rm.contains(inst));
                if to_rm.len() != 0 {
                    log_file!("rm_unuse_def.txt", "bb{}", bb.label);
                    for rm in to_rm {
                        log_file!("rm_unuse_def.txt", "inst:{},{:?}", rm.as_ref(), rm.as_ref());
                    }
                    finish_flag = false;
                }
            }
            if finish_flag {
                break;
            }
            out = true;
        }
        out
    }
}

///short cut 系列
impl Func {
    //针对mv的值短路
    //会把对已经存在的数值的使用,改为从最早寄存器获取,
    pub fn short_cut_mv(&mut self, regs_used_but_not_saved: &HashMap<String, HashSet<Reg>>) {
        // Func::print_func(ObjPtr::new(&self), "before_short_cut_mv.txt");
        //维护每个寄存器当前的值
        //维护每个值先后出现的次数
        //只针对块内的局部短路
        for bb in self.blocks.iter() {
            let mut val_occurs: HashMap<Value, LinkedList<Reg>> = HashMap::new();
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
                                    occurs.pop_front();
                                    continue;
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
                let mut to_saveds = HashMap::new();
                if inst.get_type() == InstrsType::Call {
                    let func = inst.get_func_name().unwrap();
                    let func = func.as_str();
                    let reg_used_but_not_saved = regs_used_but_not_saved.get(func).unwrap();
                    for (reg, val) in program_stat.reg_val.iter() {
                        if reg_used_but_not_saved.contains(reg) {
                            continue;
                        }
                        to_saveds.insert(*reg, val.clone());
                    }
                }
                program_stat.consume_inst(inst);
                for (reg, val) in to_saveds {
                    program_stat.reg_val.insert(reg, val);
                }

                //判断下一条指令是否需要的值在当前就已经存在了,而且存在某个寄存器里面
                //判断def 是否能够化简
                if let Some(def_reg) = inst.get_def_reg() {
                    let def_reg = def_reg;
                    //判断当前值是否是常数,如果是常数,修改为li指令
                    //ps,def之后一定有值
                    let val = program_stat.get_val_from_reg(&def_reg).unwrap();
                    //判断当前值是否是之前已经出现过的值,如果是,使用过最早出现的值
                    if !val_occurs.contains_key(&val) {
                        val_occurs.insert(val.clone(), LinkedList::new());
                    }
                    let occurs = val_occurs.get_mut(&val).unwrap();
                    //当这条指令是mv指令的时候做特殊处理
                    match inst.get_type() {
                        InstrsType::OpReg(SingleOp::Mv) => {
                            //认为源寄存器更早出现
                            let src_reg = inst.get_lhs().drop_reg();
                            occurs.push_back(src_reg);
                            debug_assert!(
                                program_stat.reg_val.get(&src_reg).unwrap()
                                    == program_stat.reg_val.get(&def_reg).unwrap()
                            );
                        }
                        _ => (),
                    };

                    occurs.push_back(def_reg);
                    while !occurs.is_empty() {
                        let front = occurs.front().unwrap();
                        let pre_val = program_stat.get_val_from_reg(front);
                        if pre_val.is_none() {
                            occurs.pop_front();
                            continue;
                        }
                        let pre_val = pre_val.unwrap();
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

        // Func::print_func(ObjPtr::new(&self), "after_short_cut_mv.txt");
    }

    //针对常数赋值的值短路, (对于常量值的加载,优先改为li)
    pub fn short_cut_const(&mut self) {
        // Func::print_func(ObjPtr::new(&self), "before_short_cut_const.txt");
        //对于所有常量类型的赋值,如果确定是常量,改为li
        for bb in self.blocks.iter() {
            let mut program_stat = ProgramStat::new();
            for inst in bb.insts.iter() {
                program_stat.consume_inst(inst);
                let def_reg = inst.get_def_reg();
                if def_reg.is_none() {
                    continue;
                }
                let def_reg = def_reg.unwrap();
                let def_reg = &def_reg;
                let val = program_stat.get_val_from_reg(def_reg).unwrap();
                match val {
                    Value::IImm(val) => {
                        //常量替换
                        let imm_li = LIRInst::build_li_inst(def_reg, val);
                        *inst.as_mut() = imm_li;
                    }
                    _ => (),
                }
            }
        }
        // Func::print_func(ObjPtr::new(self), "after_short_cut_const.txt");
    }

    ///针对计算表达式的值短路,如果要获取一个常数值,则使用最近的比较大的值中获取
    pub fn short_cut_const_count(&mut self) {}

    ///对于单链mv进行值传递,(只考虑两环的情况,因为经过short cut const,const count,mv之后,只剩下两环的mv能优化)
    pub fn shrink_mv_trans(&mut self) {
        self.calc_live_base();
        //
        unimplemented!("也许不需要计算这个,可以通过考虑全局的short cut mv解决");
        for bb in self.blocks.iter() {
            //分析blocks 内的def use关系
            //记录值链以及中间的def use关系
            let mut defs: HashMap<Reg, ObjPtr<LIRInst>> = HashMap::new();
            // let mut used_between
            let mut trans: Vec<(ObjPtr<LIRInst>, ObjPtr<LIRInst>)> = Vec::new();
        }
    }

    ///针对特殊表达式进行的值短路
    pub fn short_cut_complex_expr(&mut self) {}
}

//handle call后对于load store指令的消除
impl Func {
    pub fn remove_sl_after_handle_call(
        &mut self,
        pool: &mut BackendPool,
        used_but_not_saved: &HashSet<String, HashSet<Reg>>,
    ) {
        self.calc_live_base();
        //块内消除

        //块间消除
    }

    pub fn remove_sl_after_handle_call_in_block(
        &mut self,
        pool: &mut BackendPool,
        used_but_not_saved: &HashSet<String, HashSet<Reg>>,
    ) {
    }

    pub fn remove_sl_after_handle_call_between_blocks(
        &mut self,
        pool: &mut BackendPool,
        used_but_not_saved: &HashSet<String, HashSet<Reg>>,
    ) {
    }
}

///因为handle spill产生的无用指令的删除
impl Func {
    ///都是需要用到的时候才进行寄存器值得归还与存储
    ///但是在不同块之间的时候可以用寄存器的借还操作代替从内存空间读取值的操作
    pub fn remove_inst_suf_spill(&mut self, pool: &mut BackendPool) {
        // //进行寄存器之间的移动操作
        // self.calc_live_base();
        // //如果只有一个前继块,则前继块中的spilling优先使用可用的物理寄存器移动到后方
        // self.remove_unuse_load_after_handle_spill(pool);
        // while self.remove_self_mv() {
        //     self.remove_unuse_load_after_handle_spill(pool);
        // }
    }
    //无用的load指令
    //当且仅当v2p后能够使用
    fn remove_unuse_load_after_handle_spill(&mut self, pool: &mut BackendPool) {
        // return;
        //找到一个load指令,先往前寻找,判断是否能够块内找到中间有能够使用的物理寄存器以及store指令
        //如果找到的话,就替换该指令
        //块内部分
        self.remove_unuse_load_in_block_after_handle_spill(pool);
        // // // //块间部分,块间消除load,要找到前继块中所有的对应store,使用mv操作代替store和load操作
        self.remove_unuse_load_between_blocks_after_handle_spill(pool);
        self.remove_self_mv();
        // Func::print_func(ObjPtr::new(&self), "./pre_rm_unuse_store.txt");
        self.remove_unuse_store();
        // Func::print_func(ObjPtr::new(&self), "./suf_rm_unuse_store.txt");
        // self.remove_unuse_def();
        // Func::print_func(ObjPtr::new(&self), "after_rm_load.txt");
    }

    fn remove_unuse_load_in_block_after_handle_spill(&mut self, pool: &mut BackendPool) {
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

    fn remove_unuse_load_between_blocks_after_handle_spill(&mut self, pool: &mut BackendPool) {
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
                        return Some(to_reg);
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
                *load_inst.as_mut() = LIRInst::build_mv(&mid_reg, &to_reg);
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
}

///通用的sl指令的替换与删除
impl Func {}

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
