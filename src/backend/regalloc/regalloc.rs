use std::collections::{HashMap, HashSet, VecDeque};
use std::process::Output;

use crate::backend::block;
use crate::backend::func::Func;
use crate::backend::instrs::{Operand, BB, InstrsType};
use crate::backend::operand::Reg;
use crate::backend::regalloc::structs::FuncAllocStat;
use crate::frontend::ast::Continue;
use crate::ir::instruction::Inst;
use crate::utility::{ObjPtr, ScalarType};

use super::structs::RegUsedStat;

// 该处理下，全局量被翻译到内存中，
// 以函数为寄存器分配的基本单位
pub trait Regalloc {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat;
}

pub fn countStackSize(
    func: &Func,
    spillings: &HashSet<i32>,
) -> (usize, HashMap<ObjPtr<BB>, usize>) {
    // 遍历所有块,找到每个块中的spillings大小,返回其中大小的最大值,
    let mut stackSize: usize = 0;
    let mut bb_stack_sizes: HashMap<ObjPtr<BB>, usize> = HashMap::new();
    let mut passed: HashSet<ObjPtr<BB>> = HashSet::new();
    let mut walk: VecDeque<ObjPtr<BB>> = VecDeque::new();
    walk.push_back(func.entry.unwrap().clone());
    passed.insert(func.entry.unwrap());
    // TOTEST
    while !walk.is_empty() {
        let cur = walk.pop_front().unwrap();
        let mut bbspillings: HashSet<i32> = HashSet::new();
        //println!("{}",cur.label);
        for reg in &cur.as_ref().live_in {
            if spillings.contains(&reg.get_id()) {
                bbspillings.insert(reg.get_id());
            }
        }
        let start = bbspillings.len() * 8;
        bb_stack_sizes.insert(cur, start);
        bbspillings.clear();
        // 统计spilling数量
        for inst in &cur.as_ref().insts {
            for reg in inst.as_ref().get_reg_def() {
                if spillings.contains(&reg.get_id()) {
                    bbspillings.insert(reg.get_id());
                }
            }
            for reg in inst.as_ref().get_reg_use() {
                if spillings.contains(&reg.get_id()) {
                    bbspillings.insert(reg.get_id());
                }
            }
        }
        if bbspillings.len() * 8 + start > stackSize {
            stackSize = bbspillings.len() * 8 + start;
        }
        // 扩展未扩展的节点
        for bb in &cur.as_ref().out_edge {
            if passed.contains(&bb) {
                continue;
            }
            passed.insert(bb.clone());
            walk.push_back(bb.clone());
        }
    }
    (spillings.len() * 8, bb_stack_sizes)
}

// 计算某个寄存器spill可能造成的冲突代价
// 它作为某个指令的def的时候冲突代价为2
// 作为某个指令的def以及use的时候冲突代价为2
// 只作为某个指令的use的时候冲突代价为1
// 该结果为一种朴素的结果
pub fn count_spill_confict(func: &Func) -> HashMap<i32, i32> {
    let mut out: HashMap<i32, i32> = HashMap::new();
    //
    for bb in func.blocks.iter() {
        for inst in bb.insts.iter() {
            let dst_reg= match inst.get_reg_def().get(0){
                Some(reg)=>Some(*reg),
                None=>None,
            }; 
            // TODO,判断这个指令是否有目标寄存器
            
            let mut is_use = false;
            for reg in inst.get_reg_use() {
                if !reg.is_virtual() {
                    continue;
                }
                if let Some(treg) = dst_reg {
                    if treg == reg {
                        is_use = true
                    }
                }
                out.insert(reg.get_id(), out.get(&reg.get_id()).unwrap_or(&0) + 1);
            }
            for reg in inst.get_reg_def() {
                if !reg.is_virtual() {
                    continue;
                }
                if is_use {
                    out.insert(reg.get_id(), out.get(&reg.get_id()).unwrap_or(&0) + 1);
                } else {
                    out.insert(reg.get_id(), out.get(&reg.get_id()).unwrap_or(&0) + 2);
                }
            }
        }
    }
    out
}


