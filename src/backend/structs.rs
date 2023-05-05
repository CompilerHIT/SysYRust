use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::Result;

use crate::backend::instrs::Instrs;
use crate::backend::operand::Reg;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::const_int::{self, ConstInt};
use crate::ir::instruction::Instruction;
use crate::utility::Pointer;
use crate::utility::ScalarType;

use super::module::AsmModule;

#[derive(Clone)]
pub struct GlobalVar<V> {
    name: String,
    value: V,
    dtype: ScalarType,
}

pub struct StackObj {}

pub struct Context {
    stack_offset: i32,
    epilogue: Option<Box<dyn Fn()>>,
    prologue: Option<Box<dyn Fn()>>,
}

pub struct BB {
    label: String,
    called: bool,

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

// #[derive(Clone)]
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
            called: false,
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

    pub fn construct(&mut self, block: Pointer<BasicBlock>, func: Pointer<Func>, next_block: Pointer<BB>) {
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

    pub fn push_back(&mut self, inst: Pointer<Box<dyn Instrs>>) {
        self.insts.push(inst);
    }

    pub fn push_back_list(&mut self, inst: &mut Vec<Pointer<Box<dyn Instrs>>>) {
        self.insts.append(inst);
    }

    // fn clear_reg_info(&mut self) {
    //     self.live_def.clear();
    //     self.live_use.clear();
    //     self.live_in.clear();
    //     self.live_out.clear();
    // }
}

impl GenerateAsm for BB {
    fn generate(&self, context: Pointer<Context>,f: &mut File) -> String {
        if self.called {
            writeln!(f, "{}:", Self::label);
        }

        for inst in self.insts {
            inst.borrow().generate(context, f);
        }
    }
}

impl Func {
    pub fn new(name: &String) -> Self {
        Self {
            label: name.to_string(),
            blocks: Vec::new(),
            params: Vec::new(),
            entry: None,
            reg_def: Vec::new(),
            reg_use: Vec::new(),
            fregs: HashSet::new(),
        }
    }

    pub fn construct(&mut self, module: Pointer<AsmModule>, ir_func: Pointer<Function>) {

    }

    pub fn del_inst_reg(&mut self, cur_info: &CurInstrInfo, inst: Pointer<Box<dyn Instrs>>) {
        for reg in inst.borrow().get_reg_use() {
            self.reg_use[reg.get_id()].remove(cur_info);
        }
        for reg in inst.borrow().get_reg_def() {
            self.reg_def[reg.get_id()].remove(cur_info);
        }
    }

    pub fn add_inst_reg(&mut self, cur_info: &CurInstrInfo, inst: Pointer<Box<dyn Instrs>>) {
        for reg in inst.borrow().get_reg_use() {
            self.reg_use[reg.get_id()].insert(*cur_info);
        }
        for reg in inst.borrow().get_reg_def() {
            self.reg_def[reg.get_id()].insert(*cur_info);
        }
    }
}

impl GenerateAsm for Func {

}

impl Context {
    pub fn new() -> Self {
        Self {
            stack_offset: 0,
            epilogue: None,
            prologue: None,
        }
    }

    pub fn set_epilogue_event<F: Fn() + 'static>(&mut self, callback: F) {
        self.epilogue = Some(Box::new(callback));
    }
    
    pub fn set_prologue_event<F: Fn() + 'static>(&mut self, callback: F) {
        self.prologue = Some(Box::new(callback));
    }

    pub fn set_offset(&mut self, offset: i32) {
        self.stack_offset = offset;
    }

    pub fn get_offset(&self) -> i32 {
        self.stack_offset
    }

    pub fn call_epilogue_event(&self) {
        if let Some(ref callback) = self.epilogue {
            callback();
        }
    }

    pub fn call_prologue_event(&self) {
        if let Some(ref callback) = self.prologue {
            callback();
        }
    }
}

impl<V> GlobalVar<V> {
    pub fn new(name: String, value: V, dtype: ScalarType) -> Self {
        Self { name, value, dtype }
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

pub trait GenerateAsm {
    fn generate(&self, context: Pointer<Context>,f: &mut File) -> Result<()> {
        writeln!(f, "to realize")?;
        Ok(())
    }
}