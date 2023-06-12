pub use std::collections::HashMap;
pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
use std::io::prelude::*;
pub use std::io::Result;

use crate::backend::block::BB;
use crate::backend::instrs::{LIRInst, Operand};
use crate::backend::operand::{FImm, IImm};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::Inst;
use crate::utility::ObjPtr;

use super::asm_builder::AsmBuilder;

#[derive(Clone)]
pub struct IGlobalVar {
    name: String,
    init: bool,
    value: IImm,
}
#[derive(Clone)]
pub struct FGlobalVar {
    name: String,
    init: bool,
    value: FImm,
}

//TODO: to implement const array
#[derive(Clone)]
pub enum GlobalVar {
    IGlobalVar(IGlobalVar),
    FGlobalVar(FGlobalVar),
    GlobalConstArray(IntArray)
}

impl GlobalVar {
    pub fn get_name(&self) -> &String {
        match self {
            GlobalVar::IGlobalVar(var) => &var.name,
            GlobalVar::FGlobalVar(var) => &var.name,
            GlobalVar::GlobalConstArray(var) => &var.name,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StackSlot {
    pos: i32,
    size: i32,
}

pub struct Context {
    stack_offset: i32,
    reg_info: HashMap<i32, i32>,
    epilogue: Option<Box<dyn FnMut()>>,
    prologue: Option<Box<dyn FnMut()>>,
}

#[derive(Clone)]
pub struct CurInstrInfo {
    block: Option<ObjPtr<BB>>,
    pub insts_it: Option<ObjPtr<LIRInst>>,
    pub pos: i32,
}

impl Context {
    pub fn new() -> Self {
        Self {
            stack_offset: 0,
            reg_info: HashMap::new(),
            epilogue: None,
            prologue: None,
        }
    }

    pub fn set_reg_map(&mut self, map: &HashMap<i32, i32>) {
        self.reg_info = map.clone();
    }

    pub fn get_reg_map(&self) -> &HashMap<i32, i32> {
        &self.reg_info
    }

    pub fn set_epilogue_event<F: FnMut() + 'static>(&mut self, callback: F) {
        self.epilogue = Some(Box::new(callback));
    }

    pub fn set_prologue_event<F: FnMut() + 'static>(&mut self, callback: F) {
        self.prologue = Some(Box::new(callback));
    }

    pub fn set_offset(&mut self, offset: i32) {
        self.stack_offset = offset;
    }

    pub fn get_offset(&self) -> i32 {
        self.stack_offset
    }

    pub fn call_epilogue_event(&mut self) {
        if let Some(ref mut callback) = self.epilogue {
            callback();
        }
    }

    pub fn call_prologue_event(&mut self) {
        if let Some(ref mut callback) = self.prologue {
            callback();
        }
    }
}

impl CurInstrInfo {
    pub fn new(pos: i32) -> Self {
        Self {
            pos,
            block: None,
            insts_it: None,
        }
    }

    pub fn band_block(&mut self, block: ObjPtr<BB>) {
        self.block = Some(block);
    }

    pub fn get_block(&self) -> Option<ObjPtr<BB>> {
        self.block
    }
}

impl IGlobalVar {
    pub fn init(name: String, value: i32, init: bool) -> Self {
        Self {
            name,
            value: IImm::new(value),
            init,
        }
    }
    pub fn new(name: String) -> Self {
        Self::init(name, 0, false)
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_init(&self) -> IImm {
        self.value
    }
}

impl FGlobalVar {
    pub fn init(name: String, value: f32, init: bool) -> Self {
        Self {
            name,
            value: FImm::new(value),
            init,
        }
    }
    pub fn new(name: String) -> Self {
        Self::init(name, 0.0, false)
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_init(&self) -> FImm {
        self.value
    }
}

pub trait GenerateAsm {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        writeln!(f, "unreachable")?;
        Ok(())
    }
}

impl PartialEq for CurInstrInfo {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}

impl Eq for CurInstrInfo {}

impl Hash for CurInstrInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
    }
}

impl StackSlot {
    pub fn new(pos: i32, size: i32) -> Self {
        Self { pos, size }
    }
    pub fn get_pos(&self) -> i32 {
        self.pos
    }
    pub fn get_size(&self) -> i32 {
        self.size
    }

    pub fn set_pos(&mut self, pos: i32) {
        self.pos = pos
    }
    pub fn set_size(&mut self, size: i32) {
        self.size = size
    }
}

pub struct Mapping {
    pub ir_block_map: HashMap<ObjPtr<BasicBlock>, ObjPtr<BB>>,
    pub block_ir_map: HashMap<ObjPtr<BB>, ObjPtr<BasicBlock>>,
    //TODO:for float
    pub array_slot_map: HashMap<ObjPtr<Inst>, i32>,

    pub val_map: HashMap<ObjPtr<Inst>, Operand>,
    pub block_branch: HashMap<String, ObjPtr<LIRInst>>,
    pub phis_to_block: HashMap<String, HashSet<ObjPtr<LIRInst>>>,
    // pub func_
}

impl Mapping {
    pub fn new() -> Self {
        Self {
            ir_block_map: HashMap::new(),
            block_ir_map: HashMap::new(),
            array_slot_map: HashMap::new(),
            val_map: HashMap::new(),
            block_branch: HashMap::new(),
            phis_to_block: HashMap::new(),
        }
    }
}


#[derive(Clone)]
pub struct IntArray {
    pub name: String,
    pub size: i32,
    pub init: bool,
    pub value: Vec<i32>,
}

impl IntArray {
    pub fn new(name: String, size: i32, init: bool, value: Vec<i32>) -> Self {
        Self {
            name,
            size,
            init,
            value,
        }
    }
    pub fn set_use(&mut self, used: bool) {
        self.init = used;
    }
    pub fn get_use(&self) -> bool {
        self.init
    }
    pub fn get_value(&self, index: i32) -> i32 {
        self.value[index as usize]
    }
    pub fn get_array(&self) -> &Vec<i32> {
        &self.value
    }
}

impl GenerateAsm for IntArray {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        let mut builder = AsmBuilder::new(f);
        builder.print_array(&self.value, self.name.clone());
        Ok(())
    }
}

impl Hash for IntArray {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for IntArray {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for IntArray {}