// TODO 获取冲突表
pub fn interferences_reg(func:&Func)->HashMap<i32,HashSet<i32>> {

    todo!()
}



// 获取 （下标,块)->失效寄存器集合  表
pub fn ends_index_bb(func:& Func)->HashMap<(i32,ObjPtr<BB>),HashSet<Reg>> {
    // 获取reg
    let mut out:HashMap<(i32,ObjPtr<BB>),HashSet<Reg>> =HashMap::new();
    let mut passed_regs:HashSet<Reg>=HashSet::new();
    for bb in func.blocks.iter(){
        for (index,inst) in bb.insts.iter().enumerate().rev() {
            for reg in inst.get_reg_use() {
                if bb.live_out.contains(&reg) {continue;}
                if passed_regs.contains(&reg) {continue;}
                passed_regs.insert(reg);
                if let None=out.get(&(index as i32,*bb)) {
                    out.insert((index as i32,*bb), HashSet::new());
                }
                out.get_mut(&(index as i32,*bb)).unwrap().insert(reg);
            }
        }
    }
    out
}


// 通用寄存器合并
pub fn merge_alloc(func: &Func, dstr: &mut HashMap<i32, i32>, spillings: &HashSet<i32>) {
    // 合并条件,如果一个mv x55 x66指令， 后面 x66指令不再使用了,
    // 则x55(color1),x66(color2)可以进行合并，
    // 可以取一个它们合并之后不会产生新的矛盾的颜色合并
    let ends_index_bb=ends_index_bb(func);
    let mut availables: HashMap<i32, RegUsedStat> = HashMap::new();
    // let reg_use_stat=
    // 首先进行冲突分析分析出它们的剩余可用颜色
    let mut analyse_one = |livenow: &mut VecDeque<Reg>,
                       reg: &Reg,
                       dstr: &mut HashMap<i32, i32>,
                       spillings: &HashSet<i32>| {
        if !reg.is_virtual() {
            return;
        }
        if spillings.contains(&reg.get_id()) {
            return;
        }
        if let None = availables.get(&reg.get_id()) {
            availables.insert(reg.get_id(), RegUsedStat::new());
        }
        let color = dstr.get(&reg.get_id()).unwrap();
        for reg_another in livenow.iter() {
            let available = availables.get_mut(&reg.get_id()).unwrap();
            let color_another = dstr.get(&reg_another.get_id()).unwrap();
            available.use_reg(*color_another);
            let available_another = availables.get_mut(&reg_another.get_id()).unwrap();
            available_another.use_reg(*color);
        }
        livenow.push_back(*reg);
    };
    let mut analyse = |bb: ObjPtr<BB>, dstr: &mut HashMap<i32, i32>, spillings: &HashSet<i32>| {
        // 对某个块进行冲突分析
        // 获取每个寄存器的终结时间
        // 获取寄存器链表
        let mut livenow: VecDeque<Reg> = VecDeque::new();
        bb.live_in.iter().for_each(|reg|analyse_one(&mut livenow,reg,dstr,spillings));
        for (index, inst) in bb.insts.iter().enumerate().rev() {
            // 先对live now中的参数进行排序,找到结束时间最晚的一个
            let mut i=0;
            while i<livenow.len() {
                
            } {
                let tmp_set:HashSet<Reg>=HashSet::new();
                let tmp_reg=livenow.get(i).unwrap();
                if ends_index_bb.get(&(index as i32,bb)).unwrap_or(&tmp_set).contains(tmp_reg) {
                    livenow.remove(i);
                }else{
                    i+=1;
                }
            }

            for reg in inst.get_reg_def() {
                analyse_one(&mut livenow,&reg,dstr,spillings);
            }
        }
        
    };
    // 分析寄存器的availble关系
    for block in func.blocks.iter() {
        analyse(*block, dstr, spillings);
    }
    
    let merge = |bb: ObjPtr<BB>,availables:& HashMap<i32, RegUsedStat>, dstr: &mut HashMap<i32, i32>,spillings: &HashSet<i32>| {
        // 首先定位到可能出现merge的指令，比如mv
        let tmp_set:HashSet<i32>=HashSet::new();
        for  (index ,inst) in bb.insts.iter().enumerate() {
            if inst.get_type()!=InstrsType::OpReg(crate::backend::instrs::SingleOp::IMv) {
                continue;
            }
            let dst_reg=inst.get_reg_def().get(0).unwrap().get_id();
            let src_reg=inst.get_reg_use().get(0).unwrap().get_id();
            if dst_reg==src_reg {
                continue;
            }
            // fixme
            // if ends_index_bb.get(&(index as i32,bb)).unwrap_or(&tmp_set).contains(&src_reg) {
            //     //TODO

            // }

        }



    };
    // 根据冲突结果进行寄存器合并
    for block in func.blocks.iter() {
        merge(*block, &availables  ,dstr,spillings);
    }
}


