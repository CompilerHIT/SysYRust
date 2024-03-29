use crate::backend::regalloc::{easy_gc_alloc, ls_alloc, perfect_alloc};

use super::*;

impl Func {
    /// 计算活跃区间的时候, 主动把6个临时寄存器的生存周期设置为无限(zero,ra,sp,gp,tp,s0)
    /// 并且选择3个临时通用寄存器和3个临时浮点寄存器,也设置为无线
    /// i5-7 和 f18-20
    pub fn calc_live_for_alloc_reg(&self) {
        self.calc_live_base();
        //把sp和ra寄存器加入到所有的块的live out,live in中，表示这些寄存器永远不能在函数中自由分配使用
        for bb in self.blocks.iter() {
            //0:zero, 1:ra, 2:sp,3:gp,4:tp 是必须保存的,5-7做临时寄存器
            //8:s0用于处理overflow
            for id in 0..=8 {
                bb.as_mut().live_in.insert(Reg::new(id, ScalarType::Int));
                bb.as_mut().live_out.insert(Reg::new(id, ScalarType::Int));
            }
            for id in 18..=20 {
                bb.as_mut()
                    .live_in
                    .insert(Reg::new(id + FLOAT_BASE, ScalarType::Float));
                bb.as_mut()
                    .live_out
                    .insert(Reg::new(id + FLOAT_BASE, ScalarType::Float));
            }
        }
    }

    ///显式禁止使用某些寄存器的分配方式
    pub fn alloc_reg_without(&mut self, unavailables: &HashSet<Reg>) {
        config::record_event("start calc live");
        self.calc_live_base();
        config::record_event("finish calc live");
        for bb in self.blocks.iter() {
            for reg in unavailables.iter() {
                bb.as_mut().live_in.insert(*reg);
                bb.as_mut().live_out.insert(*reg);
            }
        }
        // // 加入线性扫描(如果代码行数大于某个阈值,则启动线性扫描)
        if self.num_insts() > 10_0000 {
            config::record_event("start ls alloc");
            let alloc_stat = ls_alloc::alloc(&self);
            regalloc::check_alloc_v2(&self, &alloc_stat.dstr, &alloc_stat.spillings);
            self.reg_alloc_info = alloc_stat;
            self.context.as_mut().set_reg_map(&self.reg_alloc_info.dstr);
            config::record_event("finish ls alloc");
            return;
        }
        if self.num_insts() < 5_0000 {
            config::record_event("start  perfect alloc");
            let alloc_stat = perfect_alloc::alloc_with_constraints(&self, &HashMap::new());
            if alloc_stat.is_some() {
                let alloc_stat = alloc_stat.unwrap();
                regalloc::check_alloc_v2(&self, &alloc_stat.dstr, &alloc_stat.spillings);
                self.reg_alloc_info = alloc_stat;
                self.context.as_mut().set_reg_map(&self.reg_alloc_info.dstr);
                return;
            }
        }
        config::record_event("start easygc alloc");
        let alloc_stat = easy_gc_alloc::alloc(&self);
        regalloc::check_alloc_v2(&self, &alloc_stat.dstr, &alloc_stat.spillings);
        self.reg_alloc_info = alloc_stat;
        self.context.as_mut().set_reg_map(&self.reg_alloc_info.dstr);
        config::record_event("finish easygc alloc");
        return;
    }
}
