// 或者可以认为是没有启发的线性扫描寄存器分配

use std::collections::{HashMap, HashSet};

use crate::{backend::{regalloc::{regalloc::Regalloc, self, structs::RegUsedStat}, instrs::BB, operand::Reg}, utility::{ObjPtr, ScalarType}, frontend::ast::Continue, log_file};

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
        let  calout="callive.txt";
        let mut dstr:HashMap<i32,i32>=HashMap::new();
        let mut spillings:HashSet<i32>=HashSet::new();

        log_file!(calout,"\n\n{} start:\n",func.label);
        let alloc_one=|reg:&Reg,reg_used_stat:&mut RegUsedStat,dstr:&mut HashMap<i32,i32>,spillings:&mut HashSet<i32>,livenow:&mut HashSet<i32>|{
            if !reg.is_virtual() {return;}
            if spillings.contains(&reg.get_id()) {return;}
            if livenow.contains(&reg.get_id()) { return;}
            livenow.insert(reg.get_id());
            if dstr.contains_key(&reg.get_id()) {
                // 
                let color=dstr.get(&reg.get_id()).unwrap();
                if !reg_used_stat.is_available_reg(*color) {
                    dstr.remove(&reg.get_id());
                    spillings.insert(reg.get_id());
                }else{
                    reg_used_stat.use_reg(*color);
                }
                return;
            }
            let mut color=Option::None;
            // 寻找个可用颜色,否则加入spilling
            if reg.get_type()==ScalarType::Int {
                color=reg_used_stat.get_available_ireg();
               
            }else if reg.get_type()==ScalarType::Float {
                color=reg_used_stat.get_available_freg();
            }
            if let Some(color)=color {
                dstr.insert(reg.get_id(), color);
                reg_used_stat.use_reg(color);
            }else{
                spillings.insert(reg.get_id());
            }

        };

        let mut count =|bb:ObjPtr<BB>|{
            let mut livenow:HashSet<i32>=HashSet::new();
            let mut reg_used_stat=RegUsedStat::new();
            let mut last_use:HashMap<i32,HashSet<i32>> =HashMap::new();  //记录最后一次use
            let mut passed_regs=HashSet::new();
            // 根据live now给某个虚拟寄存器分配寄存器
            // 获取寄存器终结时间
            for (index,inst) in bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_use(){
                    if !reg.is_virtual() {continue;}
                    if bb.live_out.contains(&reg) {continue;}   //live out中的寄存器器 不可能有终结时间
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
                   alloc_one(&reg,&mut reg_used_stat,&mut dstr,&mut spillings,&mut livenow);
            });

            for (index,inst) in bb.insts.iter().enumerate() {
                // 加入新live now,
                for reg in inst.get_regs() {
                    alloc_one(&reg,&mut reg_used_stat,&mut dstr,&mut spillings,&mut livenow);
                }
                // 删除旧live now
                if let Some(ends)=last_use.get(&(index as i32)){
                    for reg in ends.iter() {
                        livenow.remove(reg);
                        if spillings.contains(reg) {continue;}
                        let color=dstr.get(reg).unwrap();
                        reg_used_stat.release_reg(*color);
                    }
                }

            }
        };

        // let mut remove_spillings=|bb:ObjPtr<BB>| {
        //     let mut livenow:HashSet<i32>=HashSet::new();
        //     let mut regusestat=RegUsedStat::new();
        //     // 先取出所有live out
        //     let mut ends:HashMap<i32,HashSet<i32>>=HashMap::new();
        //     let mut passedreg=HashSet::new();
        //     // 记录终点
        //     for (index,it) in bb.insts.iter().enumerate().rev() {
        //         for reg in it.get_reg_use() {
        //             if !reg.is_virtual() {continue;}
        //             if passedreg.contains(&reg.get_id()) {continue;}
        //             passedreg.insert(reg.get_id());
        //             if !ends.contains_key(&(index as i32)) {ends.insert(index as i32, HashSet::new());}
        //             ends.get_mut(&(index as i32)).unwrap().insert(reg.get_id()); 
        //         }
        //     }
            
        //     let checkallocone=|reg:&Reg|{
        //         if (!reg.is_virtual()) {return ;}
        //         if (spillings.contains(&reg.get_id())) {return;}
        //         let color =dstr.get(&reg.get_id()).unwrap();
        //         if regusestat.is_available_reg(*color) {regusestat.use_reg(*color);}
        //         else {
        //             dstr.remove(&reg.get_id());
        //             spillings.insert(reg.get_id());
        //         }
        //     }; 
        //     // 检查分配
        //     bb.live_in.iter().for_each(checkallocone);
        //     for (index,it) in bb.insts.iter().enumerate() {
        //         // 
        //         if let Some(regs)=ends.get(&(index as i32)) {
        //             for reg in regs.iter() {
                        
        //             }
        //         }
        //     }

        // };


        for bb in func.blocks.iter() {
            count(*bb);
        }  

        let  (func_stack,bbstacks)=regalloc::regalloc::countStackSize(func,&spillings);
        
        log_file!(calout,"\n\n{} final:\n",func.label);
        log_file!(calout,"dstr:{:?}\nspillings:{:?}",dstr ,spillings);



        FuncAllocStat{
            stack_size: func_stack,
            bb_stack_sizes: bbstacks,
            spillings,
            dstr,
        }
        
    }
}