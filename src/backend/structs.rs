use std::collections::LinkedList;
pub use std::collections::{HashSet, VecDeque};
pub use std::collections::HashMap;
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
pub use std::fs::write;

use crate::utility::ObjPtr;
use crate::backend::operand::{IImm, FImm};
use crate::backend::instrs::LIRInst;
use crate::backend::block::BB;
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::Inst;

use super::asm_builder::AsmBuilder;
use super::instrs::Operand;


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
    value:FImm,
}

//TODO: to implement const array
#[derive(Clone)]
pub enum GlobalVar {
    IGlobalVar(IGlobalVar),
    FGlobalVar(FGlobalVar)
}

#[derive(Clone, Copy, PartialEq)]
pub struct StackSlot {
    pos: i32,
    size: i32,
}

pub struct Context {
    stack_offset: i32,
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
            epilogue: None,
            prologue: None,
        }
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
        Self { name, value: IImm::new(value), init }
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
        Self { name, value: FImm::new(value), init }
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
    fn generate(&self, _: ObjPtr<Context>){
        println!("unreachable");
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
        Self{ pos, size }
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
    pub stack_slot_set: LinkedList<StackSlot>,
    //TODO:for float
    pub int_array_map: HashSet<IntArray>,
    pub array_slot_map: HashMap<ObjPtr<Inst>, i32>,

    pub val_map: HashMap<ObjPtr<Inst>, Operand>,

    pub func_imm_map: HashMap<Operand, ObjPtr<LIRInst>>
    // pub func_
}

impl Mapping {
    pub fn new() -> Self {
        Self {
            ir_block_map: HashMap::new(),
            block_ir_map: HashMap::new(),
            stack_slot_set: LinkedList::new(),
            int_array_map: HashSet::new(),
            array_slot_map: HashMap::new(),
            val_map: HashMap::new(),
            func_imm_map: HashMap::new(),
        }
    }
}

impl Hash for ObjPtr<Inst> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.as_ref(), state);
    }
}

pub struct IntArray {
    pub name: String,
    pub size: i32,
    pub init: bool,
    pub value: Vec<i32>,
}

impl IntArray {
    pub fn new(name: String,size: i32, init: bool, value: Vec<i32>) -> Self {
        Self { name, size, init, value }
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
    pub fn get_array(&self)  -> &Vec<i32> {
        &self.value
    }
}

//TODO: generate array
impl GenerateAsm for IntArray {
    fn generate(&self, _: ObjPtr<Context>) {
        let mut builder = AsmBuilder::new();
        builder.print_array(&self.value, self.name.clone());
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