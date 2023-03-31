#[derive(Clone, Copy, PartialEq, Hash)]
pub struct IReg {
    id: usize,
}

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct IImm {
    data: isize,
}

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct Addr {
    label: &'static str,
}

impl IImm {
    fn to_hex_string(&self) -> String {
        format!("{:x}", self.data)
    }
    fn to_string(&self) -> String {
        self.data.to_string()
    }
}

impl IReg {
    fn new(id: usize) -> IReg {
        IReg { id }
    }

    fn to_string(&self) -> String {
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
            _ => panic!("Invalid Physic Register Id"),
        }
    }

    // ra, t0, t1-2, a0-1, a2-7, t3-6
    fn is_caller_save(&self) -> bool {
        self.id == 1
            || (self.id >= 5 && self.id <= 7)
            || (self.id >= 10 && self.id <= 17)
            || (self.id >= 28 && self.id <= 31)
    }

    // sp, s0(fp), s1, s2-11
    fn is_callee_save(&self) -> bool {
        self.id == 2 || self.id == 8 || self.id == 9 || (self.id >= 18 && self.id <= 27)
    }

    // sp for both callee and special
    // zero, sp, gp, tp
    fn is_special(&self) -> bool {
        self.id == 0 || (self.id >= 2 && self.id <= 4)
    }
}
