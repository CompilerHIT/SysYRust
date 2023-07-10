use crate::backend::regalloc;

use super::*;

impl BackendPass {
    pub fn clear_pass(&mut self) {
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    self.rm_useless(*block);
                });
                self.rm_useless_def(func.clone());
                self.rm_repeated_sl(func.clone());
            }
        });
    }
    ///移除重复的load语句和store语句
    fn rm_repeated_sl(&self, func: ObjPtr<Func>) {
        // 删除
        todo!()
    }

    fn rm_useless_def(&self, func: ObjPtr<Func>) {
        let ends_index_bb = regalloc::regalloc::build_ends_index_bb(func.as_ref());
        for bb in func.blocks.iter() {
            let mut rm_num = 0; //已经删除掉的指令
            let mut index = 0; //当前到达的指令的位置
            loop {
                if index >= bb.insts.len() {
                    break;
                }
                // 获取当前指令实际对应的下标
                let real_index = index + rm_num;
                let inst = bb.insts.get(index).unwrap();
                let reg = inst.get_reg_def();
                if reg.is_empty() {
                    index += 1;
                    continue;
                }
                let reg = reg.get(0).unwrap();
                let ends = ends_index_bb.get(&(real_index as i32, *bb));
                if ends.is_none() {
                    index += 1;
                    continue;
                }
                let ends = ends.unwrap();
                if !ends.contains(reg) {
                    index += 1;
                    continue;
                }
                bb.as_mut().insts.remove(index);
                rm_num += 1;
            }
        }
    }

    fn rm_useless(&self, block: ObjPtr<BB>) {
        let mut index = 0;
        loop {
            if index >= block.insts.len() {
                break;
            }
            let inst = block.insts[index];
            if self.is_mv_same(inst) {
                block.as_mut().insts.remove(index);
                continue;
            }
            if index > 0 {
                let prev_inst = block.insts[index - 1];
                if self.is_sl_same(inst, prev_inst) {
                    block.as_mut().insts.remove(index);
                    continue;
                }
                if self.is_sl_same_offset(inst, prev_inst) {
                    inst.as_mut().replace_kind(InstrsType::OpReg(SingleOp::Mv));
                    inst.as_mut()
                        .replace_op(vec![inst.get_dst().clone(), prev_inst.get_dst().clone()]);
                    index += 1;
                    continue;
                }
            }
            index += 1;
        }
    }
    fn is_mv_same(&self, inst: ObjPtr<LIRInst>) -> bool {
        if inst.get_type() == InstrsType::OpReg(SingleOp::Mv) {
            if inst.get_dst() == inst.get_lhs() {
                return true;
            }
        }
        false
    }

    fn is_sl_same(&self, inst: ObjPtr<LIRInst>, prev_inst: ObjPtr<LIRInst>) -> bool {
        if self.is_sl_same_offset(inst, prev_inst) && inst.get_dst() == prev_inst.get_dst() {
            return true;
        }
        false
    }

    fn is_sl_same_offset(&self, inst: ObjPtr<LIRInst>, prev_inst: ObjPtr<LIRInst>) -> bool {
        if inst.get_type() == InstrsType::LoadFromStack
            && prev_inst.get_type() == InstrsType::StoreToStack
        {
            if inst.get_stack_offset() == prev_inst.get_stack_offset() {
                return true;
            }
        }
        false
    }
}
