const REG_COUNT: i8 = 32;

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Reg {
    id: usize,
    r_type: RegType
}

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
enum RegType {
    Int,
    Float
}

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct IImm {
    data: isize,
}

#[derive(Clone, Copy, PartialEq)]
pub struct FImm {
    data: f32
}

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub struct Addr {
    label: &'static str,
}

trait ToString {
    fn to_string(&self) -> String 
    where Self: std::fmt::Display {
        format!("{}", self)
    }
    fn to_hex_string(&self) -> String
    where Self: std::fmt::LowerHex {
        format!("{:x}", self)
    }
}

impl ToString for IImm {}
impl ToString for FImm {}
impl ToString for Addr {
    fn to_hex_string(&self) -> String {
        panic!("Wrong Call")
    }
}


impl Reg {
    fn new(id: usize, r_type: RegType) -> Self {
        Self {
            id,
            r_type
        }
    }
    fn to_string(&self) -> String {
        if self.r_type == RegType::Int {
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
    fn is_caller_save(&self) -> bool {
        match self.r_type {
            RegType::Int => self.id == 1
                                || (self.id >= 5 && self.id <= 7)
                                || (self.id >= 10 && self.id <= 17)
                                || (self.id >= 28 && self.id <= 31),
            RegType::Float => (self.id >= 0 && self.id <= 7) 
                                || (self.id >= 10 && self.id <= 17)
                                || (self.id >= 28 && self.id <= 31),
            _ => panic!("Wrong Type")
        }
        
    }

    // sp, s0(fp), s1, s2-11    
    // f8-9, f18-27
    fn is_callee_save(&self) -> bool {
        match self.r_type {
            RegType::Int => self.id == 2 || self.id == 8 || self.id == 9 
                                || (self.id >= 18 && self.id <= 27),
            RegType::Float => (self.id >= 8 && self.id <= 9) || (self.id >= 18 && self.id <= 27),
            _ => panic!("Wrong Type")
        }
    }

    // sp for both callee and special
    // zero, sp, gp, tp
    fn is_special(&self) -> bool {
        self.id == 0 || (self.id >= 2 && self.id <= 4)
    }

    // if virtual reg
    fn is_virtual(&self) -> bool {
        self.id > 31
    }

    // if physic reg
    fn is_physic(&self) -> bool {
        self.id <= 31
    }

    // if mistake
    fn is_mistake(&self) -> bool {
        self.id < 0
    }
}
