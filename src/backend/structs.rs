use std::collections::{HashSet, VecDeque};

use crate::utility::Pointer;
use crate::backend::operand::Reg;
use crate::backend::instrs::Instrs;
use crate::utility::ScalarType;

pub struct BB {
    label: String,

    pred: VecDeque<BB>,
    insts: Vec<Pointer<Box<dyn Instrs>>>,

    in_edge: Vec<Pointer<BB>>,
    out_edge: Vec<Pointer<BB>>,

    live_use: HashSet<Reg>,
    live_def: HashSet<Reg>,
    live_in: HashSet<Reg>,
    live_out: HashSet<Reg>,
}

impl BB {
    fn clear_reg_info(&mut self) {
        self.live_def.clear();
        self.live_use.clear();
        self.live_in.clear();
        self.live_out.clear();
    }
}

#[derive(Clone)]
pub struct GlobalVar {
    pub name: String,
    pub size: i32,   // only available when is_int
    // void *init, // when !is_int, must not empty. Q: how to imply void* type
    pub is_const: bool,
    pub dtype: ScalarType,
}

#[derive(Clone)]
pub struct Func {

}