use super::*;

impl BackendPass {
    pub fn block_pass(&mut self, pool: &mut BackendPool) {
        self.clear_one_jump();
        self.fuse_imm_br(pool);
        self.fuse_basic_block(pool);
        self.clear_empty_block();
    }

    fn fuse_imm_br(&mut self, pool: &mut BackendPool) {
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut imm_br: Vec<ObjPtr<BB>> = vec![];
                func.blocks.iter().for_each(|block| {
                    let afters = block.get_after();
                    let prevs = block.get_prev();
                    if afters.len() == 1 && prevs.len() == 1 && is_jump(prevs[0]) {
                        imm_br.push(block.clone());
                    }
                });

                imm_br.iter().for_each(|block| {
                    let prev = block.get_prev()[0];
                    let after = block.get_after()[0];
                    prev.as_mut().insts.pop();
                    block.as_mut().insts.pop();
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
                        if is_jump(prev) {
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

    fn clear_one_jump(&mut self) {
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    if block.insts.len() == 1 {
                        let tail = block.get_tail_inst();
                        if tail.get_type() == InstrsType::Jump {
                            block.as_mut().insts.clear();
                            let after_label = block.get_after()[0].label.clone();
                            let mut final_prevs: Vec<ObjPtr<BB>> = block.get_after()[0]
                                .as_mut()
                                .in_edge
                                .clone()
                                .into_iter()
                                .filter(|b| b.label != block.label.clone())
                                .collect();
                            final_prevs.append(&mut block.get_prev().clone());
                            block.get_after()[0].as_mut().in_edge = final_prevs;
                            let prevs = block.get_prev();
                            prevs.iter().for_each(|prev| {
                                let last_two_tail = prev.get_last_not_tail_inst();
                                let prev_tail = prev.get_tail_inst();
                                if *prev_tail.get_label() == Operand::Addr(block.label.clone()) {
                                    prev_tail.as_mut().replace_label(after_label.clone());
                                } else {
                                    last_two_tail.as_mut().replace_label(after_label.clone());
                                }
                                prev.as_mut().out_edge = vec![block.get_after()[0].clone()];
                            })
                        }
                    }
                })
            }
        })
    }

    fn clear_empty_block(&mut self) {
        let mut exsit_blocks: Vec<ObjPtr<BB>> = vec![];
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    if block.insts.len() > 0 {
                        exsit_blocks.push(block.clone());
                    }
                });
                func.as_mut().blocks = exsit_blocks.clone();
                exsit_blocks.clear();
            }
        })
    }
}

fn is_jump(block: ObjPtr<BB>) -> bool {
    if block.insts.len() > 0 {
        if block.get_tail_inst().get_type() == InstrsType::Jump {
            if block.insts.len() > 1 {
                let last_tow_inst = block.get_last_not_tail_inst();
                match last_tow_inst.get_type() {
                    InstrsType::Branch(..) => {
                        return false;
                    }
                    _ => {
                        return true;
                    }
                }
            } else {
                return true;
            }
        }
    }
    false
}
