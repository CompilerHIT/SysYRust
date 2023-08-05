use super::*;

///寄存器重分配相关接口的实现
impl Func {
    ///p_to_v
    ///把函数中所有在regs中的物理寄存器进行ptov(除了call指令def和call指令use的寄存器)<br>
    /// 该行为需要在handle call之前执行 (在这个试图看来,一个call前后除了a0的值可能发生改变,其他寄存器的值并不会发生改变)
    ///因为在handle call后有有些寄存器需要通过栈来restore,暂时还没有分析这个行为
    /// 该函数会绝对保留原本程序的结构，并且不会通过构造phi等行为增加指令,不会调整指令顺序,不会合并寄存器等等
    pub fn p2v_pre_handle_call(&mut self, regs_to_decolor: HashSet<Reg>) -> HashSet<Reg> {
        return self.p2v_2(regs_to_decolor);

        let path = "p2v.txt";

        debug_assert!(!regs_to_decolor.contains(&Reg::get_sp()));
        debug_assert!(!regs_to_decolor.contains(&Reg::get_ra()));
        debug_assert!(!regs_to_decolor.contains(&Reg::get_tp()));
        debug_assert!(!regs_to_decolor.contains(&Reg::get_gp()));

        let mut new_v_regs = HashSet::new(); //用来记录新产生的虚拟寄存器
                                             // self.print_func();
        self.calc_live_base();
        let unchanged_def = self.build_unchanged_def();
        let unchanged_use = self.build_unchanged_use();

        // let mut to_pass: LinkedList<ObjPtr<BB>> = LinkedList::new();
        // to_pass.push_back(first_block);
        let mut forward_passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
        let mut backward_passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
        // 搜索单元分为正向搜索单元与反向搜索单元
        for bb in self.blocks.iter() {
            if bb.insts.len() == 0 {
                continue;
            }

            let mut old_new: HashMap<Reg, Reg> = HashMap::with_capacity(64);
            let mut to_forward: LinkedList<(ObjPtr<BB>, Reg, Reg)> = LinkedList::new();
            let mut to_backward: LinkedList<(ObjPtr<BB>, Reg, Reg)> = LinkedList::new();
            // 对于live out的情况(插入一些到forward中)
            for reg in bb.live_out.iter() {
                //
                if !reg.is_physic() {
                    continue;
                }
                if !regs_to_decolor.contains(reg) {
                    continue;
                }
                if backward_passed.contains(&(*bb, *reg)) {
                    continue;
                }
                let new_reg = Reg::init(reg.get_type());
                new_v_regs.insert(new_reg);
                old_new.insert(*reg, new_reg);
                backward_passed.insert((*bb, *reg));
                // 加入到后出表中
                for out_bb in bb.out_edge.iter() {
                    if out_bb == bb {
                        continue;
                    }
                    if !out_bb.live_in.contains(reg) {
                        continue;
                    }
                    debug_assert!(!forward_passed.contains(&(*out_bb, *reg)));
                    forward_passed.insert((*out_bb, *reg));
                    to_forward.push_back((*out_bb, *reg, new_reg));
                }
            }
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                if bb.label == "params_mix" && index == 161 {
                    //161
                    println!("{:?}", inst.as_ref());
                    let reg_use = inst.get_reg_use();
                    debug_assert!(reg_use.contains(&Reg::get_a0()));
                    debug_assert!(
                        unchanged_use.contains(&(*inst, Reg::get_a0())),
                        "{:?}",
                        inst.as_ref()
                    );
                }

                for reg_def in inst.get_reg_def() {
                    if !regs_to_decolor.contains(&reg_def) {
                        continue;
                    }
                    if !reg_def.is_physic() {
                        continue;
                    }
                    if unchanged_def.contains(&(*inst, reg_def)) {
                        continue;
                    }
                    debug_assert!(reg_def.is_physic() && regs_to_decolor.contains(&reg_def));
                    debug_assert!(old_new.contains_key(&reg_def), "{}", inst.as_ref());
                    log_file!(
                        path,
                        "replace def:{},{}{}{}->{}",
                        index,
                        bb.label,
                        inst.as_ref(),
                        reg_def,
                        old_new.get(&reg_def).unwrap()
                    );
                    inst.as_mut()
                        .replace_only_def_reg(&reg_def, old_new.get(&reg_def).unwrap());
                    old_new.remove(&reg_def);
                }
                for reg_use in inst.get_reg_use() {
                    if !regs_to_decolor.contains(&reg_use) {
                        continue;
                    }
                    if !reg_use.is_physic() {
                        continue;
                    }
                    debug_assert!(reg_use.is_physic() && regs_to_decolor.contains(&reg_use));
                    if unchanged_use.contains(&(*inst, reg_use)) {
                        continue;
                    }
                    if !old_new.contains_key(&reg_use) {
                        let new_v_reg = Reg::init(reg_use.get_type());
                        new_v_regs.insert(new_v_reg);
                        old_new.insert(reg_use, new_v_reg);
                    }
                    log_file!(
                        path,
                        "replace use:{}{}{}->{}",
                        bb.label,
                        inst.as_ref(),
                        reg_use,
                        old_new.get(&reg_use).unwrap()
                    );
                    inst.as_mut()
                        .replace_only_use_reg(&reg_use, old_new.get(&reg_use).unwrap());
                }
            }
            // 对于最后剩下来的寄存器,初始化前向表
            for (old_reg, new_reg) in old_new.iter() {
                for in_bb in bb.in_edge.iter() {
                    if in_bb == bb {
                        continue;
                    }
                    if backward_passed.contains(&(*in_bb, *old_reg)) {
                        continue;
                    }
                    backward_passed.insert((*in_bb, *old_reg));
                    to_backward.push_back((*in_bb, *old_reg, *new_reg));
                }
            }

            loop {
                //遍历前后向表,反着色
                while !to_forward.is_empty() {
                    let (bb, old_reg, new_reg) = to_forward.pop_front().unwrap();
                    //对于前向表(先进行反向试探)
                    for in_bb in bb.in_edge.iter() {
                        if !in_bb.live_out.contains(&old_reg) {
                            continue;
                        }
                        let key = (*in_bb, old_reg);
                        if backward_passed.contains(&key) {
                            continue;
                        }
                        backward_passed.insert(key);
                        to_backward.push_back((*in_bb, old_reg, new_reg));
                    }

                    let mut if_keep_forward = true;

                    for inst in bb.insts.iter() {
                        for reg_use in inst.get_reg_use() {
                            if reg_use != old_reg {
                                continue;
                            }
                            // debug_assert!(
                            //     !unchanged_use.contains(&(*inst, reg_use)),
                            //     "{},{}",
                            //     inst.as_ref(),
                            //     reg_use,
                            // );
                            if !unchanged_use.contains(&(*inst, reg_use)) {
                                inst.as_mut().replace_only_use_reg(&old_reg, &new_reg);
                            }
                        }
                        if inst.get_reg_def().contains(&old_reg) {
                            if_keep_forward = false;
                            break;
                        }
                    }

                    //如果中间结束,则直接进入下一轮
                    if !if_keep_forward {
                        continue;
                    }
                    // 到了尽头,判断是否后递
                    for out_bb in bb.out_edge.iter() {
                        let key = (*out_bb, old_reg);
                        if forward_passed.contains(&key) {
                            continue;
                        }
                        forward_passed.insert(key);
                        to_forward.push_back((*out_bb, old_reg, new_reg));
                    }
                }
                while !to_backward.is_empty() {
                    let (bb, old_reg, new_reg) = to_backward.pop_front().unwrap();

                    //反向者寻找所有前向
                    for out_bb in bb.out_edge.iter() {
                        if !out_bb.live_in.contains(&old_reg) {
                            continue;
                        }
                        let key = (*out_bb, old_reg);
                        if forward_passed.contains(&key) {
                            continue;
                        }
                        forward_passed.insert(key);
                        to_forward.push_back((*out_bb, old_reg, new_reg));
                    }

                    let mut if_keep_backward = true;

                    for inst in bb.insts.iter().rev() {
                        if inst.get_reg_def().contains(&old_reg) {
                            if !unchanged_def.contains(&(*inst, old_reg)) {
                                inst.as_mut().replace_only_def_reg(&old_reg, &new_reg);
                            }
                            if_keep_backward = false;
                            break;
                        }
                        inst.as_mut().replace_only_use_reg(&old_reg, &new_reg);
                    }
                    if !if_keep_backward {
                        continue;
                    }
                    for in_bb in bb.in_edge.iter() {
                        if !in_bb.live_out.contains(&old_reg) {
                            continue;
                        }
                        let key = (*in_bb, old_reg);
                        if backward_passed.contains(&key) {
                            continue;
                        }
                        backward_passed.insert(key);
                        to_backward.push_back((*in_bb, old_reg, new_reg));
                    }
                }
                if to_forward.is_empty() && to_backward.is_empty() {
                    break;
                }
            }
        }
        //从基础搜索单元开始遍历

