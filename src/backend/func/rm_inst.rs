use super::*;

//关于无用指令消除的实现
impl Func {
    ///移除无用指令
    pub fn remove_unuse_inst(&mut self) {
        //TOCHECK
        // 移除mv va va 类型指令
        for bb in self.blocks.iter() {
            let mut index = 0;
            while index < bb.insts.len() {
                let inst = bb.insts[index];
                if inst.operands.len() < 2 {
                    index += 1;
                    continue;
                }
                let dst = inst.get_dst();
                let src = inst.get_lhs();
                if inst.get_type() == InstrsType::OpReg(SingleOp::Mv) && dst == src {
                    bb.as_mut().insts.remove(index);
                } else {
                    index += 1;
                }
            }
        }
        // 移除无用def
        self.remove_unuse_def();
    }

    ///移除无用def指令
    pub fn remove_unuse_def(&mut self) {
        loop {
            self.calc_live_for_alloc_reg();
            let mut if_finish = true;
            for bb in self.blocks.iter() {
                let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::with_capacity(bb.insts.len());
                let mut to_removed: HashSet<usize> = HashSet::new();
                let mut live_now: HashSet<Reg> = HashSet::new();
                bb.live_out.iter().for_each(|reg| {
                    live_now.insert(*reg);
                });
                //标记阶段 ,标记需要清除的指令
                for (index, inst) in bb.insts.iter().enumerate().rev() {
                    for reg in inst.get_reg_def() {
                        if !live_now.contains(&reg) && inst.get_type() != InstrsType::Call {
                            to_removed.insert(index);
                            break;
                        }
                        live_now.remove(&reg);
                    }
                    if to_removed.contains(&index) {
                        continue;
                    }
                    for reg in inst.get_reg_use() {
                        live_now.insert(reg);
                    }
                }
                //清楚阶段, 清除之前标记的指令
                for (index, inst) in bb.insts.iter().enumerate() {
                    if to_removed.contains(&index) {
                        if_finish = false;
                        log_file!(
                            "remove_unusedef.txt",
                            ":{}-{}:{}",
                            self.label,
                            bb.label,
                            inst.to_string()
                        );
                        continue;
                    }
                    new_insts.push(*inst);
                }
                bb.as_mut().insts = new_insts;
            }
            if if_finish {
                break;
            }
        }
        // self.print_func();
    }
}
