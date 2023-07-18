use crate::log;
use crate::utility::ScalarType;
use std::{
    collections::HashSet,
    fmt::{format, Display},
};

use super::block::FLOAT_BASE;
pub const REG_COUNT: i32 = 32;
pub const ARG_REG_COUNT: i32 = 8;
pub const REG_SP: i32 = 2;
pub const IMM_12_Bs: i32 = 2047;
pub const IMM_20_Bs: i32 = 524287;
pub static mut REG_ID: i32 = 64;

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub struct Reg {
    id: i32,
    r_type: ScalarType,
}

impl Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = self.get_id();
        match self.get_type() {
            ScalarType::Float => write!(f, "f{}", id),
            ScalarType::Int => write!(f, "i{}", id),
            ScalarType::Void => write!(f, "void{}", id),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub struct IImm {
    data: i32,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FImm {
    data: f32,
}

impl IImm {
    pub fn new(data: i32) -> Self {
        Self { data }
    }
    pub fn get_data(&self) -> i32 {
        self.data
    }
}

impl FImm {
    pub fn new(data: f32) -> Self {
        Self { data }
    }
    pub fn get_data(&self) -> f32 {
        self.data
    }
}

pub fn is_imm_20bs(imm: i32) -> bool {
    imm >= -IMM_20_Bs - 1 && imm <= IMM_20_Bs
}
pub fn is_imm_12bs(imm: i32) -> bool {
    imm >= -IMM_12_Bs - 1 && imm <= IMM_12_Bs
}

pub trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for IImm {
    fn to_string(&self) -> String {
        self.data.to_string()
    }
}

impl ToString for FImm {
    fn to_string(&self) -> String {
        unsafe { format!("{}", *(&self.data as *const f32 as *const i32)) }
    }
}

impl Reg {
    pub fn new(id: i32, r_type: ScalarType) -> Self {
        debug_assert!(
            || -> bool {
                if id > 63 {
                    true
                } else if (id >= 0 && id < 32) && r_type == ScalarType::Int {
                    true
                } else if (id <= 63 && id >= 32) && r_type == ScalarType::Float {
                    true
                } else if id < 0 {
                    unreachable!();
                } else {
                    false
                }
            }(),
            "{}{:?}",
            id,
            r_type
        );

        Self { id, r_type }
    }

    pub fn init(r_type: ScalarType) -> Self {
        unsafe {
            let id = REG_ID;
            REG_ID += 1;
            Self { id, r_type }
        }
    }
    pub fn to_string(&self, is_row: bool) -> String {
        if is_row {
            return format!("x{}", self.id);
        }
        if self.r_type == ScalarType::Int {
            match self.id {
                0 => String::from("zero"),
                1 => String::from("ra"),
                2 => String::from("sp"),
                3 => String::from("gp"),
                4 => String::from("tp"),
                5..=7 => format!("t{}", self.id - 5),
                8..=9 => format!("s{}", self.id - 8),
                10..=17 => format!("a{}", self.id - 10),
                18..=27 => format!("s{}", self.id - 16),
                28..=31 => format!("t{}", self.id - 25),
                _ => {
                    // 使用虚拟寄存器
                    format!("v{}", self.id)
                    // log!("id: {}", self.id);
                    // panic!("Invalid Physic Integer Register Id")
                }
            }
        } else {
            let id = self.id - FLOAT_BASE;
            assert!(id >= 0);
            match id {
                0..=7 => format!("ft{}", id),
                8..=9 => format!("fs{}", id - 8),
                10..=17 => format!("fa{}", id - 10),
                18..=27 => format!("fs{}", id - 16),
                28..=31 => format!("ft{}", id - 20),
                _ => format!("fv{}", self.id),
            }
        }
    }

    pub fn to_row(&self) -> String {
        format!("r{}", self.id)
    }

    // ra, t0, t1-2, a0-1, a2-7, t3-6
    // f0-7, f10-17, f28-31
    pub fn is_caller_save(&self) -> bool {
        match self.r_type {
            ScalarType::Int => {
                self.id == 1
                    || (self.id >= 5 && self.id <= 7)
                    || (self.id >= 10 && self.id <= 17)
                    || (self.id >= 28 && self.id <= 31)
            }
            ScalarType::Float => {
                let id = self.id - FLOAT_BASE;
                assert!(id >= 0);
                (id >= 0 && id <= 7) || (id >= 10 && id <= 17) || (id >= 28 && id <= 31)
            }
            _ => panic!("Wrong Type"),
        }
    }

    // sp, s0(fp), s1, s2-11
    // f8-9, f18-27
    pub fn is_callee_save(&self) -> bool {
        match self.r_type {
            ScalarType::Int => {
                self.id == 2 || self.id == 8 || self.id == 9 || (self.id >= 18 && self.id <= 27)
            }
            ScalarType::Float => {
                let id = self.id - FLOAT_BASE;
                assert!(id >= 0);
                (id >= 8 && id <= 9) || (id >= 18 && id <= 27)
            }
            _ => panic!("Wrong Type"),
        }
    }

    // sp for both callee and special
    // zero, sp, tp, ra, gp, t0-2, s0
    pub fn is_special(&self) -> bool {
        if self.r_type == ScalarType::Float {
            return false;
        }
        self.id >= 0 && self.id <= 8
    }

    pub fn is_allocable(&self) -> bool {
        !self.is_special() && self.is_physic()
    }

    // if virtual reg
    pub fn is_virtual(&self) -> bool {
        self.id > 63
    }

    // if physic reg
    pub fn is_physic(&self) -> bool {
        self.id <= 63
    }

    // if mistake
    pub fn is_mistake(&self) -> bool {
        self.id < 0
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_type(&self) -> ScalarType {
        assert!(self.r_type != ScalarType::Void);
        self.r_type
    }
}

impl Reg {
    pub fn get_color(&self) -> i32 {
        if !self.is_physic() {
            panic!("unreachab{self}");
        }
        match self.get_type() {
            ScalarType::Float => self.get_id(),
            ScalarType::Int => self.get_id(),
            _ => panic!("gg"),
        }
    }
    // 用来给bitmap唯一标识用
    pub fn bit_code(&self) -> i32 {
        (self.get_id() << 1)
            | match self.get_type() {
                ScalarType::Float => 1,
                ScalarType::Int => 0,
                _ => panic!("unreachable"),
            }
    }
    pub fn from_bit_code(bit_code: i32) -> Reg {
        if bit_code % 2 == 1 {
            Reg::new(bit_code >> 1, ScalarType::Float)
        } else {
            Reg::new(bit_code >> 1, ScalarType::Int)
        }
    }
}

///获取reg相关信息
impl Reg {
    ///获取所有caller saved寄存器
    pub fn get_all_callers_saved() -> HashSet<Reg> {
        let mut callers_saved = HashSet::new();
        for id in 0..=31 {
            let reg = Reg::new(id, ScalarType::Int);
            if reg.is_caller_save() {
                callers_saved.insert(reg);
            }
            let reg = Reg::new(id + FLOAT_BASE, ScalarType::Float);
            if reg.is_caller_save() {
                callers_saved.insert(reg);
            }
        }
        callers_saved
    }
    ///获取所有callee saved寄存器
    pub fn get_all_callees_saved() -> HashSet<Reg> {
        let mut callees_saved = HashSet::new();
        for id in 0..=31 {
            let reg = Reg::new(id, ScalarType::Int);
            if reg.is_callee_save() {
                callees_saved.insert(reg);
            }
            let reg = Reg::new(id + FLOAT_BASE, ScalarType::Float);
            if reg.is_callee_save() {
                callees_saved.insert(reg);
            }
        }

        callees_saved
    }
}

///从颜色编号到寄存器
impl Reg {
    pub fn from_color(color: i32) -> Reg {
        debug_assert!(color >= 0 && color <= 63);
        if color < 32 {
            Reg::new(color, ScalarType::Int)
        } else {
            Reg::new(color, ScalarType::Float)
        }
    }
}