        // self.print_func();
        new_v_regs
    }

    ///着色
    pub fn v2p(&mut self, colors: &HashMap<i32, i32>) {
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_physic() {
                        continue;
                    }
                    if !colors.contains_key(&reg.get_id()) {
                        continue;
                    }
                    let color = colors.get(&reg.get_id()).unwrap();
                    inst.as_mut().replace(reg.get_id(), *color);
                }
            }
        }
    }
}

impl Func {
    ///该函数计算结果依赖外部调用的calc liv3
    pub fn build_unchanged_def(&mut self) -> HashSet<(ObjPtr<LIRInst>, Reg)> {
        // self.print_func();
        //只有传参过程存在unchanged def,以及call指令可能存在unchanged def
        let mut unchanged_def: HashSet<(ObjPtr<LIRInst>, Reg)> = HashSet::new();
        for bb in self.blocks.iter() {
            for (i, inst) in bb.insts.iter().enumerate() {
                if inst.get_type() != InstrsType::Call {
                    continue;
                }
                let mut used: HashSet<Reg> = inst.get_reg_use().iter().cloned().collect();
                if i != 0 {
                    let mut index: i32 = i as i32 - 1;
                    while index >= 0 && used.len() != 0 {
                        let inst = *bb.insts.get(index as usize).unwrap();
                        for reg_def in inst.get_reg_def() {
                            if !used.contains(&reg_def) {
                                continue;
                            }
                            used.remove(&reg_def);
                            unchanged_def.insert((inst, reg_def));
                        }
                        if index == 0 {
                            break;
                        }
                        index -= 1;
                    }
                }
                if used.len() != 0 {
                    //TODO  (暂时不考虑 参数的加入不在同一个块中的情况)
                    //used 传递到前文的情况
                    let mut to_backward: LinkedList<(ObjPtr<BB>, Reg)> = LinkedList::new();
                    let mut backwarded: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
                    for reg_used in used.iter() {
                        for in_bb in bb.in_edge.iter() {
                            if !in_bb.live_out.contains(reg_used) {
                                continue;
                            }
                            // unreachable!();
                            to_backward.push_back((*in_bb, *reg_used));
                        }
                    }
                    while !to_backward.is_empty() {
                        let item = to_backward.pop_front().unwrap();
                        if backwarded.contains(&item) {
                            continue;
                        }
                        backwarded.insert(item);
                        let (bb, reg) = item;
                        let mut keep_backward = true;
                        for inst in bb.insts.iter().rev() {
                            if inst.get_reg_def().contains(&reg) {
                                unchanged_def.insert((*inst, reg));
                                keep_backward = false;
                                break;
                            }
                        }
                        if !keep_backward {
                            continue;
                        }
                        for in_bb in bb.in_edge.iter() {
                            if !in_bb.live_out.contains(&reg) {
                                continue;
                            }
                            to_backward.push_back((*in_bb, reg));
                        }
                    }
                    debug_assert!(to_backward.is_empty());
                }

                let mut defined: HashSet<Reg> = inst.get_reg_def().iter().cloned().collect();
                for defined in defined.iter() {
                    unchanged_def.insert((*inst, *defined));
                }
                let mut index = i + 1;
                // 往后继块传递defined
                while index < bb.insts.len() && defined.len() != 0 {
                    let inst = *bb.insts.get(index).unwrap();
                    for reg in inst.get_reg_def() {
                        if defined.contains(&reg) {
                            defined.remove(&reg);
                        }
                    }
                    index += 1;
                }
                if defined.len() != 0 {
                    // 按照目前的代码结构来说不应该存在
                    // 说明define到了live out中(说明其他块使用了这个块中的计算出的a0)
                    // 则其他块中计算出的a0也应该使用相同的物理寄存器号(不应该改变)
                    let mut to_pass: LinkedList<(ObjPtr<BB>, Reg)> = LinkedList::new();
                    for out_bb in bb.out_edge.iter() {
                        for reg in defined.iter() {
                            if !out_bb.live_in.contains(reg) {
                                continue;
                            }
                            to_pass.push_back((*out_bb, *reg));
                        }
                    }
                    let mut passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
                    while !to_pass.is_empty() {
                        let (bb, reg) = to_pass.pop_front().unwrap();
                        if passed.contains(&(bb, reg)) {
                            continue;
                        }
                        passed.insert((bb, reg));
                        let mut index = 0;
                        while index < bb.insts.len() {
                            let inst = bb.insts.get(index).unwrap();
                            if inst.get_reg_def().contains(&reg) {
                                break;
                            }
                            index += 1;
                        }

                        if index == bb.insts.len() {
                            //说明可能传到live out中
                            for out_bb in bb.out_edge.iter() {
                                if out_bb.live_in.contains(&reg) {
                                    to_pass.push_back((*out_bb, reg));
                                }
                            }
                        }
                    }
                }
            }
        }
        // 考虑ret
        // 一个block中只可能出现一条return最多
        let final_bb = self.get_final_bb();
        let last_inst = final_bb.insts.last().unwrap();
        let use_reg = last_inst.get_reg_use();
        debug_assert!(use_reg.len() == 1 || self.label != "main");
        if let Some(use_reg) = use_reg.get(0) {
            let mut back_bbs: LinkedList<ObjPtr<BB>> = LinkedList::new();
            back_bbs.push_back(final_bb);
            let mut passed = HashSet::new();
            while back_bbs.len() != 0 {
                let bb = back_bbs.pop_front().unwrap();
                if passed.contains(&bb) {
                    continue;
                }
                passed.insert(bb);
                let mut index = bb.insts.len() - 1;
                let mut if_finish = false;
                loop {
                    let inst = bb.insts.get(index).unwrap();
                    if inst.get_reg_def().contains(use_reg) {
                        unchanged_def.insert((*inst, *use_reg));
                        if_finish = true;
                        break;
                    }
                    if index == 0 {
                        break;
                    }
                    index -= 1;
                }

                if !if_finish {
                    for in_bb in bb.in_edge.iter() {
                        debug_assert!(in_bb.live_out.contains(use_reg));
                        back_bbs.push_back(*in_bb);
                    }
                }
            }
            // break;
        }

        unchanged_def
    }
    pub fn build_unchanged_use(&mut self) -> HashSet<(ObjPtr<LIRInst>, Reg)> {
        //首先根据call上下文初始化 unchanged use 和 unchanged def.这些告诉我们哪些寄存器不能够p2v
        let mut unchanged_use: HashSet<(ObjPtr<LIRInst>, Reg)> = HashSet::new();
        for bb in self.blocks.iter() {
            for (i, inst) in bb.insts.iter().enumerate() {
                if inst.get_type() != InstrsType::Call {
                    continue;
                }
                let mut used: HashSet<Reg> = inst.get_reg_use().iter().cloned().collect();
                for used in used.iter() {
                    unchanged_use.insert((*inst, *used));
                }
                if i != 0 {
                    let mut index: i32 = i as i32 - 1;
                    while index >= 0 && used.len() != 0 {
                        let inst = *bb.insts.get(index as usize).unwrap();
                        for reg_def in inst.get_reg_def() {
                            used.remove(&reg_def);
                        }
                        for reg_use in inst.get_reg_use() {
                            if used.contains(&reg_use) {
                                unchanged_use.insert((inst, reg_use));
                            }
                        }
                        if index == 0 {
                            break;
                        }
                        index -= 1;
                    }
                }
                if used.len() != 0 {
                    //TODO  (暂时不考虑 参数的加入不在同一个块中的情况)
                    //used 传递到前文的情况
                    let mut to_backward: LinkedList<(ObjPtr<BB>, Reg)> = LinkedList::new();
                    let mut backwarded: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
                    for reg_used in used.iter() {
                        for in_bb in bb.in_edge.iter() {
                            if !in_bb.live_out.contains(reg_used) {
                                continue;
                            }
                            // unreachable!();
                            to_backward.push_back((*in_bb, *reg_used));
                        }
                    }
                    while !to_backward.is_empty() {
                        let item = to_backward.pop_front().unwrap();
                        if backwarded.contains(&item) {
                            continue;
                        }
                        backwarded.insert(item);
                        let (bb, reg) = item;
                        let mut keep_backward = true;
                        for inst in bb.insts.iter().rev() {
                            if inst.get_reg_def().contains(&reg) {
                                keep_backward = false;
                                break;
                            }
                            if inst.get_reg_use().contains(&reg) {
                                unchanged_use.insert((*inst, reg));
                            }
                        }
                        if !keep_backward {
                            continue;
                        }
                        for in_bb in bb.in_edge.iter() {
                            if !in_bb.live_out.contains(&reg) {
                                continue;
                            }
                            to_backward.push_back((*in_bb, reg));
                        }
                    }
                    debug_assert!(to_backward.is_empty());
                }

                let mut defined: HashSet<Reg> = inst.get_reg_def().iter().cloned().collect();
                let mut index = i + 1;
                // 往后继块传递defined
                while index < bb.insts.len() && defined.len() != 0 {
                    let inst = *bb.insts.get(index).unwrap();
                    for reg in inst.get_reg_use() {
                        if defined.contains(&reg) {
                            unchanged_use.insert((inst, reg));
                        }
                    }
                    for reg in inst.get_reg_def() {
                        if defined.contains(&reg) {
                            defined.remove(&reg);
                        }
                    }
                    index += 1;
                }
                if defined.len() != 0 {
                    // 按照目前的代码结构来说不应该存在
                    // 说明define到了live out中(说明其他块使用了这个块中的计算出的a0)
                    // 则其他块中计算出的a0也应该使用相同的物理寄存器号(不应该改变)
                    let mut to_pass: LinkedList<(ObjPtr<BB>, Reg)> = LinkedList::new();
                    for out_bb in bb.out_edge.iter() {
                        for reg in defined.iter() {
                            if !out_bb.live_in.contains(reg) {
                                continue;
                            }
                            to_pass.push_back((*out_bb, *reg));
                        }
                    }
                    let mut passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
                    while !to_pass.is_empty() {
                        let (bb, reg) = to_pass.pop_front().unwrap();
                        if passed.contains(&(bb, reg)) {
                            continue;
                        }
                        passed.insert((bb, reg));
                        let mut index = 0;
                        while index < bb.insts.len() {
                            let inst = bb.insts.get(index).unwrap();
                            for use_reg in inst.get_reg_use() {
                                if use_reg == reg {
                                    unchanged_use.insert((*inst, reg));
                                }
                            }
                            if inst.get_reg_def().contains(&reg) {
                                break;
                            }
                            index += 1;
                        }

                        if index == bb.insts.len() {
                            //说明可能传到live out中
                            for out_bb in bb.out_edge.iter() {
                                if out_bb.live_in.contains(&reg) {
                                    to_pass.push_back((*out_bb, reg));
                                }
                            }
                        }
                    }
                }
            }
        }

        // 考虑ret
        // 一个block中只可能出现一条return最多
        let final_bb = self.get_final_bb();
        let last_inst = final_bb.insts.last().unwrap();
        let use_reg = last_inst.get_reg_use();
        debug_assert!(use_reg.len() == 1 || self.label != "main");
        if let Some(use_reg) = use_reg.get(0) {
            unchanged_use.insert((*last_inst, *use_reg));
            let mut back_bbs: LinkedList<ObjPtr<BB>> = LinkedList::new();
            back_bbs.push_back(final_bb);
            let mut passed = HashSet::new();
            while back_bbs.len() != 0 {
                let bb = back_bbs.pop_front().unwrap();
                if passed.contains(&bb) {
                    continue;
                }
                passed.insert(bb);
                let mut index = bb.insts.len() - 1;
                let mut if_finish = false;
                loop {
                    let inst = bb.insts.get(index).unwrap();
                    if inst.get_reg_def().contains(use_reg) {
                        if_finish = true;
                        break;
                    }
                    if inst.get_reg_use().contains(use_reg) {
                        unchanged_use.insert((*inst, *use_reg));
                    }
                    if index == 0 {
                        break;
                    }
                    index -= 1;
                }

                if !if_finish {
                    for in_bb in bb.in_edge.iter() {
                        debug_assert!(in_bb.live_out.contains(use_reg));
                        back_bbs.push_back(*in_bb);
                    }
                }
            }
            // break;
        }

        //考虑使用参数寄存器传参的情况,该情况只会发生在函数的第一个块
        //然后从entry块开始p2v
        let first_block = *self.entry.unwrap().out_edge.get(0).unwrap();
        let live_in: HashSet<Reg> = first_block.live_in.iter().cloned().collect();
        if live_in.len() != 0 {
            // println!("{}", first_block.label.clone());
            let mut args: HashSet<Reg> = Reg::get_all_args();
            args.retain(|reg| live_in.contains(reg));

            // println!("{}{:?}", first_block.label, args);
            //对于参数往后传递
            for (index, inst) in first_block.insts.iter().enumerate() {
                for reg_use in inst.get_reg_use() {
                    if args.contains(&reg_use) {
                        unchanged_use.insert((*inst, reg_use));
                        // println!("unchange used:{:?}\t{}\n", inst, reg_use);
                    }
                }
                for reg_def in inst.get_reg_def() {
                    args.remove(&reg_def);
                }
            }
            if args.len() != 0 {
                //可能传递到后面
                let mut to_pass: LinkedList<(ObjPtr<BB>, Reg)> = LinkedList::new();
                for arg in args.iter() {
                    for out_bb in first_block.out_edge.iter() {
                        if !out_bb.live_in.contains(arg) {
                            continue;
                        }
                        to_pass.push_back((*out_bb, *arg));
                    }
                }

                let mut passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
                while !to_pass.is_empty() {
                    let (bb, reg) = to_pass.pop_front().unwrap();
                    if passed.contains(&(bb, reg)) {
                        continue;
                    }
                    passed.insert((bb, reg));
                    let mut if_forward = true;
                    for inst in bb.insts.iter() {
                        if inst.get_reg_use().contains(&reg) {
                            unchanged_use.insert((*inst, reg));
                        }
                        if inst.get_reg_def().contains(&reg) {
                            if_forward = false;
                            break;
                        }
                    }
                    if !if_forward {
                        continue;
                    }
                    for out_bb in bb.out_edge.iter() {
                        if !out_bb.live_in.contains(&reg) {
                            continue;
                        }
                        to_pass.push_back((*out_bb, reg));
                    }
                }
                debug_assert!(to_pass.is_empty());
            }
        }

        if first_block.label == "params_mix" {
            //161
            let inst = first_block.insts.get(161).unwrap();
            let reg_use = inst.get_reg_use();
            debug_assert!(reg_use.contains(&Reg::get_a0()));
            debug_assert!(unchanged_use.contains(&(*inst, Reg::get_a0())));
        }
        unchanged_use
    }

