use std::collections::{HashSet, VecDeque};

use crate::utility::Pointer;
use crate::backend::operand::*;
use crate::backend::instrs::*;
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

pub struct GlobalVar {
    name: String,
    size: i32,   // only available when is_int
    // void *init, // when !is_int, must not empty. Q: how to imply void* type
    is_const: bool,
    dtype: ScalarType,
}