use super::*;
impl BackendPass {
    pub fn peephole_pass(&mut self, pool: &mut BackendPool) {
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    // 在处理handle_overflow前的优化
                    self.rm_useless_overflow(*block, pool);
                })
            }
        });
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
                    let gp = Operand::Reg(Reg::new(3, ScalarType::Int));
                    block.as_mut().insts.insert(
                        index,
                        pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Li),
                            vec![gp.clone(), Operand::IImm(of)],
                        )),
                    );
                    index += 1;
                    block.as_mut().insts.insert(
                        index,
                        pool.put_inst(LIRInst::new(InstrsType::Binary(BinaryOp::Add), vec![gp.clone(), gp.clone(), ])),
                    );
                    new_inst
                        .as_mut()
                        .replace_op(vec![insts[0].get_dst().clone(), insts[0].get_lhs().clone()]);
                    new_inst.as_mut().replace_stack_offset(offset);
                    block
                        .as_mut()
                        .insts
                        .splice(index..index + insts.len(), vec![new_inst].into_iter());
                    index += 1;
                    continue;
                }
            }
            index += 1;
        }
    }

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
}
