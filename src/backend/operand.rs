use crate::utility::ScalarType;

pub const REG_COUNT: i32 = 32;
pub const ARG_REG_COUNT: i32 = 8;
pub const REG_SP: i32 = 2;
pub const IMM_12_Bs: i32 = 2047;
pub const IMM_20_Bs: i32 = 524287;
pub static mut I_REG_ID: i32 = 32;
pub static mut F_REG_ID: i32 = 32;

#[derive(Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub struct Reg {
    id: i32,
    r_type: ScalarType,
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

impl Reg {
    pub fn new(id: i32, r_type: ScalarType) -> Self {
        Self { id, r_type }
    }
    pub fn init(r_type: ScalarType) -> Self {
        match r_type {
            ScalarType::Int => unsafe {
                let mut id = I_REG_ID;
                I_REG_ID += 1;
                while id >= 0 && id <= 4 || id == 10 {
                    id = I_REG_ID;
                    I_REG_ID += 1;
                }
                Self { id, r_type }
            },
            ScalarType::Float => unsafe {
                let id = F_REG_ID;
                F_REG_ID += 1;
                Self { id, r_type }
            },
            _ => panic!("Wrong Type"),
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
                _ => {
                    println!("id: {}", self.id);
                    panic!("Invalid Physic Integer Register Id")
                },
            }
        } else {
            match self.id {
                0..=7 => format!("ft{}", self.id),
                8..=9 => format!("fs{}", self.id),
                10..=11 => format!("fa{}", self.id),
                12..=17 => format!("fs{}", self.id - 10),
                18..=27 => format!("fs{}", self.id - 16),
                28..=31 => format!("ft{}", self.id - 20),
                _ => panic!("Invalid Physic Float Register Id"),
            }
        }
    }

    // ra, t0, t1-2, a0-1, a2-7, t3-6
    // f0-7, f10-17, f28-31
    pub fn is_caller_save(&self) -> bool {
        match self.r_type {
            ScalarType::Int => {
                self.id == 1
                    || self.id == 3
                    || (self.id >= 5 && self.id <= 7)
                    || (self.id >= 10 && self.id <= 17)
                    || (self.id >= 28 && self.id <= 31)
            }
            ScalarType::Float => {
                (self.id >= 0 && self.id <= 7)
                    || (self.id >= 10 && self.id <= 17)
                    || (self.id >= 28 && self.id <= 31)
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
            ScalarType::Float => (self.id >= 8 && self.id <= 9) || (self.id >= 18 && self.id <= 27),
            _ => panic!("Wrong Type"),
        }
    }

    // sp for both callee and special
    // zero, sp, tp, ra, gp
    pub fn is_special(&self) -> bool {
        if self.r_type == ScalarType::Float {
            return false;
        }
        self.id >= 0 && self.id <= 7
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

    pub fn get_id(&self) -> i32 {
        self.id
    }
    pub fn get_type(&self) -> ScalarType {
        assert!(self.r_type != ScalarType::Void);
        self.r_type
    }
}
