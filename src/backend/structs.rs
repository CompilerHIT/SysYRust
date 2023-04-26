use std::collections::{HashSet, VecDeque};

use crate::utility::Pointer;
use crate::backend::operand::Reg;
use crate::backend::instrs::Instrs;
use crate::utility::ScalarType;


#[derive(Clone)]
pub struct GlobalVar<V> {
    name: String,
    value: V,     
    dtype: ScalarType,  
}

pub struct StackObj {
    
}

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

#[derive(Clone)]
struct CurInstrInfo {
    block: Pointer<BB>,
    insts_it: Vec<Pointer<Box<dyn Instrs>>>,
    // pos: usize,
}

#[derive(Clone)]
pub struct Func {
    label: String,
    blocks: Vec<Pointer<BB>>,
    // stack_obj: Vec<Pointer<StackObj>>,
    // caller_stack_obj: Vec<Pointer<StackObj>>,
    params: Vec<Pointer<Reg>>,
    entry: Pointer<BB>,

    reg_def: Vec<HashSet<CurInstrInfo>>,
    reg_use: Vec<HashSet<CurInstrInfo>>,
    fregs: HashSet<Reg>,
}

impl BB {
    fn clear_reg_info(&mut self) {
        self.live_def.clear();
        self.live_use.clear();
        self.live_in.clear();
        self.live_out.clear();
    }
}

impl<V> GlobalVar<V> {
    pub fn new(name: String, value: V, dtype: ScalarType) -> Self {
        Self {
            name,
            value,
            dtype,
        }
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_value(&self) -> &V {
        &self.value
    }
    pub fn get_dtype(&self) -> &ScalarType {
        &self.dtype
    }
}