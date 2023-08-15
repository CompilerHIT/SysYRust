use crate::log_file_uln;

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

/// p2v  以及 undo p2v
impl Func {
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
        // 打印unchanged_def
        log_file!("unchanged.txt", "func:{}", self.label);
        debug_assert!({
            for bb in self.blocks.iter() {
                log_file!("unchanged.txt", "\nbb:{}", bb.label);
                for inst in bb.insts.iter() {
                    for reg in inst.get_regs() {
                        if unchanged_use.contains(&(*inst, reg)) {
                            log_file!("unchanged.txt", "unchange use:{} -{}", inst.as_ref(), reg);
                        }
                        if unchanged_use.contains(&(*inst, reg)) {
                            log_file!("unchanged.txt", "unchange def:{} -{}", inst.as_ref(), reg);
                        }
                    }
                }
            }
            true
        });

        let mut all_v_regs: HashSet<Reg> = HashSet::new();
        let mut p2v_actions = Vec::new();
        Func::print_func(ObjPtr::new(&self), "before_p2v.txt");
        self.print_live_interval("live_interval_before_p2v.txt");
        // 处理块间
        // TODO,判断需要处理第二次块间是否是因为什么bug
        loop {
            self.calc_live_base();
            let old_v_len = all_v_regs.len();
            self.p2v_inter_blocks(
                &unchanged_def,
                &unchanged_use,
                regs_to_decolor,
                &mut p2v_actions,
                &mut all_v_regs,
            );
            // break;
            if old_v_len == all_v_regs.len() {
                break;
            }
        }
        self.calc_live_base();
        Func::print_func(ObjPtr::new(&self), "p2v_before_inner_block.txt");
        self.print_live_interval("live_interval_before_inner_p2v.txt");
        self.p2v_inner_blocks(
            &unchanged_def,
            &unchanged_use,
            regs_to_decolor,
            &mut p2v_actions,
            &mut all_v_regs,
        );
        (all_v_regs, p2v_actions)
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

/// build unchanged def, build unchanged use
impl Func {
    ///该函数计算结果依赖外部调用的calc liv3
    pub fn build_unchanged_def(&mut self) -> HashSet<(ObjPtr<LIRInst>, Reg)> {
        // self.print_func();
        //call指令存在unchanged def ;call指令的 use会传递到unchanged def
        let mut unchanged_def: HashSet<(ObjPtr<LIRInst>, Reg)> = HashSet::new();
        // let mut path_build_unchagned_def = "unchanged_def.txt";
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
                            if used.contains(&reg_def) {
                                used.remove(&reg_def);
                                unchanged_def.insert((inst, reg_def));
                            }
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
                        //
                        let (_, _, find) = Func::search_from_in_edge(bb, &reg);
                        for (inst, if_def) in find {
                            if if_def {
                                unchanged_def.insert((inst, reg));
                            }
                        }
                    }
                }
                // used 寄存器还应该往前搜索,使用到相同寄存器的指令也不应该def
                let mut used: HashSet<Reg> = inst.get_reg_use().iter().cloned().collect();
                {
                    let mut index = i;
                    while index < bb.insts.len() && !used.is_empty() {
                        let inst = bb.insts.get(index).unwrap();
                        for reg in inst.get_reg_def() {
                            used.remove(&reg);
                        }
                        index += 1;
                    }

                    if !used.is_empty() {
                        // 往后寻找unchanged
                        used.retain(|reg| bb.live_out.contains(reg));
                        for reg_used in used {
                            let (_, _, mut find) = Func::search_from_out_edge(bb, &reg_used);
                            find.retain(|(_, if_def)| *if_def);
                            for (inst, if_def) in find {
                                unchanged_def.insert((inst, reg_used));
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
                defined.retain(|reg| bb.live_out.contains(reg));
                for defined in defined.iter() {
                    let (_, _, mut find) = Func::search_from_out_edge(bb, defined);
                    find.retain(|(_, if_def)| *if_def);
                    for (inst, if_def) in find.iter() {
                        unchanged_def.insert((*inst, *defined));
                    }
                }
            }
        }

        //考虑参数寄存器引入的def
        //考虑使用参数寄存器传参的情况,该情况只会发生在函数的第一个块
        //然后从entry块开始p2v
        let first_block = *self.entry.unwrap().out_edge.get(0).unwrap();
        let live_in: HashSet<Reg> = first_block.live_in.iter().cloned().collect();
        if live_in.len() != 0 {
            // println!("{}", first_block.label.clone());
            let mut args: HashSet<Reg> = Reg::get_all_args();
            args.retain(|reg| live_in.contains(reg));
            //对于参数从前往后传递
            for (_, inst) in first_block.insts.iter().enumerate() {
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
                        if if_def {
                            unchanged_def.insert((inst, *arg));
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
            //对于块内的情况
            let mut index = final_bb.insts.len();
            let mut if_search_in_edge = true;
            while index > 0 {
                index -= 1;
                let inst = final_bb.insts.get(index).unwrap();
                if inst.get_reg_def().contains(use_reg) {
                    unchanged_def.insert((*inst, *use_reg));
                    if_search_in_edge = false;
                    break;
                }
                if index == 0 {
                    break;
                }
            }
            if if_search_in_edge {
                let (_, _, find) = Func::search_from_in_edge(&final_bb, use_reg);
                for (inst, if_def) in find {
                    if if_def {
                        unchanged_def.insert((inst, *use_reg));
                    }
                }
            }
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
                        let (_, _, mut find) = Func::search_from_in_edge(bb, &reg);
                        find.retain(|(inst, if_def)| !if_def);
                        for (inst, _) in find {
                            unchanged_use.insert((inst, reg));
                        }
                    }
                }

                // used 往后传递
                let mut used: HashSet<Reg> = inst.get_reg_use().iter().cloned().collect();
                {
                    let mut index = i;
                    while index < bb.insts.len() && !used.is_empty() {
                        let inst = bb.insts.get(index).unwrap();
                        for reg in inst.get_reg_use() {
                            if used.contains(&reg) {
                                unchanged_use.insert((*inst, reg));
                            }
                        }
                        for reg in inst.get_reg_def() {
                            used.remove(&reg);
                        }
                        index += 1;
                    }

                    if !used.is_empty() {
                        // 往后寻找unchanged
                        used.retain(|reg| bb.live_out.contains(reg));
                        for reg_used in used {
                            let (_, _, mut find) = Func::search_from_out_edge(bb, &reg_used);
                            find.retain(|(inst, if_def)| !*if_def);
                            for (inst, _) in find {
                                unchanged_use.insert((inst, reg_used));
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
                defined.retain(|reg| bb.live_out.contains(reg));
                for defined in defined.iter() {
                    let (_, _, mut find) = Func::search_from_out_edge(bb, defined);
                    find.retain(|(_, if_def)| !*if_def);
                    for (inst, _) in find.iter() {
                        unchanged_use.insert((*inst, *defined));
                    }
                }
            }
        }

        //考虑使用参数寄存器传参的情况,该情况只会发生在函数的第一个块
        //然后从entry块开始p2v
        let first_block = *self.entry.unwrap().out_edge.get(0).unwrap();
        let live_in: HashSet<Reg> = first_block.live_in.iter().cloned().collect();
        if live_in.len() != 0 {
            let mut args: HashSet<Reg> = Reg::get_all_args();
            args.retain(|reg| live_in.contains(reg));
            // println!("{}{:?}", first_block.label, args);
            //对于参数往后传递
            for (_, inst) in first_block.insts.iter().enumerate() {
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
                        if !if_def {
                            unchanged_use.insert((inst, *arg));
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
            //对于块内的情况
            unchanged_use.insert((*last_inst, *use_reg));
            let mut index = final_bb.insts.len();
            let mut if_search_in_edge = true;
            while index > 0 {
                index -= 1;
                let inst = final_bb.insts.get(index).unwrap();
                if inst.get_reg_def().contains(use_reg) {
                    if_search_in_edge = false;
                    break;
                }
                if inst.get_reg_use().contains(use_reg) {
                    unchanged_use.insert((*inst, *use_reg));
                }
                if index == 0 {
                    break;
                }
            }

            if if_search_in_edge {
                debug_assert!(final_bb.live_in.contains(use_reg));
                let (_, _, find) = Func::search_from_in_edge(&final_bb, use_reg);
                for (inst, if_def) in find {
                    if !if_def {
                        unchanged_use.insert((inst, *use_reg));
                    }
                }
            }
        }

        unchanged_use
    }
}

/// p2v的块间部分和块内部分
impl Func {
    ///块内p2v
    fn p2v_inner_blocks(
        &mut self,
        unchanged_def: &HashSet<(ObjPtr<LIRInst>, Reg)>,
        unchanged_use: &HashSet<(ObjPtr<LIRInst>, Reg)>,
        regs_to_decolor: &HashSet<Reg>,
        p2v_actions: &mut Vec<(ObjPtr<LIRInst>, Reg, Reg, bool)>,
        all_v_regs: &mut HashSet<Reg>,
    ) {
        for bb in self.blocks.iter() {
            //页内流
            let mut p2v: HashMap<Reg, Reg> = HashMap::new();
            for inst in bb.insts.iter() {
                let def = inst.get_reg_def();
                let used = inst.get_reg_use();
                for reg_use in used {
                    if (!regs_to_decolor.contains(&reg_use))
                        || (unchanged_use.contains(&(*inst, reg_use)))
                        || (!reg_use.is_physic())
                    {
                        continue;
                    }
                    debug_assert!(
                        p2v.contains_key(&reg_use),
                        "{},{},{}",
                        bb.label,
                        inst.to_string(),
                        {
                            Func::print_func(ObjPtr::new(self), "bad1.txt");
                            reg_use
                        }
                    );
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
    }

    ///块间P2v
    fn p2v_inter_blocks(
        &mut self,
        unchanged_def: &HashSet<(ObjPtr<LIRInst>, Reg)>,
        unchanged_use: &HashSet<(ObjPtr<LIRInst>, Reg)>,
        regs_to_decolor: &HashSet<Reg>,
        p2v_actions: &mut Vec<(ObjPtr<LIRInst>, Reg, Reg, bool)>,
        all_v_regs: &mut HashSet<Reg>,
    ) {
        let path = "p2v_actions_inter_blocks.txt";
        log_file!(path, "func:{}", self.label);
        // 处理块间出现的寄存器的使用
        // 对于出现在live_in中的寄存器
        let mut to_process: Vec<(ObjPtr<BB>, Reg)> = Vec::with_capacity(self.blocks.len() * 60);
        for bb in self.blocks.iter() {
            for reg in bb.live_out.iter() {
                if !reg.is_physic() || !regs_to_decolor.contains(reg) {
                    continue;
                }
                to_process.push((*bb, *reg));
            }
        }
        let mut backward_bb_reg_passed: HashSet<(ObjPtr<BB>, Reg)> = HashSet::new();
        for (bb, reg) in to_process.iter() {
            if backward_bb_reg_passed.contains(&(*bb, *reg)) {
                continue;
            }
            let (forward_passed, backward_passed, find) = Func::search_from_out_edge(bb, reg);
            // 打印搜索过的路径
            log_file!(path, "\n\nsearch {} from {}", reg, bb.label);
            for b_bb in backward_passed.iter() {
                log_file!(path, "backward search:{}", b_bb.label);
            }
            for f_bb in forward_passed.iter() {
                log_file!(path, "forward  search:{}", f_bb.label);
            }
            // 记录搜索过的路径,搜索过的路径之后不再搜索
            for backward_bb in backward_passed {
                backward_bb_reg_passed.insert((backward_bb, *reg));
            }
            let mut if_replace = true;
            for (inst, if_def) in find.iter() {
                if *if_def && unchanged_def.contains(&(*inst, *reg)) {
                    if_replace = false;
                    log_file!(path, "find unchanged def: {} -{},stop!", inst.as_ref(), reg);
                    break;
                } else if !*if_def && unchanged_use.contains(&(*inst, *reg)) {
                    if_replace = false;
                    log_file!(path, "find unchanged use: {} -{},stop!", inst.as_ref(), reg);
                    break;
                }
            }
            if !if_replace {
                log_file!(path, "p2v {} from {} fail!", reg, bb.label);
                continue;
            }
            log_file!(path, "p2v {} from {} success!", reg, bb.label);
            // 替换
            let new_v = Reg::init(reg.get_type());
            all_v_regs.insert(new_v);
            for (inst, if_def) in find {
                p2v_actions.push((inst, *reg, new_v, if_def));
                log_file_uln!(path, "p2v {} ", inst.as_ref());
                if if_def {
                    inst.as_mut().replace_only_def_reg(reg, &new_v);
                } else {
                    inst.as_mut().replace_only_use_reg(reg, &new_v);
                }
                log_file!(path, "to {} ", inst.as_ref());
            }
        }
    }
}

///
///从指定块得 inedge/outedges 出发寻找指定寄存器的某个值的所有可能定义和使用
/// 搜索包括自身
impl Func {
    /// 从某个块的in edge 搜索,寻找该寄存器当前值得所有可能定义以及所有可能使用
    pub fn search_from_in_edge(
        bb: &ObjPtr<BB>,
        reg: &Reg,
    ) -> (
        HashSet<ObjPtr<BB>>,
        HashSet<ObjPtr<BB>>,
        Vec<(ObjPtr<LIRInst>, bool)>,
    ) {
        debug_assert!(bb.live_in.contains(reg));
        let mut to_forward: LinkedList<ObjPtr<BB>> = LinkedList::new();
        let mut to_backward: LinkedList<ObjPtr<BB>> = LinkedList::new();
        let mut forward_passed: HashSet<ObjPtr<BB>> = HashSet::new();
        let mut backward_passed: HashSet<ObjPtr<BB>> = HashSet::new();
        for in_bb in bb.in_edge.iter() {
            debug_assert!(in_bb.live_out.contains(reg));
            to_backward.push_back(*in_bb);
        }
        let find = Func::search_forward_and_backward_until_def(
            reg,
            &mut to_forward,
            &mut to_backward,
            &mut backward_passed,
            &mut forward_passed,
        );
        (forward_passed, backward_passed, find)
    }

    /// 从某个块的out edge搜索,寻找该寄存器当前值得所有可能定义以及所有可能使用
    pub fn search_from_out_edge(
        bb: &ObjPtr<BB>,
        reg: &Reg,
    ) -> (
        HashSet<ObjPtr<BB>>,
        HashSet<ObjPtr<BB>>,
        Vec<(ObjPtr<LIRInst>, bool)>,
    ) {
        debug_assert!(bb.live_out.contains(reg));
        let mut to_forward: LinkedList<ObjPtr<BB>> = LinkedList::new();
        let mut to_backward: LinkedList<ObjPtr<BB>> = LinkedList::new();
        let mut forward_passed: HashSet<ObjPtr<BB>> = HashSet::new();
        let mut backward_passed: HashSet<ObjPtr<BB>> = HashSet::new();
        for out_bb in bb.out_edge.iter() {
            if out_bb.live_in.contains(reg) {
                to_forward.push_back(*out_bb);
            }
        }
        let find = Func::search_forward_and_backward_until_def(
            reg,
            &mut to_forward,
            &mut to_backward,
            &mut backward_passed,
            &mut forward_passed,
        );
        (forward_passed, backward_passed, find)
    }
}

/// p2v 依赖的指令搜索方法
impl Func {
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
}
