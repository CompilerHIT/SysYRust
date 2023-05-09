use crate::utility::ScalarType;

pub const REG_COUNT: usize = 32;
pub const ARG_REG_COUNT: usize = 8;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Reg {
    id: usize,
    r_type: ScalarType,
}

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct IImm {
    data: isize,
}

#[derive(Clone, Copy, PartialEq)]
pub struct FImm {
    data: f64
}

pub trait ImmBs {
    fn is_imm_20bs(&self) -> bool;
    fn is_imm_12bs(&self) -> bool;
}

impl ImmBs for IImm {
    fn is_imm_20bs(&self) -> bool {
        self.data >= -524288 && self.data <= 524287
    }
    fn is_imm_12bs(&self) -> bool {
        self.data >= -2048 && self.data <= 2047
    }
}

impl ImmBs for FImm {
    fn is_imm_20bs(&self) -> bool {
        self.data >= -524288.0 && self.data <= 524287.0
    }
    fn is_imm_12bs(&self) -> bool {
        self.data >= -2048.0 && self.data <= 2047.0
    }
}

#[derive(Clone, PartialEq, Hash, Eq)]
pub struct Addr {
    label: &'static str,
}

pub trait ToString {
    fn to_string(&self) -> String;
    fn to_hex_string(&self) -> String;
}

impl ToString for IImm {
    fn to_string(&self) -> String {
        self.data.to_string()
    }
    fn to_hex_string(&self) -> String {
        format!("{:x}", self.data)
    }
}
impl ToString for FImm {
    fn to_string(&self) -> String {
        self.data.to_string()
    }
    fn to_hex_string(&self) -> String {
        let bits = self.data.to_bits();
        format!("0x{:x}", bits)
    }
}
impl ToString for Addr {
    fn to_string(&self) -> String {
        self.label.to_string()
    }
    fn to_hex_string(&self) -> String {
        panic!("Wrong Call")
    }
}


impl Reg {
    pub fn new(id: usize, r_type: ScalarType) -> Self {
        Self {
            id,
            r_type
        }
    }
    pub fn to_string(&self) -> String {
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
                _ => panic!("Invalid Physic Integer Register Id")
            }
        } else {
            match self.id {
                0..=7 => format!("ft{}", self.id),
                8..=9 => format!("fs{}", self.id),
                10..=11 => format!("fa{}", self.id),
                12..=17 => format!("fs{}", self.id - 10),
                18..=27 => format!("fs{}", self.id - 16),
                28..=31 => format!("ft{}", self.id - 20),
                _ => panic!("Invalid Physic Float Register Id")
            }
        }
    }

    // ra, t0, t1-2, a0-1, a2-7, t3-6
    // f0-7, f10-17, f28-31
    pub fn is_caller_save(&self) -> bool {
        match self.r_type {
            ScalarType::Int => self.id == 1
                                || (self.id >= 5 && self.id <= 7)
                                || (self.id >= 10 && self.id <= 17)
                                || (self.id >= 28 && self.id <= 31),
            ScalarType::Float => (self.id >= 0 && self.id <= 7) 
                                || (self.id >= 10 && self.id <= 17)
                                || (self.id >= 28 && self.id <= 31),
            _ => panic!("Wrong Type")
        }
        
    }

    // sp, s0(fp), s1, s2-11    
    // f8-9, f18-27
    pub fn is_callee_save(&self) -> bool {
        match self.r_type {
            ScalarType::Int => self.id == 2 || self.id == 8 || self.id == 9 
                                || (self.id >= 18 && self.id <= 27),
            ScalarType::Float => (self.id >= 8 && self.id <= 9) || (self.id >= 18 && self.id <= 27),
            _ => panic!("Wrong Type")
        }
    }

    // sp for both callee and special
    // zero, sp, gp, tp
    pub fn is_special(&self) -> bool {
        self.id == 0 || (self.id >= 2 && self.id <= 4)
    }

    pub fn is_allocable(&self) -> bool {
        !self.is_special()
    }

    // if virtual reg
    pub fn is_virtual(&self) -> bool {
        self.id > 31
    }

    // if physic reg
    pub fn is_physic(&self) -> bool {
        self.id <= 31
    }

    // if mistake
    pub fn is_mistake(&self) -> bool {
        self.id < 0
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}
