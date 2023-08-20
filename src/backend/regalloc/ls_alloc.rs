use std::collections::{HashMap, HashSet, VecDeque};

use biheap::{bivec::order, BiHeap};

use crate::{
    backend::{
        instrs::{Func, LIRInst, BB},
        operand::Reg,
        regalloc::structs::RegUsedStat,
    },
    container::bitmap::Bitmap,
    utility::ObjPtr,
};

use super::{structs::FuncAllocStat, *};

// 默认按照end进行排序
#[derive(PartialEq, Eq)]
struct RegInteval {
    reg: Reg,
    color: i32,
    start: usize,
    end: usize,
}
impl Ord for RegInteval {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for RegInteval {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.end.partial_cmp(&other.end)
    }
}

///深度优先遍历块建立线性序
pub fn build_linear_order(func: &Func) -> Vec<ObjPtr<LIRInst>> {
    let first_bb = func.get_first_block();
    let mut to_pass: VecDeque<ObjPtr<BB>> = VecDeque::new();
    let mut passed: HashSet<ObjPtr<BB>> = HashSet::new();
    let mut orders: Vec<ObjPtr<LIRInst>> = Vec::new();
    to_pass.push_back(first_bb);
    // 深度优先
    while !to_pass.is_empty() {
        let bb = to_pass.pop_back().unwrap();
        if passed.contains(&bb) {
            continue;
        }
        passed.insert(bb);
        orders.extend(bb.insts.iter());
        for out_bb in bb.out_edge.iter() {
            to_pass.push_back(*out_bb);
        }
    }
    orders
}

/// 时间复杂度O(NMlogN)
pub fn alloc(func: &Func) -> FuncAllocStat {
    let live_out_for_inst = func.build_live_out_for_insts();
    // 根据live out 建立live in
    let mut live_in_for_inst: HashMap<ObjPtr<LIRInst>, Bitmap> = HashMap::new();
    for bb in func.blocks.iter() {
        if bb.insts.len() == 0 {
            continue;
        }
        let first_inst = bb.insts.first().unwrap();
        let mut in_bitmap = Bitmap::new();
        bb.live_in
            .iter()
            .for_each(|reg| in_bitmap.insert(reg.bit_code() as usize));
        live_in_for_inst.insert(*first_inst, in_bitmap);
        let mut index = 1;
        while index < bb.insts.len() {
            let pre_inst = bb.insts.get(index - 1).unwrap();
            let cur_inst = bb.insts.get(index).unwrap();
            let pre_out = live_out_for_inst.get(pre_inst).unwrap();
            live_in_for_inst.insert(*cur_inst, pre_out.clone());
            index += 1;
        }
    }
    // 建立指令的线性序
    let orders = build_linear_order(func);
    // 建立寄存器在伪拓扑中的生命周期,并按照生命起点进行排序
    let mut starts: HashMap<Reg, usize> = HashMap::new();
    let mut ends: HashMap<Reg, usize> = HashMap::new();
    let mut regs = func.draw_all_regs();
    let regs2 = regs.clone();
    let regs_base = regs.clone();
    for (index, inst) in orders.iter().enumerate() {
        let defed: HashSet<Reg> = inst.get_reg_def().iter().cloned().collect();
        let mut to_rm: HashSet<Reg> = HashSet::new();
        for reg in regs.iter() {
            if defed.contains(reg)
                || live_in_for_inst
                    .get(inst)
                    .unwrap()
                    .contains(reg.bit_code() as usize)
            {
                starts.insert(*reg, index);
                to_rm.insert(*reg);
            }
        }
        regs.retain(|reg| !to_rm.contains(reg));
    }
    let mut regs2 = regs2;
    for (index, inst) in orders.iter().enumerate().rev() {
        let used: HashSet<Reg> = inst.get_reg_use().iter().cloned().collect();
        let mut to_rm: HashSet<Reg> = HashSet::new();
        for reg in regs2.iter() {
            if used.contains(reg)
                || live_out_for_inst
                    .get(inst)
                    .unwrap()
                    .contains(reg.bit_code() as usize)
            {
                ends.insert(*reg, index);
                to_rm.insert(*reg);
            }
        }
        regs2.retain(|reg| !to_rm.contains(reg));
    }

    debug_assert!({
        let m = true;
        let regs = func.draw_all_regs();
        for reg in regs.iter() {
            assert!(starts.contains_key(reg) && ends.contains_key(reg));
        }
        m
    });
    // 对全部reg 按照start顺序进行排序,并维护一个表
    let mut regs: Vec<Reg> = regs_base.iter().cloned().collect();
    regs.sort_by_cached_key(|reg| starts.get(reg).unwrap());
    let mut colors: HashMap<i32, i32> = HashMap::new();
    let mut spillings: HashSet<i32> = HashSet::new();
    let mut windows: BiHeap<RegInteval> = BiHeap::new();
    let mut reg_use_stat = RegUsedStat::new();
    for reg in regs.iter() {
        let cur_index = starts.get(reg).unwrap();
        // 判断当前是否有终结日期小于cur_index的寄存器,如果有,则可以释放
        while !windows.is_empty() {
            let min = windows.peek_min().unwrap().end;
            if &min < cur_index {
                let RegInteval {
                    reg,
                    color,
                    start,
                    end,
                } = windows.pop_min().unwrap();
                if !reg.is_physic() {
                    colors.insert(reg.get_id(), color);
                }
                reg_use_stat.release_reg(color);
            } else {
                break;
            }
        }
    }

    //alloc
    todo!();
}
