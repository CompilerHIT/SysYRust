use std::collections::{HashSet, VecDeque};

use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::Instruction;
use crate::ir::instruction::const_int::{self, ConstInt};
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
    pub fn new(label: &String) -> Self {
        Self {
            label: label.to_string(),
            pred: VecDeque::new(),
            insts: Vec::new(),
            in_edge: Vec::new(),
            out_edge: Vec::new(),
            live_use: HashSet::new(),
            live_def: HashSet::new(),
            live_in: HashSet::new(),
            live_out: HashSet::new(),
        }
    }

    pub fn construct(block: Pointer<BasicBlock>, func: Pointer<Func>, next_block: Pointer<BB>) {
        let mut ir_block_inst = block.borrow().get_dummy_head_inst();
        while let Some(inst) = ir_block_inst.borrow_mut().next() {
            let dr_inst = inst.borrow().as_any();

            //TODO: wait for ir:
            // if let Some(inst1) = dr_inst.downcast_ref::<IR::Instructruction>() {

            // } else if let Some(inst2) = dr_inst.downcast_mut::<IR::Instructruction>() {

            // } 
            // ...
            // else {
            //     panic!("fail to downcast inst");
            // }
            
            if Pointer::point_eq(&inst, &block.borrow().get_tail_inst().unwrap()) {
                break;
            }
        }
    }

    pub fn push_back(&mut self, inst: Pointer<Box<dyn Instruction>>) {
        //FIXME: push 'lir inst' back
        // match self.get_tail_inst() {
        //     Some(tail) => {
        //         tail.borrow_mut().insert_after(inst);
        //     }
        //     None => {
        //         let mut head = self.inst_head.borrow_mut();
        //         let mut inst_b = inst.borrow_mut();
        //         head.set_next(inst.clone());
        //         head.set_prev(inst.clone());

        //         inst_b.set_next(self.inst_head.clone());
        //         inst_b.set_prev(self.inst_head.clone());
        //     }
        // }
    }

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