use std::collections::HashSet;

use crate::backend::operand::Reg;

use super::AsmModule;

impl AsmModule {
    /// 第一次运行v2p时不映射临时寄存器，第二次运行前清空tmp_vars set
    pub fn map_v_to_p(&mut self) {
        self.name_func.iter().for_each(|(_, func)| {
            if func.is_extern {
                return;
            }
            func.blocks.iter().for_each(|block| {
                block.insts.iter().for_each(|inst| {
                    inst.as_mut()
                        .v_to_phy(func.context.get_reg_map().clone(), func.tmp_vars.clone());
                });
            });
        });
    }
    fn p2v(&mut self) {
        self.name_func.iter().for_each(|(_, func)| {
            if func.is_extern {
                return;
            }
            func.as_mut().p2v(&Reg::get_all_recolorable_regs());
        });
    }

    pub fn alloc_without_tmp_and_s0(&mut self) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            if func.is_extern {
                return;
            }
            let mut unavailables = Reg::get_all_tmps();
            unavailables.insert(Reg::get_s0());
            func.as_mut().alloc_reg_without(&unavailables);
            // func.as_mut().allocate_reg();
        });
    }

    ///在handle spill前进行的最后一次重分配,仍然保留tmps,s0
    pub fn first_realloc(&mut self) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            if func.is_extern {
                return;
            }
            if func.reg_alloc_info.spillings.len() == 0 {
                return;
            }
            //
            func.as_mut().p2v(&Reg::get_all_recolorable_regs());
            let mut unavailables = Reg::get_all_tmps();
            unavailables.insert(Reg::get_s0());
            func.as_mut().alloc_reg_without(&unavailables);
            func.as_mut().v2p(&func.reg_alloc_info.dstr);
        });
    }

    ///TODO,在handle spill前完成一次的handle spill,
    pub fn alloc_without_s0(&mut self) {
        self.name_func.iter_mut().for_each(|(_, func)| {
            // log!("allocate reg fun: {}", func.as_ref().label);
            debug_assert!(!func.is_extern);
            let mut unavailables = HashSet::new();
            unavailables.insert(Reg::get_s0());
            func.as_mut().alloc_reg_without(&unavailables);
        });
    }
}
