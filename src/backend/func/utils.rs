use super::*;

/// 从函数中提取信息
impl Func {
    ///在handle spill之后调用
    /// 返回该函数使用了哪些callee saved的寄存器
    pub fn draw_used_callees(&self) -> HashSet<Reg> {
        let mut callees: HashSet<Reg> = HashSet::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_callee_save() {
                        callees.insert(reg);
                    }
                }
            }
        }
        callees
    }

    /// 该函数应该在vtop之后调用
    /// 获取该函数使用到的caller save寄存器
    pub fn draw_used_callers(&self) -> HashSet<Reg> {
        let mut callers: HashSet<Reg> = HashSet::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_caller_save() {
                        callers.insert(reg);
                    }
                }
            }
        }
        callers
    }

    // 实现一些关于函数信息的估计和获取的方法
    pub fn draw_phisic_regs(&self) -> RegUsedStat {
        let mut used = RegUsedStat::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_physic() {
                        used.use_reg(reg.get_color());
                    }
                }
            }
        }
        used
    }

    // 估计寄存器数量
    pub fn estimate_num_regs(&self) -> usize {
        let mut out = 0;
        self.blocks.iter().for_each(|bb| out += bb.insts.len());
        return out;
    }
    // 获取指令数量
    pub fn num_insts(&self) -> usize {
        let mut out = 0;
        self.blocks.iter().for_each(|bb| out += bb.insts.len());
        return out;
    }

    // 获取寄存器数量
    pub fn num_regs(&self) -> usize {
        let mut passed: Bitmap = Bitmap::with_cap(1000);
        let mut out = 0;
        self.blocks.iter().for_each(|bb| {
            bb.insts.iter().for_each(|inst| {
                for reg in inst.get_reg_def() {
                    let id = reg.get_id() << 1
                        | match reg.get_type() {
                            ScalarType::Float => 0,
                            ScalarType::Int => 1,
                            _ => panic!("unleagal"),
                        };
                    if passed.contains(id as usize) {
                        continue;
                    }
                    passed.insert(id as usize);
                    out += 1;
                }
            })
        });
        return out;
    }

    // 获取所有虚拟寄存器和用到的物理寄存器
    pub fn draw_all_virtual_regs(&self) -> HashSet<Reg> {
        let mut passed = HashSet::new();
        self.blocks.iter().for_each(|bb| {
            bb.insts.iter().for_each(|inst| {
                for reg in inst.get_regs() {
                    if reg.is_physic() {
                        continue;
                    }
                    passed.insert(reg);
                }
            })
        });
        passed
    }

    // 获取所有虚拟寄存器和用到的物理寄存器
    pub fn draw_all_regs(&self) -> HashSet<Reg> {
        let mut passed = HashSet::new();
        self.blocks.iter().for_each(|bb| {
            bb.insts.iter().for_each(|inst| {
                for reg in inst.get_regs() {
                    passed.insert(reg);
                }
            })
        });
        passed
    }
}

// 为每个指令创建寄存器的in和 out
impl Func {
    /// 依赖外部调用的calc live,为了加快速度,使用哈希表
    pub fn build_live_out_for_insts(&self) -> HashMap<ObjPtr<LIRInst>, Bitmap> {
        let mut live_out_for_insts: HashMap<ObjPtr<LIRInst>, Bitmap> = HashMap::new();
        self.blocks.iter().for_each(|bb| {
            Func::analyse_inst_with_live_now_bitmap_backorder(*bb, &mut |inst, live_now| {
                live_out_for_insts.insert(inst, live_now.clone());
            })
        });
        live_out_for_insts
    }
}

///为函数创建寄存器活跃区间
impl Func {
    /// 为函数创建寄存器活跃区间
    /// 在使用它之前需要现在外部调用某种calc live
    /// 内部不会调用 任何calc live (依赖于外部计算出来的 live in live out live use live def)
    /// 表面是unmut self,但是会通过内部可变性修改内部的 blocks的属性
    pub fn build_reg_intervals(&self) {
        for bb in self.blocks.iter() {
            bb.as_mut().build_reg_intervals();
        }
    }
}

//找到函数的最后一个块
impl Func {
    pub fn get_final_bb(&self) -> ObjPtr<BB> {
        let mut rets: Vec<ObjPtr<BB>> = Vec::new();
        for bb in self.blocks.iter() {
            if bb.insts.len() <= 0 {
                continue;
            }
            let last_inst = bb.insts.last().unwrap();
            match last_inst.get_type() {
                InstrsType::Ret(_) => {
                    rets.push(*bb);
                }
                _ => (),
            }
        }
        debug_assert!(rets.len() == 1);
        *rets.get(0).unwrap()
    }
}

///带live now分析block的inst
impl Func {
    pub fn analyse_inst_with_live_now_backorder(
        bb: ObjPtr<BB>,
        analyser: &mut dyn FnMut(ObjPtr<LIRInst>, &HashSet<Reg>),
    ) {
        let mut live_now = HashSet::new();
        bb.live_out.iter().for_each(|reg| {
            live_now.insert(*reg);
        });
        for inst in bb.insts.iter().rev() {
            analyser(*inst, &live_now);
            for reg in inst.get_reg_def() {
                live_now.remove(&reg);
            }
            for reg in inst.get_reg_use() {
                live_now.insert(reg);
            }
        }
    }

