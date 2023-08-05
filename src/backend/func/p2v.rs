use super::*;

///寄存器重分配相关接口的实现
impl Func {
    ///p_to_v
    ///把函数中所有在regs中的物理寄存器进行ptov(除了call指令def和call指令use的寄存器)<br>
    /// 该行为需要在handle call之前执行 (在这个试图看来,一个call前后除了a0的值可能发生改变,其他寄存器的值并不会发生改变)
    ///因为在handle call后有有些寄存器需要通过栈来restore,暂时还没有分析这个行为
    /// 该函数会绝对保留原本程序的结构，并且不会通过构造phi等行为增加指令,不会调整指令顺序,不会合并寄存器等等
    pub fn p2v_pre_handle_call(&mut self, regs_to_decolor: &HashSet<Reg>) -> HashSet<Reg> {
        self.p2v(regs_to_decolor).0
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
                let mut index = bb.insts.len();
                let mut if_finish = false;
                while index > 0 {
                    index -= 1;
                    let inst = bb.insts.get(index).unwrap();
                    if inst.get_reg_def().contains(use_reg) {
                        unchanged_def.insert((*inst, *use_reg));
                        if_finish = true;
                        break;
                    }
                }
                if !if_finish {
                    for in_bb in bb.in_edge.iter() {
                        debug_assert!(in_bb.live_out.contains(use_reg));
                        if in_bb.live_out.contains(use_reg) {
                            back_bbs.push_back(*in_bb);
                        }
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
                let mut index = bb.insts.len();
                let mut if_finish = false;
                while index > 0 {
                    index -= 1;
                    let inst = bb.insts.get(index).unwrap();
                    if inst.get_reg_def().contains(use_reg) {
                        if_finish = true;
                        break;
                    }
                    if inst.get_reg_use().contains(use_reg) {
                        unchanged_use.insert((*inst, *use_reg));
                    }
                }

                if !if_finish {
                    for in_bb in bb.in_edge.iter() {
                        if in_bb.live_out.contains(use_reg) {
                            back_bbs.push_back(*in_bb);
                        }
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

        unchanged_use
    }

    ///search_forward_backward,前向搜索直到遇到def(不包含),反向搜索直到遇见def(包含)
    /// 返回 (前向搜索得到的相关指令, 反向搜索得到的相关指令)
    pub fn search_forward_and_backward_until_def(
        reg: &Reg,
        to_forward: &mut LinkedList<ObjPtr<BB>>,
        to_backward: &mut LinkedList<ObjPtr<BB>>,
        backward_passed: &mut HashSet<ObjPtr<BB>>,
        forward_passed: &mut HashSet<ObjPtr<BB>>,
    ) -> Vec<(ObjPtr<LIRInst>, bool)> {
        let mut find = Vec::new();
        loop {
            let mut new_to_forward = HashSet::new();
            let mut new_to_backward = HashSet::new();
            while !to_forward.is_empty() {
                let bb = to_forward.pop_front().unwrap();
                if forward_passed.contains(&bb) {
                    continue;
                }
                forward_passed.insert(bb);
                let mut index = 0;
                while index < bb.insts.len() {
                    let inst = bb.insts.get(index).unwrap();
                    if inst.get_reg_use().contains(reg) {
                        find.push((*inst, false));
                    }
                    if inst.get_reg_def().contains(reg) {
                        break;
                    }
                    index += 1;
                }
                if index != bb.insts.len() {
                    continue;
                }
                //读到尽头
                for out_bb in bb.out_edge.iter() {
                    if out_bb.live_in.contains(reg) {
                        new_to_forward.insert(*out_bb);
                    }
                }
            }
            while !to_backward.is_empty() {
                let bb = to_backward.pop_front().unwrap();
                if backward_passed.contains(&bb) {
                    continue;
                }
                backward_passed.insert(bb);
                let mut index = bb.insts.len();
                let mut if_continue = true;
                while index > 0 {
                    index -= 1;
                    let inst = bb.insts.get(index).unwrap();
                    if inst.get_reg_def().contains(reg) {
                        find.push((*inst, true));
                        if_continue = false;
                        break;
                    }
                    if inst.get_reg_use().contains(reg) {
                        find.push((*inst, false));
                    }
                }
                if !if_continue {
                    continue;
                }
                debug_assert!(index == 0);
                for in_bb in bb.in_edge.iter() {
                    if in_bb.live_out.contains(reg) {
                        new_to_backward.insert(*in_bb);
                    }
                }
            }

            //计算新的 to forward和tobackward
            new_to_forward.retain(|bb| !forward_passed.contains(bb));
            new_to_backward.retain(|bb| !backward_passed.contains(bb));

            loop {
                let old_to_forward_len = new_to_forward.len();
                let old_to_backward_len = new_to_backward.len();
                for nf in new_to_forward.iter() {
                    for in_bb in nf.in_edge.iter() {
                        if in_bb.live_out.contains(reg) && !backward_passed.contains(in_bb) {
                            new_to_backward.insert(*in_bb);
                        }
                    }
                }
                for nb in new_to_backward.iter() {
                    for out_bb in nb.out_edge.iter() {
                        if out_bb.live_in.contains(reg) && !forward_passed.contains(out_bb) {
                            new_to_forward.insert(*out_bb);
                        }
                    }
                }

                if old_to_backward_len == new_to_backward.len()
                    && (old_to_forward_len == new_to_forward.len())
                {
                    break;
                }
            }
            to_forward.extend(new_to_forward.iter());
            to_backward.extend(new_to_backward.iter());
            if to_forward.is_empty() && to_backward.is_empty() {
                break;
            }
        }
        find
    }

    //返回p2v产生的新虚拟寄存器,以及该过程的动作序列
    // vregs  ,  (inst,p_reg,v_reg,def_or_use)
    pub fn p2v(
        &mut self,
        regs_to_decolor: &HashSet<Reg>,
    ) -> (HashSet<Reg>, Vec<(ObjPtr<LIRInst>, Reg, Reg, bool)>) {
        //一种简单的p2v方式
        self.calc_live_base();
        let unchanged_def = self.build_unchanged_def();
        let unchanged_use = self.build_unchanged_use();
        let mut all_v_regs: HashSet<Reg> = HashSet::new();
        let mut p2v_actions = Vec::new();
        //以bb,inst,reg为遍历的基本单位
        //首先处理块间的物理寄存器
        //使用栈的方式, 先入后出,后入先出的原则
        let mut forward_passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
        let mut backward_passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
        //进行流处理 (以front作为栈顶)
        //先解决块间问题
        for bb in self.blocks.iter() {
            for reg in bb.live_out.iter() {
                if !regs_to_decolor.contains(reg) {
                    continue;
                }
                if !reg.is_physic() {
                    continue;
                }

                if backward_passed.contains(&(*bb, *reg)) {
                    continue;
                }
                //剩下部分用来着色,不同的块间不同的寄存器的反复着色状态不同
                //首先判断该reg是否能够p2v
                let mut to_forward = HashSet::new();
                let mut to_backward = HashSet::new();
                to_backward.insert(*bb);
                //找齐所有出入
                loop {
                    let old_backward_len = to_backward.len();
                    let old_forward_len = to_forward.len();
                    for back_bb in to_backward.iter() {
                        for out_bb in back_bb.out_edge.iter() {
                            if out_bb.live_in.contains(reg) {
                                to_forward.insert(*out_bb);
                            }
                        }
                    }
                    for forward_bb in to_forward.iter() {
                        for in_bb in forward_bb.in_edge.iter() {
                            if in_bb.live_out.contains(reg) {
                                to_backward.insert(*in_bb);
                            }
                        }
                    }
                    if old_backward_len == to_backward.len() && old_forward_len == to_forward.len()
                    {
                        break;
                    }
                }
                let mut to_forward = to_forward.iter().cloned().collect();
                let mut to_backward = to_backward.iter().cloned().collect();
                let mut f_passed = HashSet::new();
                let mut b_passed = HashSet::new();
                let find = Func::search_forward_and_backward_until_def(
                    reg,
                    &mut to_forward,
                    &mut to_backward,
                    &mut b_passed,
                    &mut f_passed,
                );
                //把内容加入passed表
                for bb in f_passed {
                    forward_passed.insert((bb, *reg));
                }
                for bb in b_passed {
                    backward_passed.insert((bb, *reg));
                }
                let mut if_p2v = true;
                for (inst, if_def) in find.iter() {
                    if *if_def && unchanged_def.contains(&(*inst, *reg))
                        || (!*if_def) && unchanged_use.contains(&(*inst, *reg))
                    {
                        log_file!(
                            "unchanged_for_p2v_between_blocks.txt",
                            "{}{}",
                            reg,
                            inst.as_ref()
                        );
                        if_p2v = false;
                        break;
                    }
                }
                if if_p2v {
                    let v_reg = Reg::init(reg.get_type());
                    all_v_regs.insert(v_reg);
                    for (inst, if_def) in find {
                        p2v_actions.push((inst, *reg, v_reg, if_def));
                        if if_def {
                            inst.as_mut().replace_only_def_reg(reg, &v_reg);
                        } else {
                            inst.as_mut().replace_only_use_reg(reg, &v_reg);
                        }
                    }
                }
            }
        }
        self.calc_live_base();

        // Func::print_func(ObjPtr::new(&self), "p2v_tmp.txt");
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
                    let v_reg = p2v.get(&reg_use).unwrap();
                    p2v_actions.push((*inst, reg_use, *v_reg, false));
                    inst.as_mut().replace_only_use_reg(&reg_use, v_reg);
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
                    p2v_actions.push((*inst, reg_def, v_reg, true));
                    inst.as_mut()
                        .replace_only_def_reg(&reg_def, p2v.get(&reg_def).unwrap());
                }
            }

            //对于最后存活的p2v寄存器
            for (p_reg, _) in p2v {
                debug_assert!(!bb.live_out.contains(&p_reg), "{},{}", bb.label, p_reg);
            }
        }
        (all_v_regs, p2v_actions)
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

    pub fn undo_p2v(p2v_actions: &Vec<(ObjPtr<LIRInst>, Reg, Reg, bool)>) {
        for (inst, p_reg, v_reg, if_def) in p2v_actions {
            if *if_def {
                inst.as_mut().replace_only_def_reg(v_reg, p_reg);
            } else {
                inst.as_mut().replace_only_use_reg(v_reg, p_reg);
            }
        }
    }
}
