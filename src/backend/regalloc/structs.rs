use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::ops::Range;

use crate::backend::block::BB;
use crate::utility::ObjPtr;
use crate::utility::ScalarType;

#[derive(Clone, Copy)]
pub struct RegUsedStat {
    iregs_used: u32,
    fregs_used: u32,
}

// 对于regusedstat来说，通用寄存器映射到0-31，浮点寄存器映射到32-63
impl RegUsedStat {
    pub fn new() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0,
            fregs_used: 0,
        }
    }

    // 产生i专用的初始使用情况
    pub fn init_for_i() -> RegUsedStat {
        let mut out = RegUsedStat::new();
        for reg in out.get_rest_fregs() {
            out.use_reg(reg);
        }
        out
    }

    // 产生f专用的初始使用情况
    pub fn init_for_f() -> RegUsedStat {
        let mut out = RegUsedStat::new();
        for reg in out.get_rest_iregs() {
            out.use_reg(reg);
        }
        out
    }

    pub fn init_for_reg(kind: ScalarType) -> RegUsedStat {
        match kind {
            ScalarType::Float => RegUsedStat::init_for_f(),
            ScalarType::Int => RegUsedStat::init_for_i(),
            _ => panic!(),
        }
    }

    // 判断是否有多余的寄存器
    pub fn is_available(&self, kind: ScalarType) -> bool {
        // TODO,使用位运算加速过程
        match kind {
            ScalarType::Float => self.num_available_fregs() != 0,
            ScalarType::Int => self.num_avialable_iregs() != 0,
            _ => false,
        }
    }

    pub fn is_available_ireg(&self, ireg: i32) -> bool {
        let mut unusable: HashSet<i32> = HashSet::from([0]); //保存x0
        unusable.insert(2); //保留sp
        unusable.insert(1); //保留ra
        unusable.insert(3); //保留gp

        unusable.insert(4); //保留tp寄存器
        unusable.extend(5..=7); //三个临时寄存器用来处理spill逻辑
                                // unusable.insert(8); //保留fp/s0寄存器
        unusable.insert(10); //保留a0
                             // unusable.extend(11..=17);   //保留a1-a7
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
        let mut unusable: HashSet<i32> = HashSet::from([18, 19, 20]);
        // unusable.extend(0..20);
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
        for i in 0..=31 {
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

    // 获取剩余可用的某类寄存器
    pub fn get_rest_regs_for(&self, kind: ScalarType) -> Vec<i32> {
        match kind {
            ScalarType::Float => self.get_rest_fregs(),
            ScalarType::Int => self.get_rest_iregs(),
            _ => panic!("gg"),
        }
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

impl Display for RegUsedStat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "i:{:?},f:{:?}",
            self.get_rest_iregs(),
            self.get_rest_fregs()
        )
    }
}

#[derive(Clone)]
pub struct FuncAllocStat {
    pub stack_size: usize,
    pub bb_stack_sizes: HashMap<ObjPtr<BB>, usize>, //统计翻译bb的时候前面已经用过的栈空间
    pub spillings: HashSet<i32>,                    //spilling regs
    pub dstr: HashMap<i32, i32>,                    //distribute regs
}

impl FuncAllocStat {
    pub fn new() -> FuncAllocStat {
        let mut out = FuncAllocStat {
            spillings: HashSet::new(),
            stack_size: 0,
            bb_stack_sizes: HashMap::new(),
            dstr: HashMap::new(),
        };
        for i in 0..=63 {
            out.dstr.insert(i, i);
        }
        out
    }
}

pub struct BlockAllocStat {
    spillings: HashSet<i32>,
    dstr: HashMap<i32, i32>,
}

impl BlockAllocStat {
    pub fn new() -> BlockAllocStat {
        BlockAllocStat {
            spillings: HashSet::new(),
            dstr: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod test_regusestat {
    use crate::backend::regalloc::regalloc::Regalloc;

    use super::RegUsedStat;

    #[test]
    fn test_num() {
        // TODO
        let a = RegUsedStat::new();
        assert_eq!(a.num_available_fregs(), 32);
        assert_eq!(a.num_avialable_iregs(), 23); //保留t0-t2三个临时寄存器,sp,a0,tp,x0,gp五个个特殊寄存器,保留a0用作返回值
        assert_eq!(a.num_available_regs(crate::utility::ScalarType::Float), 32);
        assert_eq!(a.num_available_regs(crate::utility::ScalarType::Int), 23);
    }

    #[test]
    fn test_base() {
        let mut m = RegUsedStat::new();
        for i in 0..=31 {
            m.use_ireg(i);
        }
        assert!(m.iregs_used == 0x_FFFF_FFFF);
        assert_eq!(m.iregs_used, 0b_1111_1111_1111_1111_1111_1111_1111_1111)
    }
}