    //返回p2v产生的新虚拟寄存器,以及该过程的动作序列
    // vregs  ,  (inst,p_reg,v_reg,def_or_use)
    pub fn p2v_2(&mut self, regs_to_decolor: HashSet<Reg>) -> HashSet<Reg> {
        //一种简单的p2v方式
        self.calc_live_base();
        let unchanged_def = self.build_unchanged_def();
        let unchanged_use = self.build_unchanged_use();
        let mut all_v_regs: HashSet<Reg> = HashSet::new();
        //以bb,inst,reg为遍历的基本单位
        //首先处理块间的物理寄存器
        //使用栈的方式, 先入后出,后入先出的原则
        let mut to_forward: LinkedList<(ObjPtr<BB>, Reg, Reg)> = LinkedList::new();
        let mut to_backward: LinkedList<(ObjPtr<BB>, Reg, Reg)> = LinkedList::new();
        let mut forward_passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
        let mut backward_passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
        //进行流处理 (以front作为栈顶)
        let process = |to_forward: &mut LinkedList<(ObjPtr<BB>, Reg, Reg)>,
                       to_backward: &mut LinkedList<(ObjPtr<BB>, Reg, Reg)>,
                       backward_passed: &mut HashSet<(ObjPtr<BB>, Reg)>,
                       forward_passed: &mut HashSet<(ObjPtr<BB>, Reg)>| {
            loop {
                while !to_forward.is_empty() {
                    let item = to_forward.pop_front().unwrap();
                    let key = (item.0, item.1);
                    if forward_passed.contains(&key) {
                        continue;
                    }
                    forward_passed.insert(key);
                    let (bb, p_reg, v_reg) = item;
                    //加入入口
                    for in_bb in bb.in_edge.iter() {
                        if in_bb.live_out.contains(&p_reg) {
                            to_backward.push_front((*in_bb, p_reg, v_reg));
                        }
                    }

                    let mut index = 0;
                    while index < bb.insts.len() {
                        let inst = bb.insts.get(index).unwrap();
                        if unchanged_use.contains(&(*inst, p_reg)) {
                            break;
                        }

                        if inst.get_reg_use().contains(&p_reg) {
                            inst.as_mut().replace_only_use_reg(&p_reg, &v_reg);
                        }
                        if inst.get_reg_def().contains(&p_reg) {
                            break;
                        }
                        index += 1;
                    }
                    if index == bb.insts.len() && bb.live_out.contains(&p_reg) {
                        for out_bb in bb.out_edge.iter() {
                            if out_bb.live_in.contains(&p_reg) {
                                to_forward.push_front((*out_bb, p_reg, v_reg));
                            }
                        }
                    }
                }
                while !to_backward.is_empty() {
                    let item = to_backward.pop_front().unwrap();
                    let key = (item.0, item.1);
                    if backward_passed.contains(&key) {
                        continue;
                    }
                    backward_passed.insert(key);
                    let (bb, p_reg, v_reg) = item;
                    for out_bb in bb.out_edge.iter() {
                        if out_bb.live_in.contains(&p_reg) {
                            to_forward.push_front((*out_bb, p_reg, v_reg));
                        }
                    }

                    let mut index = bb.insts.len();
                    let mut if_finish = false;
                    while index > 0 {
                        index -= 1;
                        let inst = bb.insts.get(index).unwrap();
                        if unchanged_def.contains(&(*inst, p_reg)) {
                            if_finish = true;
                            break;
                        }
                        if inst.get_reg_def().contains(&p_reg) {
                            inst.as_mut().replace_only_def_reg(&p_reg, &v_reg);
                            if_finish = true;
                            break;
                        }
                        if inst.get_reg_use().contains(&p_reg) {
                            inst.as_mut().replace_only_use_reg(&p_reg, &v_reg);
                        }
                    }
                    if !if_finish {
                        debug_assert!(bb.live_in.contains(&p_reg));
                        for in_bb in bb.in_edge.iter() {
                            if in_bb.live_out.contains(&p_reg) {
                                to_backward.push_front((*in_bb, p_reg, v_reg));
                            }
                        }
                    }
                }
                if to_forward.is_empty() && to_backward.is_empty() {
                    break;
                }
            }
        };
        for bb in self.blocks.iter() {
            for reg in bb.live_out.iter() {
                if !regs_to_decolor.contains(reg) {
                    continue;
                }
                let v_reg = Reg::init(reg.get_type());
                all_v_regs.insert(v_reg);
                to_backward.push_front((*bb, *reg, v_reg));
                for out_bb in bb.out_edge.iter() {
                    if out_bb.live_in.contains(reg) {
                        to_forward.push_front((*out_bb, *reg, v_reg));
                    }
                }
                process(
                    &mut to_forward,
                    &mut to_backward,
                    &mut backward_passed,
                    &mut forward_passed,
                );
            }
        }
        self.calc_live_base();
        //然后处理块内的物理寄存器
        //现在对于块内的物理寄存器没有能够超出范围的了
        for bb in self.blocks.iter() {
            //页内流
            let mut p2v: HashMap<Reg, Reg> = HashMap::new();
            for inst in bb.insts.iter() {
                let def = inst.get_reg_def();
                let used = inst.get_reg_use();
                for reg_use in used {
                    if !regs_to_decolor.contains(&reg_use)
                        || unchanged_use.contains(&(*inst, reg_use))
                        || !reg_use.is_physic()
                    {
                        continue;
                    }
                    inst.as_mut()
                        .replace_only_use_reg(&reg_use, p2v.get(&reg_use).unwrap())
                }
                for reg_def in def {
                    p2v.remove(&reg_def);
                    if !reg_def.is_physic()
                        || !regs_to_decolor.contains(&reg_def)
                        || unchanged_def.contains(&(*inst, reg_def))
                    {
                        continue;
                    }
                    let v_reg = Reg::init(reg_def.get_type());
                    all_v_regs.insert(v_reg);
                    p2v.insert(reg_def, v_reg);
                    inst.as_mut()
                        .replace_only_def_reg(&reg_def, p2v.get(&reg_def).unwrap());
                }
            }

            //对于最后存活的p2v寄存器
            for (p_reg, _) in p2v {
                debug_assert!(!bb.live_out.contains(&p_reg));
            }
        }
        all_v_regs
    }

    //把某条指令的某个物理寄存器给解着色,并返回产生的虚拟寄存器
    pub fn p2v_certain_reg_and_inst(
        bb: ObjPtr<BB>,
        index: usize,
        p_reg: &Reg,
        def_or_use: bool,
    ) -> Reg {
        debug_assert!(p_reg.is_physic());
        let mut v_reg = Reg::init(p_reg.get_type());
        let inst = bb.insts.get(index).unwrap();
        unimplemented!();
        debug_assert!(
            (def_or_use && inst.get_reg_def().contains(p_reg))
                || (!def_or_use && inst.get_reg_use().contains(p_reg))
        );
        if def_or_use {
            //作为def来解着色
            let mut index = index + 1;
            inst.as_mut().replace_only_def_reg(p_reg, &v_reg);
            while index < bb.insts.len() {
                let inst = bb.insts.get(index).unwrap();
                if inst.get_reg_use().contains(p_reg) {
                    inst.as_mut().replace_only_use_reg(p_reg, &v_reg);
                }
                if inst.get_reg_def().contains(&v_reg) {
                    break;
                }
                index += 1;
            }
            if index == bb.insts.len() && bb.live_out.contains(p_reg) {
                //正向,与反向
            }
        } else {
        }

        v_reg
    }
}
