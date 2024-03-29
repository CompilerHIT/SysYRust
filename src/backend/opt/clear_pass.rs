use std::collections::{HashMap, HashSet};

// use crate::{
//     backend::{block::FLOAT_BASE, regalloc, simulator::program_stat::ProgramStat},
//     log_file,
//     utility::ObjPool,
// };

use crate::{
    backend::{block::FLOAT_BASE, simulator::program_stat::ProgramStat},
    log_file,
};

use super::*;

impl BackendPass {
    pub fn clear_pass(&mut self, _pool: &BackendPool) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    self.rm_useless(*block);
                });
                func.as_mut().remove_unuse_inst();
                self.rm_repeated_sl(func.clone());
            }
        });
    }

    /// 该代码应该在后端常量传播和常量折叠(表达式归纳,编译时计算) 后完成
    /// 移除重复的load语句和store语句 (目前只针对loadstack和storestack)
    fn rm_repeated_sl(&self, func: ObjPtr<Func>) {
        // 删除重复的StoreStack和重复的LoadStack
        loop {
            let mut if_finish = true;
            for bb in func.blocks.iter() {
                // last_read[key]= if true=>上一条为读记录 elif false=>上一条为写记录 else 记录不存在
                let mut last_load: HashMap<Reg, IImm> = HashMap::new(); //记录这对寄存器在之前的栈空间对中发生了写操作
                let mut last_store: HashMap<Reg, IImm> = HashMap::new();
                let mut to_removed: HashSet<usize> = HashSet::new(); //记录将要移除的指令位置
                                                                     // 遍历指令
                for (index, inst) in bb.insts.iter().enumerate() {
                    let inst_type = inst.get_type();
                    if inst_type != InstrsType::LoadFromStack
                        && inst_type != InstrsType::StoreToStack
                    {
                        // 如果遇到一条关于reg的def而且不是读取的语句,则寻找到该寄存器的读所在地
                        for reg in self.get_reg_def_for_remove_repeated_load_store(*inst) {
                            last_load.remove(&reg);
                            last_store.remove(&reg);
                        }
                        continue;
                    }
                    let reg = inst.get_dst().drop_reg();
                    let stack_slot = inst.get_stack_offset();
                    if inst_type == InstrsType::LoadFromStack {
                        if !last_load.contains_key(&reg) {
                            last_load.insert(reg, stack_slot);
                            continue;
                        }
                        let last_load_stack_slot = last_load.get(&reg).unwrap();
                        if *last_load_stack_slot == stack_slot {
                            to_removed.insert(index);
                        }
                        last_load.insert(reg, stack_slot);
                    } else if inst_type == InstrsType::StoreToStack {
                        if !last_store.contains_key(&reg) {
                            last_store.insert(reg, stack_slot);
                            continue;
                        }
                        let last_store_stack_slot = last_store.get(&reg).unwrap();
                        if *last_store_stack_slot == stack_slot {
                            to_removed.insert(index);
                        }
                        last_store.insert(reg, stack_slot);
                    } else {
                        unreachable!();
                    }
                }
                // 对相应指令进行删除
                let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
                for (index, inst) in bb.insts.iter().enumerate() {
                    if to_removed.contains(&index) {
                        if_finish = false;
                        log_file!(
                            "remove_load_store.txt",
                            "{}={}-{}",
                            func.label,
                            bb.label,
                            index
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
        //删除重复的Store和Load
    }

    pub fn rm_useless(&self, block: ObjPtr<BB>) {
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

impl BackendPass {
    fn get_reg_def_for_remove_repeated_load_store(&self, inst: ObjPtr<LIRInst>) -> Vec<Reg> {
        if inst.get_type() != InstrsType::Call {
            //TODO, 根据func来指定使用的caller save寄存器

            return inst.get_reg_def();
        }
        let mut out = Vec::new();
        //加入所有caller save寄存器的值
        let mut iv = vec![1, 5, 6, 7];
        iv.extend(10..=17);
        iv.extend(28..=31);
        let mut fv = vec![];
        fv.extend(0..=7);
        fv.extend(10..=17);
        fv.extend(28..=31);
        for ireg in iv {
            out.push(Reg::new(ireg, ScalarType::Int));
        }
        for freg in fv {
            out.push(Reg::new(freg + FLOAT_BASE, ScalarType::Float));
        }

        return out;
    }
}
