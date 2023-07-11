use std::{
    collections::{HashMap, HashSet},
    fs::{self, File, OpenOptions},
};

use crate::{
    backend::{block::FLOAT_BASE, regalloc},
    log_file,
    utility::ObjPool,
};

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
                // let mut bf = OpenOptions::new()
                //     .create(true)
                //     .append(true)
                //     .open("before_rmls.txt")
                //     .unwrap();
                // let mut sf = OpenOptions::new()
                //     .create(true)
                //     .append(true)
                //     .open("after_rmls.txt")
                //     .unwrap();
                // func.as_mut().generate_row(func.context, &mut bf);
                // func.as_mut().generate_row(func.context, &mut sf);
            }
        });
    }
    ///移除重复的load语句和store语句 (目前只针对loadstack和storestack)
    fn rm_repeated_sl(&self, func: ObjPtr<Func>) {
        // 记录某个栈空间前面是进行了读还是写,是读到某个寄存器还是写入某个寄存器
        // 如果是一个寄存器对一个栈空间的重复读写的话,则可以省略
        for bb in func.blocks.iter() {
            // last_read[key]= if true=>上一条为读记录 elif false=>上一条为写记录 else 记录不存在
            let mut last_load: HashMap<Reg, IImm> = HashMap::new(); //记录这对寄存器在之前的栈空间对中发生了写操作
            let mut last_store: HashMap<Reg, IImm> = HashMap::new();
            let mut to_removed: HashSet<usize> = HashSet::new(); //记录将要移除的指令位置
                                                                 // 遍历指令
            for (index, inst) in bb.insts.iter().enumerate() {
                let inst_type = inst.get_type();
                if inst_type != InstrsType::LoadFromStack && inst_type != InstrsType::StoreToStack {
                    // 如果遇到一条关于reg的def而且不是读取的语句,则寻找到该寄存器的读所在地
                    for reg in self.get_reg_def_for_remove_repeated_load_store(*inst) {
                        last_load.remove(&reg);
                        last_store.remove(&reg);
                    }
                    continue;
                }
                let reg = inst.get_dst().drop_reg();
                let stack_slot = inst.get_stack_offset();
                let m = IImm::new(0);
                if stack_slot == m {
                    let b = 2;
                }
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
                    log_file!("rrsl.txt", "{}={}-{}", func.label, bb.label, index);
                    continue;
                }
                new_insts.push(*inst);
            }
            bb.as_mut().insts = new_insts;
        }
        // 删除
        // todo!()
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
                // 如果是call指令或者是ret指令则不能删除
                match inst.get_type() {
                    InstrsType::Call | InstrsType::Ret(_) => {
                        index += 1;
                        continue;
                    }
                    _ => (),
                };

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

impl BackendPass {
    fn get_reg_def_for_remove_repeated_load_store(&self, inst: ObjPtr<LIRInst>) -> Vec<Reg> {
        if inst.get_type() != InstrsType::Call {
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