// 通用寄存器分配结果检查,判断是否仍然存在冲突情况,若存在,返回冲突的寄存器集合以及所在的指令编号，块标识符)
// (old_reg,cur_reg,inst index,block label)
pub fn check_alloc(func: & Func, dstr: &HashMap<i32, i32>, spillings: &HashSet<i32>) -> Vec<(i32,i32,i32, String)> {
    let mut out:Vec<(i32,i32,i32, String)>=Vec::new();
    let ends_index_bb=ends_index_bb(func);
    let mut check_alloc_one=|reg:&Reg,index:i32,bb:ObjPtr<BB>,reg_use_stat:&mut RegUsedStat,livenow:&mut HashMap<i32, HashSet<i32>>| {
        if spillings.contains(&reg.get_id()) {return;}
        if reg.is_physic() {
            if reg.get_type()==ScalarType::Float {
                //fixme
            }else if reg.get_type()==ScalarType::Int {
                reg_use_stat.use_ireg(reg.get_id())
            }
            return;
        }
        // println!("g?{}",reg.get_id());
        let color=dstr.get(&reg.get_id()).unwrap();
        // 
        if !reg_use_stat.is_available_reg(*color) {
            let interef_regs=livenow.get(color).unwrap();
            if interef_regs.contains(&reg.get_id()) {return;}
            for interef_reg in interef_regs.iter() {
                out.push( (*interef_reg,reg.get_id(),index,bb.label.clone()));
            }
        }
        reg_use_stat.use_reg(*color);
        livenow.get_mut(color).unwrap().insert(reg.get_id());
    };
    for bb in func.blocks.iter() {
        let mut reg_use_stat=RegUsedStat::new();
        let mut livenow:HashMap<i32,HashSet<i32>>=HashMap::new();
        for i in 0..=63 {
            livenow.insert(i, HashSet::new());
        }
        
        bb.live_in.iter().for_each(|reg|check_alloc_one(reg,-1,*bb,&mut reg_use_stat, &mut livenow));
        for (index,inst) in  bb.insts.iter().enumerate() {
            // 先處理生命周期結束的寄存器
            if let Some(end_regs) = ends_index_bb.get(&(index as i32,*bb)) {
                for reg in end_regs {
                    if spillings.contains(&reg.get_id()) {continue;}
                    if !reg.is_virtual() {continue;}
                    // if reg.is_physic() {
                    //     continue;
                    // }
                    // if !reg.is_allocable() {
                    //     //TODO
                    //     continue;
                    // }
                    println!("{}",reg.get_id());
                    let color =dstr.get(&reg.get_id()).unwrap();
                    livenow.get_mut(color).unwrap().remove(&reg.get_id());
                    if livenow.get(color).unwrap().is_empty() {
                        reg_use_stat.release_reg(*color);
                    }
                }    
            }
            for reg in inst.get_reg_def() {
                check_alloc_one(&reg,index as i32,*bb,&mut reg_use_stat,&mut livenow);
            }
        }
    }
    out
}

