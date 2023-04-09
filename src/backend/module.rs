use std::collections::HashMap;
use crate::backend::operand::Reg;
use crate::backend::structs::GlobalVar;

pub struct Map {
    reg_mapping: HashMap<usize, Reg>,

    //TODO: add global mapping: complete init pointer to make sure empty or not
    global_mapping: HashMap<String, GlobalVar>,

    // const_array_mapping: HashMap<String, ArrayConst>,
    // functions: Vec!<Function>
}

impl Map {

}