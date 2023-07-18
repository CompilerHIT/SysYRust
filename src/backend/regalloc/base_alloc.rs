// 或者可以认为是没有启发的线性扫描寄存器分配

use std::{
    collections::{HashMap, HashSet},
    fs,
};

use crate::{
    backend::{
        block,
        instrs::BB,
        operand::Reg,
        regalloc::{self, regalloc::Regalloc},
    },
    container::bitmap::Bitmap,
    frontend::ast::Continue,
    log_file,
    utility::{ObjPtr, ScalarType},
};

// use super::structs::FuncAllocStat;

struct RegUsedStat {
    iregs_used: i32,
    fregs_used: i32,
}
// 对于regusedstat来说，通用寄存器映射到0-31，浮点寄存器映射到32-63
impl RegUsedStat {
    pub fn new() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0,
            fregs_used: 0,
        }
    }

    pub fn is_available_ireg(&self, ireg: i32) -> bool {
        let mut unusable: HashSet<i32> = HashSet::from([0]); //保存x0
        unusable.insert(2); //保留sp
        unusable.insert(1); //保留ra
        unusable.insert(3); //保留gp
        unusable.insert(4); //保留tp寄存器
        unusable.extend(5..=7); //三个临时寄存器用来处理spill逻辑
        unusable.insert(8); //保留s0
        unusable.insert(10); //保留a0
        unusable.extend(11..=17); //保留a1-a7
        if unusable.contains(&ireg) {
            return false;
        }
        if (1 << ireg & self.iregs_used) == 0 {
            return true;
        }
        return false;
    }
    pub fn is_available_freg(&self, freg: i32) -> bool {
        let freg = freg - 32;
        let mut unusable = HashSet::new();
        unusable.insert(10); //保留a0
        unusable.extend(11..=17); //保留a1-a7
        unusable.extend([18, 19, 20]);
        if unusable.contains(&freg) {
            return false;
        }
        if (1 << freg & self.fregs_used) == 0 {
            return true;
        }
        return false;
    }

    // 获取可用寄存器数量
    pub fn num_available_regs(&self, kind: ScalarType) -> usize {
        match kind {
            ScalarType::Float => self.num_available_fregs(),
            ScalarType::Int => self.num_avialable_iregs(),
            _ => panic!("{:?}", kind),
        }
    }
    pub fn num_avialable_iregs(&self) -> usize {
        (0..=31)
            .filter(|ireg| self.is_available_ireg(*ireg))
            .count()
    }
    pub fn num_available_fregs(&self) -> usize {
        (32..=63)
            .filter(|freg| self.is_available_freg(*freg))
            .count()
    }

    // 获取一个可用的整数寄存器
    pub fn get_available_ireg(&self) -> Option<i32> {
        // 对于通用寄存器来说，x0-x4有特殊用途
        // x10-x17用来传递函数参数
        for i in 0..=31 {
            if self.is_available_ireg(i) {
                return Some(i);
            }
        }
        None
    }

    // 获取一个可用的浮点寄存器
    pub fn get_available_freg(&self) -> Option<i32> {
        // f0作为特殊浮点寄存器保持0
        for i in 32..=63 {
            if self.is_available_freg(i) {
                return Some(i);
            }
        }
        None
    }

    pub fn get_available_reg(&self, kind: ScalarType) -> Option<i32> {
        match kind {
            ScalarType::Float => self.get_available_freg(),
            ScalarType::Int => self.get_available_ireg(),
            _ => panic!("{:?}", kind),
        }
    }

    // 获取剩余的可用通用寄存器
    pub fn get_rest_iregs(&self) -> Vec<i32> {
        let mut out = Vec::new();
        for i in 1..=31 {
            if self.is_available_ireg(i) {
                out.push(i);
            }
        }
        out
    }

    // 获取剩余的可用浮点寄存器
    pub fn get_rest_fregs(&self) -> Vec<i32> {
        let mut out = Vec::new();
        for i in 32..=63 {
            if self.is_available_freg(i) {
                out.push(i);
            }
        }
        out
    }

    pub fn release_reg(&mut self, reg: i32) {
        if reg >= 0 && reg < 32 {
            self.release_ireg(reg);
        } else if reg >= 32 && reg <= 63 {
            self.release_freg(reg);
        }
    }
    pub fn use_reg(&mut self, reg: i32) {
        if reg >= 0 && reg < 32 {
            self.use_ireg(reg);
        } else if reg >= 32 && reg <= 63 {
            self.use_freg(reg);
        }
    }

    pub fn is_available_reg(&self, reg: i32) -> bool {
        if reg >= 0 && reg < 32 {
            self.is_available_ireg(reg)
        } else if reg >= 32 && reg <= 63 {
            self.is_available_freg(reg)
        } else {
            panic!("not legal reg")
        }
    }

    // 释放一个通用寄存器
    pub fn release_ireg(&mut self, reg: i32) {
        self.iregs_used &= !(1 << reg);
    }
    // 占有一个整数寄存器
    pub fn use_ireg(&mut self, reg: i32) {
        self.iregs_used |= 1 << reg;
    }
    // 释放浮点寄存器
    pub fn release_freg(&mut self, reg: i32) {
        let reg = reg - 32;
        self.fregs_used &= !(1 << reg);
    }
    // 占有一个浮点寄存器
    pub fn use_freg(&mut self, reg: i32) {
        let reg = reg - 32;
        self.fregs_used |= 1 << reg;
    }
}
pub struct Allocator {}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {}
    }
}
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> super::structs::FuncAllocStat {
        let calout = "calout.txt";
        // fs::remove_file(calout);
        let mut dstr: HashMap<i32, i32> = HashMap::new();
        let mut spillings: HashSet<i32> = HashSet::new();

        log_file!(calout, "\n\n{} start:\n", func.label);
        let alloc_one = |reg: &Reg,
                         reg_used_stat: &mut RegUsedStat,
                         dstr: &mut HashMap<i32, i32>,
                         spillings: &mut HashSet<i32>,
                         livenow: &mut HashSet<i32>,
                         kind: ScalarType| {
            if reg.get_type() != kind {
                return;
            }
            if !reg.is_virtual() {
                return;
            }
            if spillings.contains(&reg.get_id()) {
                return;
            }
            if livenow.contains(&reg.get_id()) {
                return;
            }
            livenow.insert(reg.get_id());
            if dstr.contains_key(&reg.get_id()) {
                //
                let color = dstr.get(&reg.get_id()).unwrap();
                if !reg_used_stat.is_available_reg(*color) {
                    dstr.remove(&reg.get_id());
                    spillings.insert(reg.get_id());
                } else {
                    reg_used_stat.use_reg(*color);
                }
                return;
            }
            let mut color = Option::None;
            // 寻找个可用颜色,否则加入spilling
            color = reg_used_stat.get_available_reg(kind);
            if let Some(color) = color {
                dstr.insert(reg.get_id(), color);
                reg_used_stat.use_reg(color);
            } else {
                spillings.insert(reg.get_id());
            }
        };

        let mut count = |bb: ObjPtr<BB>, kind: ScalarType| {
            if bb.label == ".LBB0_3" {
                log_file!(calout, "g?");
            }
            log_file!(calout, "block {} start", bb.label);
            log_file!(
                calout,
                "live in:{:?}\nlive out:{:?}",
                bb.live_in
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<HashSet<i32>>(),
                bb.live_in
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<HashSet<i32>>()
            );
            let mut livenow: HashSet<i32> = HashSet::new();
            let mut reg_used_stat = RegUsedStat::new();
            let mut last_use: HashMap<i32, HashSet<i32>> = HashMap::new(); //记录最后一次use
            let mut passed_regs = HashSet::new(); //记录遍历过的寄存器号
                                                  // 根据live now给某个虚拟寄存器分配寄存器
                                                  // 获取寄存器终结时间
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_use() {
                    if reg.get_type() != kind {
                        continue;
                    }
                    if !reg.is_virtual() {
                        continue;
                    }
                    if bb.live_out.contains(&reg) {
                        continue;
                    } //live out中的寄存器器 不可能有终结时间
                    if passed_regs.contains(&reg.get_id()) {
                        continue;
                    }
                    passed_regs.insert(reg.get_id());
                    if let None = last_use.get_mut(&(index as i32)) {
                        last_use.insert(index as i32, HashSet::new());
                    }
                    last_use
                        .get_mut(&(index as i32))
                        .unwrap()
                        .insert(reg.get_id());
                }
            }
            log_file!(calout, "ends:{:?}", last_use);

            bb.live_in.iter().for_each(|reg| {
                alloc_one(
                    &reg,
                    &mut reg_used_stat,
                    &mut dstr,
                    &mut spillings,
                    &mut livenow,
                    kind,
                );
            });

            for (index, inst) in bb.insts.iter().enumerate() {
                // 删除旧live now
                if let Some(ends) = last_use.get(&(index as i32)) {
                    for reg in ends.iter() {
                        livenow.remove(reg);
                        if spillings.contains(reg) {
                            continue;
                        }
                        let color = dstr.get(reg).unwrap();
                        reg_used_stat.release_reg(*color);
                    }
                }
                // 加入新live now,
                for reg in inst.get_reg_def() {
                    alloc_one(
                        &reg,
                        &mut reg_used_stat,
                        &mut dstr,
                        &mut spillings,
                        &mut livenow,
                        kind,
                    );
                }
            }
        };

        for bb in func.blocks.iter() {
            count(*bb, ScalarType::Float);
            count(*bb, ScalarType::Int);
        }

        let (func_stack, bbstacks) = regalloc::regalloc::countStackSize(func, &spillings);

        log_file!(calout, "\n\n{} final:\n", func.label);
        log_file!(calout, "dstr:{:?}\nspillings:{:?}", dstr, spillings);

        regalloc::structs::FuncAllocStat {
            stack_size: func_stack,
            bb_stack_sizes: bbstacks,
            spillings,
            dstr,
        }
    }
}
