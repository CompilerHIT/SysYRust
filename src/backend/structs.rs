use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use crate::backend::operand::*;
use crate::backend::instrs::*;

pub struct Map {
    reg_mapping: HashMap<usize, Reg>,
    // global_mapping: HashMap<String, GlobalVar>,
    // const_array_mapping: HashMap<String, ArrayConst>,
    // functions: Vec!<Function>
}

impl Map {

}

pub struct BB {
    label: String,

    pred: VecDeque<BB>,
    insts: Vec<Rc<Box<dyn Instrs>>>,

    in_edge: Vec<Rc<BB>>,
    out_edge: Vec<Rc<BB>>,

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