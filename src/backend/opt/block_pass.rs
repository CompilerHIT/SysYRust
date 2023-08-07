use std::collections::HashSet;

use super::*;

impl BackendPass {
    pub fn block_pass_pre_clear(&mut self, pool: &mut BackendPool) {
        // 当前块只有一条跳转指令，将该块删除，并修改其前驱的跳转目标、前驱的后继、后继的前驱
        self.clear_one_jump();
        // 如果一个块的终止指令是直接跳转, 且直接跳转到的基本块里有且只有一条直接跳转的指令, 那么就把这个二次跳转消除
        // 要求中间那个只有一条跳转指令的基本块的前继只有一个,
        self.fuse_imm_br(pool);
        // 在直接跳转到另一个块, 并且跳转目标块没有其它前继的情况下, 可以直接把两个块合成为一个大块
        self.fuse_basic_block();
    }
    pub fn block_pass(&mut self) {
        // 若branch的下一条jump指令的目标块，只有一个前驱，则将该jump指令删除，并将其合并到这个块中
        self.merge_br_jump();
        // 如果branch和其紧邻的jump语句的目标块相同，则将jump语句删除
        self.resolve_merge_br();
        // 处理fuse_imm_br中，中间的基本块有多个前继的情况
        self.fuse_muti2imm_br();
        // 对于指令数量较少的那些块，复制上提
        // self.copy_exec();
        // 删除0出入度的块
        // self.clear_unreachable_block();
        // 清除空块(包括entry块)
        self.clear_empty_block();
        // jump的目标块如果紧邻，则删除jump语句
        self.clear_useless_jump();
    }

