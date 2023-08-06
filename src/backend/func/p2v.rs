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
        let mut path_build_unchagned_def = "unchanged_def.txt";
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
                    //对于call指令使用到的寄存器,加入到列表中
                    for reg in used {
                        if !bb.live_in.contains(&reg) {
                            continue;
                        }
                        let mut to_backward = LinkedList::new();
                        to_backward.push_back(*bb);
                        let mut to_forward = LinkedList::new();
                        let mut forward_passed = HashSet::new();
                        let mut backward_passed = HashSet::new();
                        let find = Func::search_forward_and_backward_until_def(
                            &reg,
                            &mut to_forward,
                            &mut to_backward,
                            &mut backward_passed,
                            &mut forward_passed,
                        );
                        for (inst, if_def) in find {
                            if if_def {
                                unchanged_def.insert((inst, reg));
                            }
                        }
                    }
                }
                let mut defined: HashSet<Reg> = inst.get_reg_def().iter().cloned().collect();
                for reg in defined.iter().cloned() {
                    unchanged_def.insert((*inst, reg));
                }
                let mut index = i + 1;
                // 往后继块传递defined,往后面传递的话可能会发现其他块中存在的unchange def,但是其他块中的也是regdef?
                while index < bb.insts.len() && defined.len() != 0 {
                    let inst = *bb.insts.get(index).unwrap();
                    for reg in inst.get_reg_def() {
                        if defined.contains(&reg) {
                            defined.remove(&reg);
                        }
                    }
                    index += 1;
                }
                if defined.len() == 0 {
                    continue;
                }
                for reg in defined.iter() {
                    //理论上当前实现对于函数返回值的使用不应该传递到后面的基本块
                    debug_assert!(!bb.live_out.contains(reg));
                }
                for defined in defined.iter() {
                    if !bb.live_out.contains(defined) {
                        continue;
                    }
                    //找到相关的指令
                    let mut to_forward = LinkedList::new();
                    let mut to_backward = LinkedList::new();
                    to_backward.push_back(*bb);
                    let mut backward_passed = HashSet::new();
                    let mut forward_passed = HashSet::new();
                    let find = Func::search_forward_and_backward_until_def(
                        defined,
                        &mut to_forward,
                        &mut to_backward,
                        &mut backward_passed,
                        &mut forward_passed,
                    );
                    for (inst, if_def) in find.iter() {
                        if *if_def {
                            debug_assert!(|| -> bool {
                                match inst.get_type() {
                                    InstrsType::Call => true,
                                    _ => false,
                                }
                            }());
                            unchanged_def.insert((*inst, *defined));
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
            debug_assert!(match last_inst.get_type() {
                InstrsType::Ret(_) => true,
                _ => false,
            });
            debug_assert!(use_reg.is_physic());
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
                for used in used.iter().cloned() {
                    unchanged_use.insert((*inst, used));
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

                //used向前传递
                if used.len() != 0 {
                    //对于call指令使用到的寄存器,加入到列表中
                    for reg in used {
                        if !bb.live_in.contains(&reg) {
                            continue;
                        }
                        let mut to_backward = LinkedList::new();
                        to_backward.push_back(*bb);
                        let mut to_forward = LinkedList::new();
                        let mut forward_passed = HashSet::new();
                        let mut backward_passed = HashSet::new();
                        let find = Func::search_forward_and_backward_until_def(
                            &reg,
                            &mut to_forward,
                            &mut to_backward,
                            &mut backward_passed,
                            &mut forward_passed,
                        );
                        for (inst, if_def) in find {
                            if !if_def {
                                unchanged_use.insert((inst, reg));
                            }
                        }
                    }
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

                if defined.len() == 0 {
                    continue;
                }
                for reg in defined.iter() {
                    //理论上当前实现对于函数返回值的使用不应该传递到后面的基本块
                    debug_assert!(!bb.live_out.contains(reg));
                }
                for defined in defined.iter() {
                    if !bb.live_out.contains(defined) {
                        continue;
                    }
                    //找到相关的指令
                    let mut to_forward = LinkedList::new();
                    let mut to_backward = LinkedList::new();
                    to_backward.push_back(*bb);
                    let mut backward_passed = HashSet::new();
                    let mut forward_passed = HashSet::new();
                    let find = Func::search_forward_and_backward_until_def(
                        defined,
                        &mut to_forward,
                        &mut to_backward,
                        &mut backward_passed,
                        &mut forward_passed,
                    );
                    for (inst, if_def) in find.iter() {
                        if !*if_def {
                            unchanged_use.insert((*inst, *defined));
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
                    }
                }
                for reg_def in inst.get_reg_def() {
                    args.remove(&reg_def);
                }
            }
            if args.len() != 0 {
                //从该处往后搜索,搜索到的指令应该都是已经加入到的use，且不可能有def
                for arg in args.iter() {
                    // if liveou
                    let mut to_forward: LinkedList<ObjPtr<BB>> = LinkedList::new();
                    let mut to_backward = LinkedList::new();
                    let mut forward_passed: HashSet<ObjPtr<BB>> = HashSet::new();
                    let mut backward_passed: HashSet<ObjPtr<BB>> = HashSet::new();
                    to_backward.push_back(first_block);
                    let find = Func::search_forward_and_backward_until_def(
                        arg,
                        &mut to_forward,
                        &mut to_backward,
                        &mut backward_passed,
                        &mut forward_passed,
                    );
                    for (inst, if_def) in find {
                        //一定只有use
                        debug_assert!(!if_def, "{},{:?}", arg, {
                            Func::print_func(ObjPtr::new(self), "tmp2.txt");
                            inst
                        });
                        //一定在本块中,也就是一定加入过了,
                        debug_assert!(unchanged_use.contains(&(inst, *arg)));
                    }
                }
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
        let mut new_to_forward: HashSet<ObjPtr<BB>> = to_forward.iter().cloned().collect();
        let mut new_to_backward: HashSet<ObjPtr<BB>> = to_backward.iter().cloned().collect();
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
        loop {
            new_to_forward.clear();
            new_to_backward.clear();
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
                if index < bb.insts.len() {
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

        Func::print_func(ObjPtr::new(&self), "before_rm_between_blocks.txt");
        // let to_check = read_number_from_console();

        //进行流处理 (以front作为栈顶)
        //先解决块间问题
        for bb in self.blocks.iter() {
            for reg in bb.live_out.iter() {
                // if reg.get_id() == to_check {
                //     let a = 2;
                // }

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
                let mut to_forward = LinkedList::new();
                let mut to_backward = LinkedList::new();
                to_backward.push_back(*bb);
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
                    log_file!("btpassed.txt", "{}{}{}", self.label, bb.label, reg);
                }
                for bb in b_passed {
                    backward_passed.insert((bb, *reg));
                    log_file!("fwpassed.txt", "{}{}{}", self.label, bb.label, reg);
                }
                let mut if_p2v = true;
                for (inst, if_def) in find.iter() {
                    if (*if_def && unchanged_def.contains(&(*inst, *reg)))
                        || ((!*if_def) && unchanged_use.contains(&(*inst, *reg)))
                    {
                        log_file!(
                            "unchanged_for_p2v_between_blocks.txt",
                            "{}:{}{}",
                            bb.label,
                            reg,
                            inst.as_ref()
                        );
                        if_p2v = false;
                        break;
                    }
                }
                if !if_p2v {
                    continue;
                }
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
                self.calc_live_base();
            }
        }

        Func::print_func(ObjPtr::new(&self), "p2v_tmp.txt");
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
                debug_assert!(
                    !bb.live_out.contains(&p_reg),
                    "{},{},{}",
                    self.label,
                    bb.label,
                    p_reg
                );
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

use std::io::{self, BufRead};

fn read_number_from_console() -> i32 {
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut input = String::new();
        println!("请输入一个整数：");

        match handle.read_line(&mut input) {
            Ok(_) => match input.trim().parse::<i32>() {
                Ok(number) => return number,
                Err(_) => println!("输入不是一个有效的整数，请重新输入。"),
            },
            Err(error) => {
                eprintln!("读取输入失败：{}", error);
                std::process::exit(1);
            }
        }
    }
}
