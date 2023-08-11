use std::collections::{HashSet, LinkedList};

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

    pub fn fuse_tmp_regs(&mut self) {
        // 需要保证临时寄存器存在，对临时寄存器进行窥孔
        self.module.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|b| {
                let live_out = &b.live_out;
                let mut delete_pos: HashSet<usize> = HashSet::new();
                if b.insts.len() < 2 {
                    return;
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
                    let mut res = vec![];
                    log!("srcs: {:?}", srcs);
                    for src in srcs {
                        let mut start = src.clone();
                        match src {
                            Operand::Reg(_) => {
                                for i in 1..=index {
                                    let inst = b.insts[index - i];
                                    if inst.get_type() == InstrsType::Call {
                                        break;
                                    }
                                    if inst.get_dst().clone() == start.clone() {
                                        if live_out.contains(&inst.get_dst().drop_reg()) {
                                            break;
                                        }
                                        if inst.get_type() == InstrsType::OpReg(SingleOp::Mv) {
                                            start = inst.get_lhs().clone();
                                            delete_pos.insert(index - i);
                                        }
                                    }
                                }
                                res.push(start.clone());

                                // for i in 1..=index {
                                //     let inst = b.insts[index - i];
                                //     if inst.get_type() == InstrsType::Call {
                                //         break;
                                //     }
                                //     if inst.get_dst().clone() == start.clone() {
                                //         if live_out.contains(&inst.get_dst().drop_reg()) {
                                //             break;
                                //         }
                                //         if b.insts[index].get_type()
                                //             == InstrsType::OpReg(SingleOp::Mv)
                                //         {
                                //             b.insts[index].as_mut().replace_kind(inst.get_type());
                                //             res = inst.operands.split_first().unwrap().1.clone().to_vec();
                                //             delete_pos.insert(index - i);
                                //             break;
                                //         }
                                //     }
                                // }
                            }
                            _ => {
                                res.push(start);
                                continue;
                            }
                        };
                    }
                    res.insert(0, dst.clone());
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
