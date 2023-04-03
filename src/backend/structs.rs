use std::collections::{HashMap, LinkedList, HashSet};

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

    pred: LinkedList<BB>,
    insts: Vec<Box<dyn Instrs>>,

    in_edge: Vec<Box<BB>>,
    out_edge: Vec<Box<BB>>,

    live_use: HashSet<Reg>,
    live_def: HashSet<Reg>,
    live_in: HashSet<Reg>,
    live_out: HashSet<Reg>,
}