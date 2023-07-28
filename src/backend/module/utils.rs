use super::*;

///一些进行分析需要用到的工具
impl AsmModule {
    pub fn analyse_inst_with_live_now(
        func: &Func,
        inst_analyser: &mut dyn FnMut(ObjPtr<LIRInst>, &HashSet<Reg>),
    ) {
        for bb in func.blocks.iter() {
            let mut livenow: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                livenow.insert(*reg);
            });
            for inst in bb.insts.iter().rev() {
                for reg in inst.get_reg_def() {
                    livenow.remove(&reg);
                }
                //
                inst_analyser(*inst, &livenow);
                for reg in inst.get_reg_use() {
                    livenow.insert(reg);
                }
            }
        }
    }
    pub fn analyse_inst_with_index_and_live_now(
        func: &Func,
        inst_analyser: &mut dyn FnMut(ObjPtr<LIRInst>, usize, &HashSet<Reg>, ObjPtr<BB>),
    ) {
        for bb in func.blocks.iter() {
            let mut livenow: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                livenow.insert(*reg);
            });
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_def() {
                    livenow.remove(&reg);
                }
                //
                inst_analyser(*inst, index, &livenow, *bb);
                for reg in inst.get_reg_use() {
                    livenow.insert(reg);
                }
            }
        }
    }

    pub fn iter_insts(func: ObjPtr<Func>, processor: &mut dyn FnMut(&ObjPtr<LIRInst>)) {
        for bb in func.blocks.iter() {
            for inst in bb.insts.iter() {
                processor(inst);
            }
        }
    }

    //从某个指令出发,往左右两边延申,反着色某个物理寄存器 (返回去色成功后的指令列表,(所有涉及去色的指令))
    pub fn get_to_recolor(
        bb: ObjPtr<BB>,
        index: usize,
        p_reg: Reg,
    ) -> Vec<(ObjPtr<LIRInst>, bool)> {
        let get_to_recolor_path = "get_to_recolor.txt";
        log_file!(get_to_recolor_path, "start recolor:{}:{}", bb.label, p_reg);
        let mut decolored_insts = Vec::new();
        let mut to_pass: LinkedList<(ObjPtr<BB>, i32, i32)> = LinkedList::new();
        let mut passed = HashSet::new();
        if bb.insts.len() > index && bb.insts.get(index).unwrap().get_reg_def().contains(&p_reg) {
            unreachable!();
            // decolored_insts.push((*bb.insts.get(index).unwrap(),));
        } else {
            to_pass.push_back((bb, index as i32, -1));
        }
        to_pass.push_back((bb, index as i32, 1));
        while !to_pass.is_empty() {
            let (bb, mut index, refresh) = to_pass.pop_front().unwrap();
            if passed.contains(&(bb, index, refresh)) {
                continue;
            }
            // println!("{}:{}:{}", bb.label, index, refresh);
            log_file!(get_to_recolor_path, "{}:{}:{}", bb.label, index, refresh);
            passed.insert((bb, index, refresh));
            index += refresh;
            while index >= 0 && index < bb.insts.len() as i32 {
                log_file!(get_to_recolor_path, "{}:{}:{}", bb.label, index, refresh);
                passed.insert((bb, index, refresh));
                let inst = bb.insts.get(index as usize).unwrap();
                if refresh == 1 {
                    if inst.get_reg_use().contains(&p_reg) {
                        decolored_insts.push((*inst, false));
                        log_file!(get_to_recolor_path, "{}", inst.as_ref());
                    }
                    if inst.get_reg_def().contains(&p_reg) {
                        break;
                    }
                } else if refresh == -1 {
                    if inst.get_reg_def().contains(&p_reg) {
                        decolored_insts.push((*inst, true));
                        log_file!(get_to_recolor_path, "{}", inst.as_ref());
                        break;
                    }
                    if inst.get_reg_use().contains(&p_reg) {
                        decolored_insts.push((*inst, false));
                        log_file!(get_to_recolor_path, "{}", inst.as_ref());
                    }
                } else {
                    unreachable!()
                }
                index += refresh;
            }
            if index >= 0 && index < bb.insts.len() as i32 {
                continue;
            }
            //加入新的块
            let mut new_forward = HashSet::new();
            let mut new_backward = HashSet::new();
            if index < 0 {
                log_file!(get_to_recolor_path, "expand backward");
                for in_bb in bb.in_edge.iter() {
                    log_file!(
                        get_to_recolor_path,
                        "{}'s live out:{:?}",
                        in_bb.label,
                        in_bb.live_out
                    );
                    if in_bb.live_out.contains(&p_reg) {
                        new_backward.insert((*in_bb, in_bb.insts.len() as i32, -1));
                    }
                }
            } else {
                log_file!(get_to_recolor_path, "expand forward");
                for out_bb in bb.out_edge.iter() {
                    log_file!(
                        get_to_recolor_path,
                        "{}'s live in:{:?}",
                        out_bb.label,
                        out_bb.live_in
                    );
                    if out_bb.live_in.contains(&p_reg) {
                        new_forward.insert((*out_bb, -1, 1));
                    }
                }
            }
            log_file!(get_to_recolor_path, "expand backward");
            for (bb, _, _) in new_forward.iter() {
                for in_bb in bb.in_edge.iter() {
                    if in_bb.live_out.contains(&p_reg) {
                        new_backward.insert((*in_bb, in_bb.insts.len() as i32, -1));
                    }
                }
            }
            log_file!(get_to_recolor_path, "expand forward");
            for (bb, _, _) in new_backward.iter() {
                for out_bb in bb.out_edge.iter() {
                    if out_bb.live_in.contains(&p_reg) {
                        new_forward.insert((*out_bb, -1, 1));
                    }
                }
            }
            // todo!();
            for forward in new_forward {
                to_pass.push_back(forward);
            }
            for backward in new_backward {
                to_pass.push_back(backward);
            }
        }

        decolored_insts
    }
}

impl AsmModule {
    pub fn print_func(&self) {
        // // debug_assert!(false, "{}", self.name_func.len());
        for (_, func) in self.name_func.iter() {
            if func.is_extern {
                continue;
            }
            Func::print_func(*func, "print_all_funcs.txt");
        }
    }
}