    fn merge_br_jump(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut jumps: Vec<ObjPtr<BB>> = vec![];
                func.blocks.iter().for_each(|block| {
                    if block.get_prev().len() == 1 && is_br(block.get_prev()[0]) {
                        let prev_tail = block.get_prev()[0].get_tail_inst();
                        let jump_label = prev_tail.get_label();
                        if *jump_label == Operand::Addr(block.label.clone()) {
                            jumps.push(block.clone());
                        }
                    }
                });
                jumps.iter().for_each(|block| {
                    let prev = block.get_prev()[0];
                    prev.as_mut().insts.pop();
                    prev.as_mut().push_back_list(&mut block.as_mut().insts);
                    block.as_mut().insts.clear();
                    adjust_prev_out(prev.clone(), block.get_after().clone(), &block.label);
                    for after in block.get_after() {
                        adjust_after_in(after.clone(), block.get_prev().clone(), &block.label);
                    }
                })
            }
        })
    }

    fn fuse_imm_br(&mut self, pool: &mut BackendPool) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut imm_br: Vec<ObjPtr<BB>> = vec![];
                func.blocks.iter().for_each(|block| {
                    print_context(block.clone());
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
                    adjust_after_in(after, block.get_prev().clone(), &block.label);
                    block.as_mut().in_edge.clear();
                    block.as_mut().out_edge.clear();
                })
            }
        });
    }

    fn fuse_muti2imm_br(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            let mut imm_br_pred: Vec<(ObjPtr<BB>, HashSet<ObjPtr<BB>>)> = vec![];
            func.blocks.iter().for_each(|block| {
                let afters = block.get_after();
                // 获取那些通过jump跳到该块的前继
                let prevs: HashSet<_> = block
                    .get_prev()
                    .iter()
                    .filter(|&&prev| {
                        prev.insts.len() > 0
                            && prev.get_tail_inst().get_type() == InstrsType::Jump
                            && (prev.get_tail_inst().get_label().clone()
                                == Operand::Addr(block.label.clone()))
                    })
                    .map(|prev| *prev)
                    .collect();

                // 如果只有一个后继且满足上述条件的前继块数量大于0
                if afters.len() == 1 && prevs.len() > 0 {
                    imm_br_pred.push((block.clone(), prevs.clone()));
                }
            });

            imm_br_pred.iter().for_each(|(block, prevs)| {
                let prevs = prevs.iter().map(|x| *x).collect::<Vec<_>>();
                let after = block.get_after()[0];
                if prevs.len() == block.get_prev().len()
                    && !exist_br_label(prevs.clone(), &block.label)
                {
                    print_context(block.clone());
                    adjust_after_in(after, prevs.clone(), &block.label);
                    block.as_mut().out_edge.clear();
                } else {
                    adjust_after_in(after, prevs.clone(), &String::from(""));
                }
                for prev in prevs.iter() {
                    let mut insts = block.insts.clone();
                    prev.as_mut().insts.pop();
                    prev.as_mut().push_back_list(&mut insts);
                    adjust_prev_out(prev.clone(), vec![after.clone()], &block.label);
                }
                block.as_mut().in_edge = block
                    .as_mut()
                    .in_edge
                    .iter()
                    .filter(|&&b| prevs.iter().all(|&prev| prev != b))
                    .map(|b| *b)
                    .collect::<Vec<ObjPtr<BB>>>();
            })
        })
    }

    // fn copy_exec(&mut self) {
    //     self.module.name_func.iter().for_each(|(_, func)| {
    //         let mut imm_br_pred: Vec<(ObjPtr<BB>, HashSet<ObjPtr<BB>>)> = vec![];
    //         func.blocks.iter().for_each(|block| {
    //             // 获取那些通过jump跳到该块的前继
    //             let prevs: HashSet<_> = block
    //                 .get_prev()
    //                 .iter()
    //                 .filter(|&&prev| {
    //                     prev.insts.len() > 0
    //                         && prev.get_tail_inst().get_type() == InstrsType::Jump
    //                         && (prev.get_tail_inst().get_label().clone()
    //                             == Operand::Addr(block.label.clone()))
    //                 })
    //                 .map(|prev| *prev)
    //                 .collect();

    //             // 如果只有一个后继且满足上述条件的前继块数量大于0
    //             if block.insts.len() <= 5 && prevs.len() > 0 {
    //                 imm_br_pred.push((block.clone(), prevs.clone()));
    //             }
    //         });

    //         imm_br_pred.iter().for_each(|(block, prevs)| {
    //             let prevs = prevs.iter().map(|x| *x).collect::<Vec<_>>();
    //             // 前继不经由branch跳到该块
    //             if prevs.len() == block.get_prev().len() && !exist_br_label(prevs.clone(), &block.label) {
    //                 for after in block.get_after().iter() {
    //                     adjust_after_in(after.clone(), prevs.clone(), &block.label);
    //                 }
    //                 block.as_mut().out_edge.clear();
    //             } else {
    //                 for after in block.get_after().iter() {
    //                     adjust_after_in(after.clone(), prevs.clone(), &String::from(""));
    //                 }
    //             }
    //             for prev in prevs.iter() {
    //                 let mut insts = block.insts.clone();
    //                 prev.as_mut().insts.pop();
    //                 prev.as_mut().push_back_list(&mut insts);
    //                 adjust_prev_out(prev.clone(), block.get_after().clone(), &block.label);
    //             }
    //             block.as_mut().in_edge = block
    //                 .as_mut()
    //                 .in_edge
    //                 .iter()
    //                 .filter(|&&b| prevs.iter().all(|&prev| prev != b))
    //                 .map(|b| *b)
    //                 .collect::<Vec<ObjPtr<BB>>>();
    //         })
    //     })
    // }

    fn fuse_basic_block(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut useless_blocks: Vec<ObjPtr<BB>> = vec![];
                func.blocks.iter().for_each(|block| {
                    let prevs = block.get_prev();
                    if prevs.len() == 1 {
                        let prev = prevs[0];
                        if is_jump(prev) && prev.get_after().len() == 1 {
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
                        adjust_after_in(after.clone(), block.get_prev().clone(), &block.label);
                    });
                    block.as_mut().in_edge.clear();
                    block.as_mut().out_edge.clear();
                })
            }
        })
    }

    fn clear_one_jump(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut clear_blocks = HashSet::new();
                func.blocks.iter().for_each(|block| {
                    if block.insts.len() == 1 {
                        let tail = block.get_tail_inst();
                        if tail.get_type() == InstrsType::Jump {
                            block.as_mut().insts.clear();
                            let next = block.get_after()[0].clone();
                            let after_label = next.label.clone();

                            //调整后继的前驱
                            adjust_after_in(next, block.get_prev().clone(), &block.label);

                            let prevs = block.get_prev();
                            prevs.iter().for_each(|prev| {
                                if prev.insts.len() == 0 {
                                    replace_first_block(block.clone(), func.clone());
                                } else {
                                    let prev_tail = prev.get_tail_inst();
                                    if *prev_tail.get_label() == Operand::Addr(block.label.clone())
                                    {
                                        prev_tail.as_mut().replace_label(after_label.clone());
                                    } else {
                                        if prev.insts.len() > 1 {
                                            let last_two_tail = prev.get_last_not_tail_inst();
                                            last_two_tail
                                                .as_mut()
                                                .replace_label(after_label.clone());
                                        }
                                    }
                                    adjust_prev_out(
                                        prev.clone(),
                                        block.get_after().clone(),
                                        &block.label,
                                    );
                                    clear_blocks.insert(block);
                                }
                            });
                        }
                    }
                });
                for b in clear_blocks.iter() {
                    b.as_mut().in_edge.clear();
                    b.as_mut().out_edge.clear();
                }
            }
        })
    }

    pub fn clear_empty_block(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                let mut exsit_blocks: Vec<ObjPtr<BB>> = vec![];
                func.blocks.iter().for_each(|block| {
                    if block.insts.len() > 0 {
                        exsit_blocks.push(block.clone());
                    } else {
                        block.as_mut().in_edge.clear();
                        block.as_mut().out_edge.clear();
                    }
                });
                func.as_mut().blocks = exsit_blocks.clone();
            }
        })
    }

    pub fn clear_unreachable_block(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            let mut exist_blocks: Vec<ObjPtr<BB>> = vec![];
            func.blocks.iter().for_each(|block| {
                if block.get_after().len() != 0 || block.get_prev().len() != 0 {
                    exist_blocks.push(block.clone());
                }
            });
            func.as_mut().blocks = exist_blocks.clone();
        })
    }

    fn resolve_merge_br(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    if block.insts.len() > 1 {
                        let last_not_tail = block.get_last_not_tail_inst();
                        let tail = block.get_tail_inst();
                        match last_not_tail.get_type() {
                            InstrsType::Branch(..) => {
                                if tail.get_type() == InstrsType::Jump
                                    && tail.get_label() == last_not_tail.get_label()
                                {
                                    block.as_mut().insts.pop();
                                    block.as_mut().insts.pop();
                                    block.as_mut().push_back(tail);
                                }
                            }
                            _ => {}
                        }
                    }
                })
            }
        });
    }

    pub fn clear_useless_jump(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            if !func.is_extern {
                for (i, block) in func.blocks.iter().enumerate() {
                    if block.insts.len() > 0 && i < func.blocks.len() - 1 {
                        let tail = block.get_tail_inst();
                        if tail.get_type() == InstrsType::Jump {
                            let label = match tail.get_label() {
                                Operand::Addr(label) => label,
                                _ => panic!("jump label error"),
                            };
                            if *label == func.blocks[i + 1].label {
                                // let labels: Vec<_> =
                                //     block.get_after().iter().map(|b| b.label.clone()).collect();
                                // log!("jump label: {:?}, next blocks label: {:?}", tail.get_label(), labels);
                                block.as_mut().insts.pop();
                            }
                        }
                    }
                }
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

fn is_br(block: ObjPtr<BB>) -> bool {
    if block.insts.len() > 1 {
        match block.get_last_not_tail_inst().get_type() {
            InstrsType::Branch(..) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
    false
}

fn adjust_after_in(block: ObjPtr<BB>, prevs: Vec<ObjPtr<BB>>, clear_label: &String) {
    let mut final_prevs: Vec<ObjPtr<BB>> = block
        .as_mut()
        .in_edge
        .clone()
        .into_iter()
        .filter(|b| b.label != *clear_label)
        .collect();
    final_prevs.append(&mut prevs.clone());
    block.as_mut().in_edge = final_prevs;
}

fn adjust_prev_out(block: ObjPtr<BB>, afters: Vec<ObjPtr<BB>>, clear_label: &String) {
    let mut final_prevs: Vec<ObjPtr<BB>> = block
        .as_mut()
        .out_edge
        .clone()
        .into_iter()
        .filter(|b| b.label != *clear_label)
        .collect();
    final_prevs.append(&mut afters.clone());
    block.as_mut().out_edge = final_prevs;
}

fn replace_first_block(block: ObjPtr<BB>, func: ObjPtr<Func>) {
    assert!(block.get_after().len() == 1);
    let after = block.get_after()[0].clone();
    let aafter = after.get_after();
    let mut index = 0;
    loop {
        if index >= func.blocks.len() {
            unreachable!("can't find after");
        }
        if func.blocks[index].label == after.label {
            break;
        }
        index += 1;
    }
    func.as_mut().blocks.remove(index);
    func.as_mut().blocks.remove(1);
    func.as_mut().blocks.insert(1, after);
    func.blocks[0].as_mut().out_edge = vec![after];
}

fn exist_br_label(blocks: Vec<ObjPtr<BB>>, label: &String) -> bool {
    for b in blocks.iter() {
        for inst in b.insts.iter() {
            match inst.get_type() {
                InstrsType::Branch(..) => {
                    if inst.get_label().clone() == Operand::Addr(label.clone()) {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}

fn print_context(block: ObjPtr<BB>) {
    log!("from: {}", block.label);
    for prev in block.get_prev().iter() {
        log!("prev: {}", prev.label);
    }
    for after in block.get_after().iter() {
        log!("after: {}", after.label);
    }
}
