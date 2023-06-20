use std::collections::{HashSet, HashMap, VecDeque};

use crate::backend::func::Func;
use crate::backend::instrs::{BB, Operand};
use crate::backend::operand::Reg;
use crate::backend::regalloc::structs::FuncAllocStat;
use crate::utility::ObjPtr;

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
    (spillings.len()*8, bb_stack_sizes)
}


// 计算某个寄存器spill可能造成的冲突代价
// 它作为某个指令的def的时候冲突代价为2
// 作为某个指令的def以及use的时候冲突代价为2
// 只作为某个指令的use的时候冲突代价为1
// 该结果为一种朴素的结果
pub fn count_spill_confict(func:&Func)->HashMap<i32,i32> {
    let mut out:HashMap<i32,i32>=HashMap::new();
    //
    for bb in func.blocks.iter() {
        for inst in bb.insts.iter() {
            let mut dst_reg:Option<Reg>=None;
            if let Operand::Reg(r)=inst.get_dst() {
                dst_reg=Option::Some(*r);
            }
            let mut is_use=false;
            for reg in inst.get_reg_use() {
                if !reg.is_virtual() {continue;}
                if let Some(treg)=dst_reg {
                    if treg==reg {
                        is_use=true
                    }
                }
                out.insert(reg.get_id(), out.get(&reg.get_id()).unwrap_or(&0)+1);
            }
            for reg in inst.get_reg_use() {
                if !reg.is_virtual() {continue;}
                if is_use {
                    out.insert(reg.get_id(), out.get(&reg.get_id()).unwrap_or(&0)+1);
                }else{
                    out.insert(reg.get_id(), out.get(&reg.get_id()).unwrap_or(&0)+2);
                }
            }
            
        }   
    }
    out
}


// 通用寄存器合并
