use super::*;

impl BackendPass {
    pub fn block_pass(&mut self, pool: &mut BackendPool) {
        self.fuse_imm_br(pool);
        self.fuse_basic_block(pool);
    }

    fn fuse_imm_br(&mut self, pool: &mut BackendPool) {
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut imm_br: Vec<ObjPtr<BB>> = vec![];
                func.blocks.iter().for_each(|block| {
                    let afters = block.get_after();
                    let prevs = block.get_prev();
                    if afters.len() == 1
                        && prevs.len() == 1
                        && prevs[0].insts.len() > 0 && prevs[0].get_tail_inst().get_type() == InstrsType::Jump
                    {
                        imm_br.push(block.clone());
                    }
                });

                imm_br.iter().for_each(|block| {
                    let prev = block.get_prev()[0];
                    let after = block.get_after()[0];
                    prev.as_mut().insts.pop();
                    prev.as_mut().push_back_list(&mut block.as_mut().insts);
                    let br = pool.put_inst(LIRInst::new(
                        InstrsType::Jump,
                        vec![Operand::Addr(after.label.clone())],
                    ));
                    prev.as_mut().push_back(br);
                    prev.as_mut().out_edge = vec![after.clone()];
                    after.as_mut().in_edge = vec![prev.clone()];
                })
            }
        });
    }

    fn fuse_basic_block(&mut self, pool: &mut BackendPool) {
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut useless_blocks: Vec<ObjPtr<BB>> = vec![];
                func.blocks.iter().for_each(|block| {
                    let prevs = block.get_prev();
                    if prevs.len() == 1 {
                        let prev = prevs[0];
                        if prev.insts.len() > 0 && prev.get_tail_inst().get_type() == InstrsType::Jump {
                            useless_blocks.push(block.clone());
                        }
                    }
                });
                
                useless_blocks.iter().for_each(|block| {
                    let prev = block.get_prev()[0];
                    prev.as_mut().insts.pop();
                    prev.as_mut().push_back_list(&mut block.as_mut().insts);
                    prev.as_mut().out_edge = block.get_after().clone();
                    block.get_after().iter().for_each(|after| {
                        after.as_mut().in_edge = vec![prev.clone()];
                    })
                })
            }
        })
    }
}
