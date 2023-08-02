use super::*;

/// 从函数中提取信息
impl Func {
    // 实现一些关于函数信息的估计和获取的方法

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
            // for reg in inst.get_reg_def() {
            //     live_now.remove(&reg);
            // }
            analyser(*inst, &live_now);
            for reg in inst.get_reg_def() {
                live_now.remove(&reg);
            }
            for reg in inst.get_reg_use() {
                live_now.insert(reg);
            }
        }
    }

    //反序分析指令直到
    pub fn analyse_inst_with_regused_backorder_until(
        bb: &BB,
        analyser: &mut dyn FnMut(ObjPtr<LIRInst>, &RegUsedStat),
        until: &dyn Fn(ObjPtr<LIRInst>) -> bool,
    ) {
        let mut reg_use_stat = RegUsedStat::new();
        bb.live_out
            .iter()
            .for_each(|reg| reg_use_stat.use_reg(reg.get_color()));
        for inst in bb.insts.iter().rev() {
            analyser(*inst, &reg_use_stat);
            if until(*inst) {
                return;
            }
            for reg in inst.get_regs() {
                reg_use_stat.release_reg(reg.get_color());
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
}

// #[cfg(predicate)]
