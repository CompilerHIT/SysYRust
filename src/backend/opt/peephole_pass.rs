// use crate::log;

use super::*;
impl BackendPass {
    pub fn peephole_pass(&mut self, pool: &mut BackendPool) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    // 经过两次fuse_imm的块合并后会产生mv tmp, src; mv dst, tmp;的可消去的无用phi指令导致的mv
                    // 匹配模式为：两次相邻的mv指令如果满足上述模式则进行融合。
                    self.rm_useless_mv();
                    // 在处理handle_overflow前的优化
                    self.rm_useless_overflow(*block, pool);
                    // self.rm_useless_param_overflow(*func, *block, pool);
                    // self.rm_same_store(*block, pool);
                })
            }
        });
    }

    fn rm_useless_mv(&self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|block| {
                block.as_mut().build_reg_intervals();
                let mut index = 0;
                loop {
                    if block.insts.len() < 2 || index > block.insts.len() - 2 {
                        break;
                    }

                    let inst1 = block.insts[index];
                    let inst2 = block.insts[index + 1];

                    if inst1.get_type() == InstrsType::OpReg(SingleOp::Mv)
                        && inst2.get_type() == InstrsType::OpReg(SingleOp::Mv)
                    {
                        if inst1.get_dst().clone() == inst2.get_lhs().clone() {
                            let inst2_reg = match inst2.get_lhs().clone() {
                                Operand::Reg(reg) => reg,
                                _ => unreachable!("must be reg"),
                            };
                            let info = block
                                .reg_intervals
                                .iter()
                                .find(|((reg, _), _)| reg.get_id() == inst2_reg.get_id())
                                .unwrap();
                            if index + 1 == info.1.1 as usize {
                                inst2
                                    .as_mut()
                                    .replace_op(vec![inst2.get_dst().clone(), inst1.get_lhs().clone()]);
                                block.as_mut().insts.remove(index);
                            }
                        }
                    }
                    index += 1;
                }
            })
        })
    }

    fn rm_useless_overflow(&self, block: ObjPtr<BB>, pool: &mut BackendPool) {
        // 将紧挨着的几条会发生地址溢出的指令，偏移量加载语句合并为一条
        // 一组指令的第一条加载间接寻址地址，后面的offset相较其偏移不超过2047则纳入同一组
        // 由于Load/Store ParamFromStack 的逻辑为相对栈顶的偏移，因此不会被纳入同一组中
        let mut index = 0;
        loop {
            if index >= block.insts.len() {
                break;
            }
            let inst = block.insts[index];
            if self.is_overflow(inst) {
                let of = inst.get_stack_offset();
                let offset = of.get_data();
                let mut insts = vec![inst];
                let mut index2 = index + 1;
                loop {
                    if index2 >= block.insts.len() {
                        break;
                    }
                    let inst2 = block.insts[index2];
                    // 处理load/store to stack
                    if self.is_overflow(inst2) {
                        if operand::is_imm_12bs(inst2.get_stack_offset().get_data() - offset) {
                            insts.push(inst2);
                            index2 += 1;
                            continue;
                        }
                    }
                    break;
                }
                if insts.len() > 1 {
                    // l/s offset(sp) -> li offset gp. add gp gp sp. l/s 0(gp).
                    let s0 = Operand::Reg(Reg::new(8, ScalarType::Int));
                    block.as_mut().insts.insert(
                        index,
                        pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Li),
                            vec![s0.clone(), Operand::IImm(of)],
                        )),
                    );
                    index += 1;
                    let mut add = LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![
                            s0.clone(),
                            s0.clone(),
                            Operand::Reg(Reg::new(2, ScalarType::Int)),
                        ],
                    );
                    add.set_double();
                    block.as_mut().insts.insert(index, pool.put_inst(add));
                    index += 1;
                    // 对组内指令进行替换
                    for ls in insts.iter() {
                        let ls_offset = ls.get_stack_offset().get_data() - offset;
                        assert!(operand::is_imm_12bs(ls_offset));
                        let kind = match ls.get_type() {
                            InstrsType::LoadFromStack => InstrsType::Load,
                            InstrsType::StoreToStack => InstrsType::Store,
                            _ => panic!("get {:?}", inst.get_type()),
                        };
                        ls.as_mut().replace_kind(kind);
                        ls.as_mut().replace_op(vec![
                            ls.get_dst().clone(),
                            s0.clone(),
                            Operand::IImm(IImm::new(ls_offset)),
                        ]);
                    }
                    let len = insts.len();
                    // 替换原指令
                    block
                        .as_mut()
                        .insts
                        .splice(index..index + insts.len(), insts.into_iter());
                    index += len;
                    continue;
                }
            }
            index += 1;
        }
    }

    // fn rm_useless_param_overflow(
    //     &self,
    //     func: ObjPtr<Func>,
    //     block: ObjPtr<BB>,
    //     pool: &mut BackendPool,
    // ) {
    //     // 处理l/s param to stack
    //     let mut index = 0;
    //     loop {
    //         if index >= block.insts.len() {
    //             break;
    //         }
    //         let inst = block.insts[index];
    //         let stack_size = func.context.get_offset();
    //         if self.is_param_overflow(inst, stack_size) {
    //             let offset = inst.get_stack_offset().get_data();
    //             let mut insts = vec![inst];
    //             let mut index2 = index + 1;
    //             loop {
    //                 if index2 >= block.insts.len() {
    //                     break;
    //                 }
    //                 let inst2 = block.insts[index2];
    //                 // 处理load/store to stack
    //                 if self.is_param_overflow(inst2, stack_size) {
    //                     if operand::is_imm_12bs(inst2.get_stack_offset().get_data() - offset) {
    //                         insts.push(inst2);
    //                         index2 += 1;
    //                         continue;
    //                     }
    //                 }
    //                 break;
    //             }
    //             if insts.len() > 1 {
    //                 // l/s offset(sp) -> li offset gp. add gp gp sp. l/s 0(gp).
    //                 let s0 = Operand::Reg(Reg::new(8, ScalarType::Int));
    //                 let true_offset = func.context.get_offset() - offset;
    //                 block.as_mut().insts.insert(
    //                     index,
    //                     pool.put_inst(LIRInst::new(
    //                         InstrsType::OpReg(SingleOp::Li),
    //                         vec![s0.clone(), Operand::IImm(IImm::new(true_offset))],
    //                     )),
    //                 );
    //                 index += 1;
    //                 let mut add = LIRInst::new(
    //                     InstrsType::Binary(BinaryOp::Add),
    //                     vec![
    //                         s0.clone(),
    //                         s0.clone(),
    //                         Operand::Reg(Reg::new(2, ScalarType::Int)),
    //                     ],
    //                 );
    //                 add.set_double();
    //                 block.as_mut().insts.insert(index, pool.put_inst(add));
    //                 index += 1;
    //                 // 对组内指令进行替换
    //                 for ls in insts.iter() {
    //                     let ls_offset = ls.get_stack_offset().get_data() - offset;
    //                     let kind = match ls.get_type() {
    //                         InstrsType::LoadParamFromStack => InstrsType::Load,
    //                         InstrsType::StoreParamToStack => InstrsType::Store,
    //                         _ => panic!("get {:?}", inst.get_type()),
    //                     };
    //                     assert!(operand::is_imm_12bs(ls_offset));
    //                     ls.as_mut().replace_kind(kind);
    //                     ls.as_mut().replace_op(vec![
    //                         ls.get_dst().clone(),
    //                         s0.clone(),
    //                         Operand::IImm(IImm::new(ls_offset)),
    //                     ]);
    //                 }
    //                 let len = insts.len();
    //                 // 替换原指令
    //                 block
    //                     .as_mut()
    //                     .insts
    //                     .splice(index..index + insts.len(), insts.into_iter());
    //                 index += len;
    //                 continue;
    //             }
    //         }
    //         index += 1;
    //     }
    // }

    fn is_overflow(&self, inst: ObjPtr<LIRInst>) -> bool {
        if inst.get_type() == InstrsType::LoadFromStack
            || inst.get_type() == InstrsType::StoreToStack
        {
            if !operand::is_imm_12bs(inst.get_stack_offset().get_data()) {
                return true;
            }
        }
        false
    }

    // fn is_param_overflow(&self, inst: ObjPtr<LIRInst>, stack_size: i32) -> bool {
    //     if inst.get_type() == InstrsType::LoadParamFromStack
    //         || inst.get_type() == InstrsType::StoreParamToStack
    //     {
    //         if !operand::is_imm_12bs(stack_size - inst.get_stack_offset().get_data()) {
    //             return true;
    //         }
    //     }
    //     false
    // }

    // fn rm_same_store(&self, block: ObjPtr<BB>, pool: &mut BackendPool) {
    //     let stores = block
    //         .insts
    //         .iter()
    //         .filter(|inst| inst.get_type() == InstrsType::Store)
    //         .collect::<Vec<_>>();

    //     let same_stores = stores
    //         .iter()
    //         .filter(|inst| {
    //             stores.iter().any(|inst2| {
    //                 inst2.get_dst() == inst.get_dst()
    //                     && inst2.get_lhs() == inst.get_lhs()
    //                     && inst2.get_rhs() == inst.get_rhs()
    //             })
    //         })
    //         .collect::<Vec<_>>();
    //     let dst = match same_stores[0].get_dst().clone() {
    //         Operand::Reg(reg) => reg,
    //         _ => panic!("get {:?}", same_stores[0].get_dst()),
    //     };
    //     let src = match same_stores[0].get_lhs().clone() {
    //         Operand::Reg(reg) => reg,
    //         _ => panic!("get {:?}", same_stores[0].get_lhs()),
    //     };

    //     let st = block
    //         .insts
    //         .iter()
    //         .position(|inst| inst == *same_stores[0])
    //         .unwrap();
    //     let ed = block
    //         .insts
    //         .iter()
    //         .position(|inst| inst == *same_stores[same_stores.len() - 1])
    //         .unwrap();

    //     block.as_mut().build_reg_intervals();
    //     let dst_info = block
    //         .reg_intervals
    //         .iter()
    //         .find(|info| info.0 .0 == dst)
    //         .unwrap();
    //     let src_info = block
    //         .reg_intervals
    //         .iter()
    //         .find(|info| info.0 .0 == src)
    //         .unwrap();

    //     let (src_st, src_ed) = (src_info.0 .1 as usize, src_info.1 .1 as usize);
    //     let (dst_st, dst_ed) = (dst_info.0 .1 as usize, dst_info.1 .1 as usize);

    //     if st >= src_st && ed <= src_ed && st >= dst_st && ed <= dst_ed {
    //         // 说明这一段store是多余的
    //         block.as_mut().insts.drain((st + 1) as usize..=ed as usize);
    //     }
    // }
}
