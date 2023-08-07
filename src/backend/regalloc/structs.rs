use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::format;
use std::fmt::Display;
use std::vec;

use crate::backend::block::BB;
use crate::backend::operand::Reg;
use crate::utility::ObjPtr;
use crate::utility::ScalarType;

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub struct RegUsedStat {
    iregs_used: u32,
    fregs_used: u32,
}

// 对于regusedstat来说，通用寄存器映射到0-31，浮点寄存器映射到32-63
impl RegUsedStat {
    pub const fn new() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0,
            fregs_used: 0,
        }
    }

    // 产生i专用的初始使用情况
    pub const fn init_for_i() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0b_0000_0000_0000_0000_0000_0000_0000_0000,
            fregs_used: 0b_1111_1111_1111_1111_1111_1111_1111_1111,
        }
    }

    // 产生f专用的初始使用情况
    pub const fn init_for_f() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0b_1111_1111_1111_1111_1111_1111_1111_1111,
            fregs_used: 0b_0000_0000_0000_0000_0000_0000_0000_0000,
        }
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
            ScalarType::Int => self.num_available_iregs() != 0,
            _ => false,
        }
    }

    pub fn is_available_ireg(&self, ireg: i32) -> bool {
        if (1 << ireg & self.iregs_used) == 0 {
            return true;
        }
        return false;
    }
    pub fn is_available_freg(&self, freg: i32) -> bool {
        let freg = freg - 32;
        if (1 << freg & self.fregs_used) == 0 {
            return true;
        }
        return false;
    }

    // 获取可用寄存器数量
    pub fn num_available_regs(&self, kind: ScalarType) -> usize {
        match kind {
            ScalarType::Float => self.num_available_fregs(),
            ScalarType::Int => self.num_available_iregs(),
            _ => panic!("{:?}", kind),
        }
    }

    pub fn num_available_iregs(&self) -> usize {
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

        // TODO,检查使用优先级
        // 修改为优先使用参数寄存器
        // 其次使用callee save寄存器
        // 然后使用剩余 caller save寄存器
        let args = 10..=17;
        let other_caller_save = vec![1, 5, 6, 7, 28, 29, 30, 31];
        let mut callees = vec![2, 8, 9];
        callees.extend(18..=27);

        // 优先使用callee saved寄存器,再使用参数寄存器,最后再使用其他caller save寄存器
        for reg in callees {
            if self.is_available_ireg(reg) {
                return Some(reg);
            }
        }

        for reg in args {
            if self.is_available_ireg(reg) {
                return Some(reg);
            }
        }

        for reg in other_caller_save {
            if self.is_available_ireg(reg) {
                return Some(reg);
            }
        }

        None
    }

    // 获取一个可用的浮点寄存器
    pub fn get_available_freg(&self) -> Option<i32> {
        // 优先使用callee saved寄存器,再使用参数寄存器,最后再使用其他caller save寄存器
        let args = 10..=17;
        let mut other_caller_save = vec![];
        other_caller_save.extend(0..=7);
        other_caller_save.extend(28..=31);
        let mut callees = vec![8, 9];
        callees.extend(18..=27);

        for reg in callees {
            let reg = reg + 32;
            if self.is_available_freg(reg) {
                return Some(reg);
            }
        }
        for reg in args {
            let reg = reg + 32;
            if self.is_available_freg(reg) {
                return Some(reg);
            }
        }

        for reg in other_caller_save {
            let reg = reg + 32;
            if self.is_available_freg(reg) {
                return Some(reg);
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

//
impl RegUsedStat {
    pub fn merge(&mut self, other: &RegUsedStat) {
        self.fregs_used |= other.fregs_used;
        self.iregs_used |= other.iregs_used;
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

impl RegUsedStat {
    /// 获取除了 x0-x5以外的寄存器集合
    /// x0-x5: zero,ra,sp,gp,tp
    pub const fn init_unspecial_regs() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0b_0000_0000_0000_0000_0000_0000_0001_1111,
            fregs_used: 0,
        }
    }
    //获取除了x0-x5以及s0以及外的寄存器的集合
    pub const fn init_unspecial_regs_without_s0() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0b_0000_0000_0000_0000_0000_0001_0001_1111,
            fregs_used: 0b_0000_0000_0000_0000_0000_0000_0000_0000,
        }
    }
    pub const fn init_unavailable() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0b_1111_1111_1111_1111_1111_1111_1111_1111,
            fregs_used: 0b_1111_1111_1111_1111_1111_1111_1111_1111,
        }
    }
}

impl RegUsedStat {
    ///绘制当前使用情况的编码标签
    pub fn draw_code_mark(&self) -> String {
        format!("{}_{}", self.iregs_used, self.fregs_used).to_string()
    }
    ///绘制表示当前寄存器使用情况的唯一对应符号标签
    pub fn draw_symbol_mark(&self) -> String {
        let mut out = "".to_string();
        for reg in 0..=63 {
            if self.is_available_reg(reg) {
                continue;
            }
            let reg = Reg::from_color(reg);
            let symbol = reg.to_string(false);
            out.push_str(&symbol);
        }
        out
    }
}

///RegUseStat求交集的实现
impl RegUsedStat {
    pub fn inter(&mut self, other: &RegUsedStat) {
        self.iregs_used &= other.iregs_used;
        self.fregs_used &= other.fregs_used;
    }
}

#[cfg(test)]
mod test_regusestat {
    use crate::backend::operand::Reg;

    use super::RegUsedStat;

    #[test]
    fn test_unavailable() {
        let reg_use_stat = RegUsedStat::init_unavailable();
        for reg in 0..=63 {
            assert!(!reg_use_stat.is_available_reg(reg));
        }
    }

    #[test]
    fn test_unspecial() {
        let reg_use_stat = RegUsedStat::init_unspecial_regs();
        for reg in Reg::get_all_specials_with_s0() {
            assert!(!reg_use_stat.is_available_reg(reg.get_color()));
        }
        for reg in Reg::get_all_not_specials() {
            assert!(reg_use_stat.is_available_reg(reg.get_color()));
        }
    }

    #[test]
    fn test_merge() {
        let mut reg = RegUsedStat::new();
        let mut reg2 = RegUsedStat::new();
        reg.use_reg(14);
        reg2.use_reg(55);
        assert!(reg.is_available_reg(55));
        reg.merge(&reg2);
        assert!(!reg.is_available_reg(55));
    }

    #[test]
    fn test_inter() {
        let m = RegUsedStat::init_for_i();
        let n = RegUsedStat::init_for_f();

        let mut gg = RegUsedStat::init_unavailable();
        gg.inter(&m);
        assert!(gg == m);
        let mut gg = RegUsedStat::init_unavailable();
        gg.inter(&n);
        assert!(gg == n);
        let mut k = m;
        for i in 0..=63 {
            let add_flag: bool = rand::random();
            if add_flag {
                k.use_reg(i);
            }
        }
        k.inter(&m);
        for i in 0..=63 {
            if k.is_available_reg(i) {
                assert!(m.is_available_reg(i))
            } else {
                assert!(!m.is_available_reg(i));
            }
        }
    }

    #[test]
    fn test_num() {
        // TODO
        let a = RegUsedStat::new();
        assert_eq!(a.num_available_fregs(), 32);
        assert_eq!(a.num_available_iregs(), 22); //保留t0-t2三个临时寄存器,sp,a0,tp,x0,gp, s0六个特殊寄存器,保留a0用作返回值
        assert_eq!(a.num_available_regs(crate::utility::ScalarType::Float), 32);
        assert_eq!(a.num_available_regs(crate::utility::ScalarType::Int), 22);
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
