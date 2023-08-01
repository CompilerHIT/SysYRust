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
