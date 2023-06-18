use std::collections::{HashSet, HashMap, VecDeque};

use crate::backend::func::Func;
use crate::backend::instrs::BB;
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