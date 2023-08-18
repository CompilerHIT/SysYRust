use std::collections::HashSet;

use crate::backend::regalloc::perfect_alloc;

use super::*;

impl BackendPass {
    pub fn opt_gep(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|block| {
                //获取块内所有load和store指令的位置，ls前有三个指令计算地址
                let ls_pos: Vec<usize> = block
                    .insts
                    .iter()
                    .filter(|inst| {
                        inst.get_type() == InstrsType::Load || inst.get_type() == InstrsType::Store
                    })
                    .map(|inst| block.insts.iter().position(|i| i == inst).unwrap())
                    .filter(|pos| *pos >= 3 && is_sl(*block, *pos))
                    .collect();

                // 将相同基地址的l和s合并为一组
                let mut ls_group_index: HashMap<Reg, Vec<usize>> = HashMap::new();
                // 过滤条件：基地址相同，计算偏移量为常数
                for pos in ls_pos.iter() {
                    let addr = match block.insts[pos - 3].get_lhs() {
                        Operand::Reg(reg) => reg,
                        _ => continue,
                    };
                    match block.insts[pos - 3].get_rhs() {
                        Operand::IImm(imm) => imm.get_data(),
                        _ => continue,
                    };
                    ls_group_index.entry(*addr).or_insert(Vec::new()).push(*pos);
                }

                // 对每一组进行优化
                let mut rm_pos: Vec<ObjPtr<LIRInst>> = Vec::new();
                for (_, poses) in ls_group_index.iter_mut() {
                    // 只计算每组中第一条指令的偏移量
                    let first_offset = match block.insts[poses[0] - 3].get_rhs() {
                        Operand::IImm(imm) => imm.get_data(),
                        _ => unreachable!("offset must be imm"),
                    };
                    let addr = block.insts[poses[0]].get_lhs();
                    poses.remove(0);
                    // 其他偏移由根据第一条指令的偏移计算
                    for pos in poses.iter() {
                        let inst = block.insts[*pos];
                        let offset = match block.insts[*pos - 3].get_rhs() {
                            Operand::IImm(imm) => imm.get_data(),
                            _ => unreachable!("offset must be imm"),
                        };
                        let new_offset = offset - first_offset;
                        inst.as_mut().replace_op(vec![
                            inst.get_dst().clone(),
                            addr.clone(),
                            Operand::IImm(IImm::new(new_offset * 4)),
                        ]);
                        let inst1 = block.insts[*pos - 1];
                        let inst2 = block.insts[*pos - 2];
                        let inst3 = block.insts[*pos - 3];
                        rm_pos.append(&mut vec![inst1, inst2, inst3]);
                    }
                }

                // 删除指令
                let new_insts = block
                    .insts
                    .iter()
                    .filter(|inst| !rm_pos.contains(inst))
                    .map(|x| *x)
                    .collect();
                block.as_mut().insts = new_insts;
            });
        })
    }

    pub fn particular_opt(&mut self) {
        self.rm_useless_shift();
        self.fuse_br_cond();
        let if_fuse = self.judge_spend();
        self.fuse_tmp_regs_up(if_fuse);
        // self.fuse_tmp_regs_down();
    }

    fn judge_spend(&mut self) -> HashMap<String, bool> {
        let mut res: HashMap<String, bool> = HashMap::new();
        self.module.name_func.iter().for_each(|(name, func)| {
            func.calc_live_for_alloc_reg();
            if let Some(_) = perfect_alloc::alloc(func) {
                res.insert(name.clone(), false);
            } else {
                res.insert(name.clone(), true);
            }
        });
        res
    }

    pub fn rm_useless_shift(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|b| {
                b.insts.iter().for_each(|inst| match inst.get_type() {
                    InstrsType::Binary(op) => match op {
                        BinaryOp::Sar | BinaryOp::Shl | BinaryOp::Shr => match inst.get_rhs() {
                            Operand::IImm(imm) => {
                                if imm.get_data() == 0 {
                                    inst.as_mut().replace_kind(InstrsType::OpReg(SingleOp::Mv));
                                    inst.as_mut().operands.pop();
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    },
                    _ => {}
                })
            })
        })
    }

    pub fn fuse_br_cond(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|b| {
                if b.insts.len() < 2 {
                    return;
                }
                let mut index = 0;
                loop {
                    if index == b.insts.len() - 1 {
                        break;
                    }
                    let inst1 = b.insts[index];
                    let inst2 = b.insts[index + 1];
                    if inst1.get_type() == InstrsType::OpReg(SingleOp::Li)
                        && *inst1.get_lhs() == Operand::IImm(IImm::new(0))
                    {
                        let operand = inst1.get_dst().clone();
                        match inst2.get_type() {
                            InstrsType::Branch(cond) => {
                                if inst2.get_reg_use().contains(&inst1.get_dst().drop_reg()) {
                                    match cond {
                                        CmpOp::Eq => {
                                            inst2
                                                .as_mut()
                                                .replace_kind(InstrsType::Branch(CmpOp::Eqz));
                                            let v: Vec<Operand> = inst2
                                                .operands
                                                .iter()
                                                .filter(|x| **x != operand)
                                                .map(|x| x.clone())
                                                .collect();
                                            inst2.as_mut().replace_op(v.clone());
                                        }
                                        CmpOp::Ne => {
                                            inst2
                                                .as_mut()
                                                .replace_kind(InstrsType::Branch(CmpOp::Nez));
                                            let v: Vec<Operand> = inst2
                                                .operands
                                                .iter()
                                                .filter(|x| **x != operand)
                                                .map(|x| x.clone())
                                                .collect();
                                            inst2.as_mut().replace_op(v.clone());
                                        }
                                        _ => {
                                            let pos = inst2
                                                .operands
                                                .iter()
                                                .position(|x| *x == operand)
                                                .unwrap();
                                            inst2.as_mut().operands[pos] =
                                                Operand::Reg(Reg::new(0, ScalarType::Int));
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    index += 1;
                }
            })
        })
    }

    pub fn fuse_tmp_regs_up(&mut self, if_fuse: HashMap<String, bool>) {
        // 需要保证临时寄存器存在，对临时寄存器进行窥孔
        self.module.name_func.iter().for_each(|(name, func)| {
            func.calc_live_base();
            if !*if_fuse.get(name).unwrap() {
                return;
            }
            func.blocks.iter().for_each(|b| {
                let live_out = &b.live_out;
                // log!("start");
                // log!("block: {}, live_out: {:?}", b.label, live_out);
                let mut delete_pos: HashSet<usize> = HashSet::new();
                if b.insts.len() < 2 {
                    return;
                }

                // 统计块内所有虚拟寄存器最后一次被def的位置
                let mut reg_def_pos: HashMap<Reg, usize> = HashMap::new();
                for (pos, inst) in b.insts.iter().enumerate() {
                    let reg_def = inst.get_reg_def();
                    if reg_def.len() > 0 {
                        reg_def_pos.insert(reg_def[0], pos);
                    }
                }

                let mut index = b.insts.len() - 1;
                loop {
                    if index == 0 {
                        break;
                    }
                    if b.insts[index].operands.len() < 2 {
                        index -= 1;
                        continue;
                    }
                    if b.insts[index].get_type() == InstrsType::Call {
                        index -= 1;
                        continue;
                    }
                    let (dst, srcs) = b.insts[index].operands.split_first().unwrap();
                    if delete_pos.contains(&index) {
                        index -= 1;
                        continue;
                    }
                    let mut is_store = false;
                    let mut src_regs = srcs.to_vec();
                    let mut res = vec![];
                    match b.insts[index].get_type() {
                        InstrsType::Store => {
                            is_store = true;
                            src_regs.insert(0, dst.clone());
                        }
                        _ => {}
                    }
                    for src in src_regs.iter() {
                        match src {
                            Operand::Reg(reg) => {
                                let mut start = *reg;
                                for i in 1..=index {
                                    let inst = b.insts[index - i];
                                    let reg_def = inst.get_reg_def();
                                    if reg_def.len() > 0 && reg_def[0] == start {
                                        let reg = reg_def[0];
                                        if live_out.contains(&reg) {
                                            break;
                                        }
                                        if inst.get_type() == InstrsType::OpReg(SingleOp::Mv) {
                                            if reg_def_pos.contains_key(&inst.get_lhs().drop_reg())
                                                && index - i
                                                    < *reg_def_pos
                                                        .get(&inst.get_lhs().drop_reg())
                                                        .unwrap()
                                            {
                                                break;
                                            }
                                            start = inst.get_lhs().drop_reg();
                                            delete_pos.insert(index - i);
                                        } else {
                                            break;
                                        }
                                    }
                                }
                                res.push(Operand::Reg(start));
                            }
                            _ => {
                                res.push(src.clone());
                                continue;
                            }
                        };
                    }
                    if !is_store {
                        res.insert(0, dst.clone());
                    }
                    debug_assert_eq!(
                        res.len(),
                        b.insts[index].operands.len(),
                        "res: {:?}, inst: {:?}",
                        res,
                        b.insts[index]
                    );
                    b.insts[index].as_mut().replace_op(res);
                    index -= 1;
                }
                log!("delete_pos: {:?}, insts: {:?}", delete_pos, b.insts);
                let new_insts: Vec<ObjPtr<LIRInst>> = b
                    .insts
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !delete_pos.contains(i))
                    .map(|(_, x)| *x)
                    .collect();
                b.as_mut().insts = new_insts;
            })
        })
    }

    pub fn fuse_tmp_regs_down(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|b| {
                let mut delete_pos: HashSet<usize> = HashSet::new();
                let live_out = &b.live_out;
                if b.insts.len() < 2 {
                    return;
                }
                let mut reg_last_use: HashMap<Reg, usize> = HashMap::new();
                // 获取reg最后一次使用的位置
                for (pos, inst) in b.insts.iter().enumerate() {
                    let reg_use = inst.get_reg_use();
                    for reg in reg_use.iter() {
                        reg_last_use.insert(*reg, pos);
                    }
                }
                let mut index = b.insts.len() - 1;
                loop {
                    if index == 0 {
                        break;
                    }
                    if b.insts[index].operands.len() < 2 {
                        index -= 1;
                        continue;
                    }
                    if b.insts[index].get_type() == InstrsType::Call {
                        index -= 1;
                        continue;
                    }
                    if delete_pos.contains(&index) {
                        index -= 1;
                        continue;
                    }
                    if b.insts[index].get_type() == InstrsType::OpReg(SingleOp::Mv) {
                        let src = b.insts[index].get_lhs();
                        let dst = b.insts[index].get_dst();
                        for i in 1..=index {
                            let inst = b.insts[index - i];
                            let reg_def = inst.get_reg_def();
                            if reg_def.len() > 0 && reg_def[0] == src.drop_reg() {
                                if inst.get_type() == InstrsType::Call {
                                    continue;
                                }
                                if live_out.contains(&reg_def[0]) {
                                    break;
                                }
                                if let Some(pos) = reg_last_use.get(&reg_def[0]) {
                                    if *pos == index {
                                        delete_pos.insert(index);
                                        match inst.operands[0] {
                                            Operand::Reg(..) => {}
                                            _ => unreachable!("{:?}", inst),
                                        }
                                        inst.as_mut().operands[0] = dst.clone();
                                    }
                                } else {
                                    break;
                                }
                            }
                        }
                    }
                    index -= 1;
                }
                log!("delete_pos: {:?}, insts: {:?}", delete_pos, b.insts);
                let new_insts: Vec<ObjPtr<LIRInst>> = b
                    .insts
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| !delete_pos.contains(i))
                    .map(|(_, x)| *x)
                    .collect();
                b.as_mut().insts = new_insts;
            })
        })
    }
}

fn is_sl(block: ObjPtr<BB>, pos: usize) -> bool {
    let inst1 = block.insts[pos - 1];
    let inst2 = block.insts[pos - 2];
    let inst3 = block.insts[pos - 3];
    if inst1.get_type() == InstrsType::Binary(BinaryOp::Add)
        && inst2.get_type() == InstrsType::Binary(BinaryOp::Shl)
        && inst3.get_type() == InstrsType::Binary(BinaryOp::Add)
    {
        return true;
    }
    false
}