    pub fn analyse_inst_with_live_now_bitmap_backorder(
        bb: ObjPtr<BB>,
        analyser: &mut dyn FnMut(ObjPtr<LIRInst>, &Bitmap),
    ) {
        let mut live_now = Bitmap::new();
        bb.live_out.iter().for_each(|reg| {
            live_now.insert(reg.bit_code() as usize);
        });
        for inst in bb.insts.iter().rev() {
            analyser(*inst, &live_now);
            for reg in inst.get_reg_def() {
                live_now.remove(reg.bit_code() as usize);
            }
            for reg in inst.get_reg_use() {
                live_now.insert(reg.bit_code() as usize);
            }
        }
    }

    //反序分析指令直到
    pub fn analyse_inst_with_regused_and_index_backorder_until(
        bb: &BB,
        analyser: &mut dyn FnMut(ObjPtr<LIRInst>, usize, &RegUsedStat),
        until: &dyn Fn(ObjPtr<LIRInst>) -> bool,
    ) {
        let mut reg_use_stat = RegUsedStat::init_unspecial_regs();
        bb.live_out
            .iter()
            .for_each(|reg| reg_use_stat.use_reg(reg.get_color()));
        for (index, inst) in bb.insts.iter().enumerate().rev() {
            analyser(*inst, index, &reg_use_stat);
            if until(*inst) {
                return;
            }
            for reg in inst.get_regs() {
                reg_use_stat.use_reg(reg.get_color());
            }
        }
    }

    pub fn analyse_inst_with_live_now_and_index_backorder(
        bb: ObjPtr<BB>,
        analyser: &mut dyn FnMut(ObjPtr<LIRInst>, usize, &HashSet<Reg>),
    ) {
        let mut live_now = HashSet::new();
        bb.live_out.iter().for_each(|reg| {
            live_now.insert(*reg);
        });
        for (index, inst) in bb.insts.iter().enumerate().rev() {
            // for reg in inst.get_reg_def() {
            //     live_now.remove(&reg);
            // }
            analyser(*inst, index, &live_now);
            for reg in inst.get_reg_def() {
                live_now.remove(&reg);
            }
            for reg in inst.get_reg_use() {
                live_now.insert(reg);
            }
        }
    }

    //获取bb的某个区间内的自由可用(可def,可use)物理寄存器 (或者说在(from index,to index)范围内的可用寄存器)
    pub fn draw_available_of_certain_area(
        bb: &BB,
        from_index: usize,
        to_index: usize,
    ) -> RegUsedStat {
        debug_assert!(from_index < to_index);
        let mut available = RegUsedStat::init_unspecial_regs();

        bb.live_out.iter().for_each(|reg| {
            if reg.is_physic() {
                available.use_reg(reg.get_color())
            }
        });
        for (index, inst) in bb.insts.iter().enumerate().rev() {
            for reg in inst.get_reg_def() {
                if reg.is_physic() {
                    available.release_reg(reg.get_color())
                }
            }
            for reg in inst.get_reg_use() {
                if reg.is_physic() {
                    available.use_reg(reg.get_color())
                }
            }
            if index == to_index {
                break;
            }
        }

        let mut index = to_index - 1;
        while index > from_index {
            let inst = bb.insts.get(index).unwrap();
            for reg in inst.get_regs() {
                if !reg.is_physic() {
                    continue;
                }
                available.use_reg(reg.get_color());
            }
            index -= 1;
        }
        available
    }

    ///建立下次出现表,依赖于上次calc live的结果<br>
    ///表元素 (index,if_def)表示下次出现 是在def中还是在use中<br>
    /// 如果 有在同一个下标的指令中同时def和use,<br>
    /// （index,false)会在(index,true)前面
    pub fn build_next_occurs(bb: &BB) -> HashMap<Reg, LinkedList<(usize, bool)>> {
        let mut next_occurs: HashMap<Reg, LinkedList<(usize, bool)>> = HashMap::new();
        //初始化holder
        bb.live_in.iter().for_each(|reg| {
            next_occurs.insert(*reg, LinkedList::new());
        });
        // 维护一个物理寄存器的作用区间队列,每次的def和use压入栈中 (先压入use,再压入def)
        // 每个链表元素为(reg,if_def)
        for (index, inst) in bb.insts.iter().enumerate() {
            for reg in inst.get_reg_use() {
                if !next_occurs.contains_key(&reg) {
                    next_occurs.insert(reg, LinkedList::new());
                }
                next_occurs.get_mut(&reg).unwrap().push_back((index, false));
            }
            for reg in inst.get_reg_def() {
                if !next_occurs.contains_key(&reg) {
                    next_occurs.insert(reg, LinkedList::new());
                }
                next_occurs.get_mut(&reg).unwrap().push_back((index, true));
            }
        }
        bb.live_out.iter().for_each(|reg| {
            next_occurs
                .get_mut(reg)
                .unwrap()
                .push_back((bb.insts.len(), false));
        });
        for (_, b) in next_occurs.iter() {
            debug_assert!(b.len() >= 1);
        }
        next_occurs
    }

    pub fn refresh_next_occurs(
        next_occurs: &mut HashMap<Reg, LinkedList<(usize, bool)>>,
        cur_index: usize,
    ) {
        let mut to_free = Vec::new();
        for (reg, next_occurs) in next_occurs.iter_mut() {
            while !next_occurs.is_empty() && next_occurs.front().unwrap().0 <= cur_index {
                next_occurs.pop_front();
            }
            if next_occurs.len() == 0 {
                to_free.push(*reg);
            }
        }
        for reg in to_free {
            next_occurs.remove(&reg);
        }
    }
}
