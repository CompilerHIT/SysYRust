// 或者可以认为是没有启发的线性扫描寄存器分配

use std::collections::{HashMap, HashSet};

use crate::{backend::{regalloc::{regalloc::Regalloc, self, structs::RegUsedStat}, instrs::BB, operand::Reg}, utility::{ObjPtr, ScalarType}, frontend::ast::Continue};

use super::structs::FuncAllocStat;

pub struct Allocator {

}

impl Allocator {
    pub fn new()->Allocator {
        Allocator {  }
    }
}
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> super::structs::FuncAllocStat {
        let mut dstr:HashMap<i32,i32>=HashMap::new();
        let mut spillings:HashSet<i32>=HashSet::new();

        let mut  alloc_one=|reg:&Reg,regUsedStat:&mut RegUsedStat,dstr:&mut HashMap<i32,i32>,spillings:&mut HashSet<i32>,livenow:&mut HashSet<i32>|{
            if spillings.contains(&reg.get_id()) {return;}
            if livenow.contains(&reg.get_id()) { return;}
            livenow.insert(reg.get_id());
            if dstr.contains_key(&reg.get_id()) {
                let color=dstr.get(&reg.get_id()).unwrap();
                regUsedStat.use_reg(*color);
                return;

            }
            // 寻找个可用颜色,否则加入spilling
            if reg.get_type()==ScalarType::Int {
                let icolor=regUsedStat.get_available_ireg();
                if let Some(icolor)=icolor {
                    regUsedStat.use_ireg(icolor);
                    dstr.insert(reg.get_id(), icolor);
                }else {spillings.insert(reg.get_id());}
            }else if reg.get_type()==ScalarType::Float {
                let fcolor=regUsedStat.get_available_freg();
                if let Some(fcolor)=fcolor {
                    regUsedStat.use_freg(fcolor);
                    dstr.insert(reg.get_id(), fcolor);
                }else{
                    spillings.insert(reg.get_id());
                }
            }else{
                //？
            }
        };

        let mut count =|bb:ObjPtr<BB>|{
            let mut livenow:HashSet<i32>=HashSet::new();
            let mut regUsedStat=RegUsedStat::new();
            let mut last_use:HashMap<i32,HashSet<i32>> =HashMap::new();  //记录最后一次use
            let mut passed_regs=HashSet::new();
            // 根据live now给某个虚拟寄存器分配寄存器

            for (index,inst) in bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_use(){
                    if !reg.is_virtual() {continue;}
                    if passed_regs.contains(&reg.get_id()) {continue;}
                    passed_regs.insert(reg.get_id());
                    if let None=last_use.get_mut(&(index as i32)) {
                        last_use.insert(index as i32 , HashSet::new());
                    }
                    last_use.get_mut(&(index as i32)).unwrap().insert(reg.get_id());
                }
            }
            bb.live_in.iter()
                .for_each(|reg|{
                   alloc_one(&reg,&mut regUsedStat,&mut dstr,&mut spillings,&mut livenow);
            });

            for (index,inst) in bb.insts.iter().enumerate() {
                // 加入新live now,
                for reg in inst.get_reg_def() {
                    alloc_one(&reg,&mut regUsedStat,&mut dstr,&mut spillings,&mut livenow);
                }
                // 删除旧live now
                if let Some(ends)=last_use.get(&(index as i32)){
                    for reg in ends.iter() {
                        livenow.remove(reg);
                        let color=dstr.get(reg).unwrap();
                        regUsedStat.release_reg(*color);
                    }
                }
            }
        };
        for bb in func.blocks.iter() {
            count(*bb);
        }  
        let  (func_stack,bbstacks)=regalloc::easy_ls_alloc::Allocator::countStackSize(func,&spillings);
        
        FuncAllocStat{
            stack_size: func_stack,
            bb_stack_sizes: bbstacks,
            spillings,
            dstr,
        }
    }
}