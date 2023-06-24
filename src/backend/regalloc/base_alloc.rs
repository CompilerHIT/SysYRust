// 或者可以认为是没有启发的线性扫描寄存器分配

use std::{collections::{HashMap, HashSet}, fs};

use crate::{backend::{regalloc::{regalloc::Regalloc, self, structs::RegUsedStat}, instrs::BB, operand::Reg, block}, utility::{ObjPtr, ScalarType}, frontend::ast::Continue, log_file};

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
        // fs::remove_file(calout);
        let mut dstr:HashMap<i32,i32>=HashMap::new();
        let mut spillings:HashSet<i32>=HashSet::new();
        let alloc_one=|reg:&Reg,reg_used_stat:&mut RegUsedStat,dstr:&mut HashMap<i32,i32>,spillings:&mut HashSet<i32>,livenow:&mut HashSet<i32>,kind:ScalarType|{
            if reg.get_type()!=kind{ return;}
            if reg.is_allocable() {
                for live in livenow.iter() {
                    let live=*live;
                    let color=dstr.get(&live).unwrap();
                    if *color==reg.get_color() {
                        dstr.remove(&live);
                        spillings.insert(live);
                        livenow.remove(&live);
                        return;
                    }
                }
                reg_used_stat.use_reg(reg.get_color());
                return;
            }
            if !reg.is_virtual() {return;}
            if spillings.contains(&reg.get_id()) {return;}
            if livenow.contains(&reg.get_id()) { return;}
            livenow.insert(reg.get_id());
            if dstr.contains_key(&reg.get_id()) {
                // panic!("Un leagal!");
                // 
                let color=dstr.get(&reg.get_id()).unwrap();
                if !reg_used_stat.is_available_reg(*color) {
                    dstr.remove(&reg.get_id());
                    livenow.remove(&reg.get_id());
                    spillings.insert(reg.get_id());
                }else{
                    reg_used_stat.use_reg(*color);
                }
                return;
            }
            let color=reg_used_stat.get_available_reg(reg.get_type());
            // 寻找个可用颜色,否则加入spilling
            if let Some(color)=color {
                dstr.insert(reg.get_id(), color);
                reg_used_stat.use_reg(color);
            }else{
                livenow.remove(&reg.get_id());
                spillings.insert(reg.get_id());
            }
        };

        let ends_index_bb=regalloc::regalloc::ends_index_bb(func);
        let mut count =|bb:ObjPtr<BB>,kind:ScalarType|{
            let mut livenow:HashSet<i32>=HashSet::new();
            let mut reg_used_stat=RegUsedStat::new();
            // 根据live now给某个虚拟寄存器分配寄存器
            bb.live_in.iter()
                .for_each(|reg|{
                   alloc_one(&reg,&mut reg_used_stat,&mut dstr,&mut spillings,&mut livenow,kind);
            });

            for (index,inst) in bb.insts.iter().enumerate() {
                // 删除旧live now
                let ends=ends_index_bb.get(&(index as i32,bb)).unwrap();
                for reg in ends.iter() {
                    if reg.get_type()!=kind {continue;}
                    livenow.remove(&reg.get_id());
                    if reg.is_physic() {
                        reg_used_stat.release_reg(reg.get_color());
                        continue;
                    }
                    if !reg.is_virtual() {continue;}
                    if spillings.contains(&reg.get_id()) {continue;}
                    if !livenow.contains(&reg.get_id()) {continue;}
                    let color=dstr.get(&reg.get_id()).unwrap();
                    reg_used_stat.release_reg(*color);
                }
                // 加入新live now,
                for reg in inst.get_reg_def() {
                    alloc_one(&reg,&mut reg_used_stat,&mut dstr,&mut spillings,&mut livenow,kind);
                }
            }
        
            
        
        };



        for bb in func.blocks.iter() {
            count(*bb,ScalarType::Float);
            count(*bb,ScalarType::Int);
        }  

        let  (func_stack,bbstacks)=regalloc::regalloc::countStackSize(func,&spillings);
        
        // log_file!(calout,"\n\n{} final:\n",func.label);
        // log_file!(calout,"dstr:{:?}\nspillings:{:?}",dstr ,spillings);

        

        FuncAllocStat{
            stack_size: func_stack,
            bb_stack_sizes: bbstacks,
            spillings,
            dstr,
        }
        
    }
}