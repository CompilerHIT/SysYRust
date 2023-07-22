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
    value: IImm,
}
#[derive(Clone)]
pub struct FGlobalVar {
    name: String,
    value: FImm,
}

//TODO: to implement const array
#[derive(Clone)]
pub enum GlobalVar {
    IGlobalVar(IGlobalVar),
    FGlobalVar(FGlobalVar),
    GlobalConstIntArray(IntArray),
    GlobalConstFloatArray(FloatArray),
}

impl GlobalVar {
    pub fn get_name(&self) -> &String {
        match self {
            GlobalVar::IGlobalVar(var) => &var.name,
            GlobalVar::FGlobalVar(var) => &var.name,
            GlobalVar::GlobalConstIntArray(var) => &var.name,
            GlobalVar::GlobalConstFloatArray(var) => &var.name,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Hash, Eq)]
pub struct StackSlot {
    is_fixed: bool,
    pos: i32,
    size: i32,
}

pub struct Context {
    stack_offset: i32,
    reg_info: HashMap<i32, i32>,
    epilogue: Option<Box<dyn FnMut()>>,
    prologue: Option<Box<dyn FnMut()>>,
    pub is_row: bool,
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
            is_row: false,
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
    pub fn init(name: String, value: i32) -> Self {
        Self {
            name,
            value: IImm::new(value),
        }
    }
    pub fn new(name: String) -> Self {
        Self::init(name, 0)
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_init(&self) -> IImm {
        self.value
    }
}

impl FGlobalVar {
    pub fn init(name: String, value: f32) -> Self {
        Self {
            name,
            value: FImm::new(value),
        }
    }
    pub fn new(name: String) -> Self {
        Self::init(name, 0.0)
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
        Self {
            pos,
            size,
            is_fixed: false,
        }
    }
    pub fn get_pos(&self) -> i32 {
        self.pos
    }
    pub fn get_size(&self) -> i32 {
        self.size
    }

    pub fn is_fixed(&self) -> bool {
        self.is_fixed
    }

    pub fn set_pos(&mut self, pos: i32) {
        self.pos = pos
    }
    pub fn set_size(&mut self, size: i32) {
        self.size = size
    }

    pub fn set_fix(&mut self) {
        self.is_fixed = true;
    }
}

pub struct Mapping {
    pub ir_block_map: HashMap<ObjPtr<BasicBlock>, ObjPtr<BB>>,
    pub block_ir_map: HashMap<ObjPtr<BB>, ObjPtr<BasicBlock>>,

    pub val_map: HashMap<ObjPtr<Inst>, Operand>,
    pub phis_to_block: HashMap<String, Vec<ObjPtr<LIRInst>>>,
}

impl Mapping {
    pub fn new() -> Self {
        Self {
            ir_block_map: HashMap::new(),
            block_ir_map: HashMap::new(),
            val_map: HashMap::new(),
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

#[derive(Clone)]
pub struct FloatArray {
    pub name: String,
    pub size: i32,
    pub init: bool,
    pub value: Vec<f32>,
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
        builder.print_array(&self.value, self.name.clone(), self.size)?;
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

impl FloatArray {
    pub fn new(name: String, size: i32, init: bool, value: Vec<f32>) -> Self {
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
    pub fn get_value(&self, index: i32) -> f32 {
        self.value[index as usize]
    }
    pub fn get_array(&self) -> &Vec<f32> {
        &self.value
    }
}

impl GenerateAsm for FloatArray {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        let mut builder = AsmBuilder::new(f);
        builder.print_farray(&self.value, self.name.clone(), self.size)?;
        Ok(())
    }
}

impl Hash for FloatArray {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for FloatArray {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for FloatArray {}

impl Operand {
    pub fn get_func_name(&self) -> String {
        match self {
            Operand::Addr(func_name) => func_name.to_owned(),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct Graph<T, R> {
    nodes: HashMap<T, Vec<R>>,
}

impl<T, R> Graph<T, R> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, node: T)
    where
        T: PartialEq + Eq + Hash,
    {
        self.nodes.entry(node).or_insert(Vec::new());
    }

    pub fn add_edge(&mut self, node: T, edge: R)
    where
        T: PartialEq + Eq + Hash,
    {
        self.nodes.entry(node).or_insert(Vec::new()).push(edge);
    }

    pub fn get_edges(&self, src: T) -> Option<&Vec<R>>
    where
        T: PartialEq + Eq + Hash + Copy,
    {
        self.nodes.get(&src)
    }

    pub fn get_mut_edges(&mut self, src: T) -> Option<&mut Vec<R>>
    where
        T: PartialEq + Eq + Hash + Copy,
    {
        self.nodes.get_mut(&src)
    }

    pub fn get_mut_nodes(&mut self) -> &mut HashMap<T, Vec<R>> {
        &mut self.nodes
    }

    pub fn get_nodes(&self) -> &HashMap<T, Vec<R>> {
        &self.nodes
    }

    pub fn delete_node(&mut self, node: T)
    where
        T: PartialEq + Eq + Hash + Copy,
    {
        self.nodes.remove(&node);
    }
}
