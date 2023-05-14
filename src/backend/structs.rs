use std::borrow::BorrowMut;
use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

use crate::backend::{instrs::Instrs, operand::{Reg, IImm, FImm}, asm_builder::AsmBuilder, module::AsmModule};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{Instruction, const_int::ConstInt};
use crate::ir::function::Function;
use crate::ir::ir_type;
use crate::ir::instruction::return_inst::ReturnInst;
use crate::utility::Pointer;
use crate::utility::ScalarType;

use super::instrs::*;

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

pub struct StackSlot {
    pos: i32,
    size: i32,
}

pub struct Context {
    stack_offset: i32,
    epilogue: Option<Box<dyn FnMut()>>,
    prologue: Option<Box<dyn FnMut()>>,
}

pub struct BB {
    label: String,
    called: bool,

    insts: Vec<Pointer<Box<dyn Instrs>>>,

    in_edge: Vec<Pointer<BB>>,
    out_edge: Vec<Pointer<BB>>,

    live_use: HashSet<Reg>,
    live_def: HashSet<Reg>,
    live_in: HashSet<Reg>,
    live_out: HashSet<Reg>,
}

#[derive(Clone)]
pub struct CurInstrInfo {
    block: Option<Pointer<BB>>,
    insts_it: Vec<Pointer<Box<dyn Instrs>>>,
    id: usize,
}

// #[derive(Clone)]
pub struct Func {
    label: String,
    blocks: Vec<Pointer<BB>>,
    stack_addr: Vec<Pointer<StackSlot>>,
    caller_stack_addr: Vec<Pointer<StackSlot>>,
    params: Vec<Pointer<Reg>>,
    entry: Option<BB>,

    reg_def: Vec<HashSet<CurInstrInfo>>,
    reg_use: Vec<HashSet<CurInstrInfo>>,
    fregs: HashSet<Reg>,

    context: Option<Pointer<Context>>,
}

impl BB {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            called: false,
            insts: Vec::new(),
            in_edge: Vec::new(),
            out_edge: Vec::new(),
            live_use: HashSet::new(),
            live_def: HashSet::new(),
            live_in: HashSet::new(),
            live_out: HashSet::new(),
        }
    }

    pub fn construct(&mut self, block: Pointer<BasicBlock>, next_block: Pointer<BB>) {
        let ir_block_inst = block.borrow().get_dummy_head_inst();
        while let Some(inst) = ir_block_inst.borrow_mut().next() {
            let inst_borrow = inst.borrow();
            let dr_inst = inst_borrow.as_any();

            //TODO: wait for ir:
            // if let Some(inst1) = dr_inst.downcast_ref::<IR::Instructruction>() {

            // } else if let Some(inst2) = dr_inst.downcast_mut::<IR::Instructruction>() {

            // }
            // ...
            // else {
            //     panic!("fail to downcast inst");
            // }
            if let Some(inst) = dr_inst.downcast_ref::<ReturnInst>() {
                match inst.get_value_type() {
                    ir_type::IrType::Void => self.insts.push(Pointer::new(Box::new(Return::new(ScalarType::Void)))),
                    ir_type::IrType::Int => {
                        let src = inst.get_return_value();
                        let src_operand = self.resolvOperand(src);
                        self.insts.push(Pointer::new(Box::new(OpReg::new(
                            SingleOp::Mov, 
                            Reg::new(0, ScalarType::Int),
                            src_operand,
                        ))));
                        self.insts.push(Pointer::new(Box::new(Return::new(ScalarType::Int))));
                    },
                    ir_type::IrType::Float => {
                        //TODO:
                    },
                    _ => panic!("cannot reach, Return false")
                }
            }
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

    fn resolvOperand(&self, src: Pointer<Box<dyn Instruction>>) -> Operand {
        if let Some(iimm) = src.borrow().as_any().downcast_ref::<ConstInt>() {
            return Operand::IImm(IImm::new(iimm.get_bonding()));
        } else {
            //TODO:
            panic!("to realize more operand solution");
        }
    }

    // fn clear_reg_info(&mut self) {
    //     self.live_def.clear();
    //     self.live_use.clear();
    //     self.live_in.clear();
    //     self.live_out.clear();
    // }
}

impl GenerateAsm for BB {
    fn generate(&self, context: Pointer<Context>,f: &mut File) -> Result<()> {
        if self.called {
            writeln!(f, "{}:", self.label)?;
        }

        for inst in self.insts.iter() {
            inst.borrow().generate(context.clone(), f)?;
        }

        Ok(())
    }
}

impl Func {
    pub fn new(name: &String) -> Self {
        Self {
            label: name.to_string(),
            blocks: Vec::new(),
            stack_addr: Vec::new(),
            caller_stack_addr: Vec::new(),
            params: Vec::new(),
            entry: None,
            reg_def: Vec::new(),
            reg_use: Vec::new(),
            fregs: HashSet::new(),

            context: None,
        }
    }

    pub fn construct(&mut self, module: &AsmModule) {
        //FIXME: temporary
        let func_map = module.get_funcs();
        for (name, func_p) in func_map {
            self.label = name.clone();
            // more infos to add
            let show = format!(".entry_{name}");
            self.entry = Some(BB::new(&show));
            // 需要遍历block的接口
            // self.borrow_mut().blocks.push(Pointer::new(self.entry));
            
        }
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
            self.reg_use[reg.get_id()].insert((*cur_info).clone());
        }
        for reg in inst.borrow().get_reg_def() {
            self.reg_def[reg.get_id()].insert((*cur_info).clone());
        }
    }

    pub fn calc_live(&mut self) {
        let mut queue : VecDeque<(Pointer<BB>, Reg)> = VecDeque::new();
        for block in self.blocks.clone().iter() {
            block.borrow_mut().live_use.clear();
            block.borrow_mut().live_def.clear();
            for it in block.borrow().insts.iter().rev() {
                for reg in it.borrow().get_reg_def().into_iter() {
                    if reg.is_virtual() || reg.is_allocable() {
                        block.borrow_mut().live_use.remove(&reg);
                        block.borrow_mut().live_def.insert(reg);
                    }
                }
                for reg in it.borrow().get_reg_use().into_iter() {
                    if reg.is_virtual() || reg.is_allocable() {
                        block.borrow_mut().live_def.remove(&reg);
                        block.borrow_mut().live_use.insert(reg);
                    }
                }
            }
            for reg in block.borrow().live_use.iter() {
                queue.push_back((block.clone(), reg.clone()));
            }
            block.borrow_mut().live_in = block.borrow().live_use.clone();
            block.borrow_mut().live_out.clear();
        }
        while let Some(value) = queue.pop_front() {
            let (block, reg) = value;
            for pred in block.borrow().in_edge.iter() {
                if pred.borrow_mut().live_out.insert(reg) {
                    if pred.borrow_mut().live_def.take(&reg) == None && pred.borrow_mut().live_in.insert(reg) {
                        queue.push_back((pred.clone(), reg.clone()));
                    }
                }
            }
        }
    }

    pub fn allocate_reg(&mut self, f: &'static mut File) {
        //FIXME: 暂时使用固定的寄存器ra与s0，即r1, r8
        //FIXME:暂时只考虑int型
        let reg_int = vec![Reg::new(1, ScalarType::Int), Reg::new(8, ScalarType::Int)];

        let mut stack_size = 0;
        for it in self.stack_addr.iter().rev() {
            it.borrow_mut().set_pos(stack_size);
            stack_size += it.borrow().get_size();
        }

        let mut reg_int_res = Vec::from(reg_int);
        let mut reg_int_res_cl = reg_int_res.clone();
        let reg_int_size = reg_int_res.len();
        
        //TODO:栈对齐 - 8字节

        let mut offset = stack_size;
        let mut f1 = f.try_clone().unwrap();
        if let Some(contxt) = &self.context {
            contxt.borrow_mut().set_prologue_event(move||{
                let mut builder = AsmBuilder::new(f, "");
                // addi sp -stack_size
                builder.addi("sp", "sp", -offset);
                for src in reg_int_res.iter() {
                    offset -= 8;
                    builder.sd(&src.to_string(), "sp", offset, false);
                }
            });
            let mut offset = stack_size;
            contxt.borrow_mut().set_epilogue_event(move||{
                let mut builder = AsmBuilder::new(&mut f1, "");
                for src in reg_int_res_cl.iter() {
                    offset -= 8;
                    builder.ld("sp", &src.to_string(), offset, false);
                }
                builder.addi("sp", "sp", offset);
            });
        }
        

        //TODO: for caller
        // let mut pos = stack_size + reg_int_size as i32 * 8;
        // for caller in self.caller_stack_addr.iter() {
        //     caller.borrow_mut().set_pos(pos);
        //     pos += caller.borrow().get_size();
        // }
        
    }
}

impl GenerateAsm for Func {
    fn generate(&self, _: Pointer<Context>, f: &mut File) -> Result<()> {
        AsmBuilder::new(f, "").show_func(&self.label);
        if let Some(contxt) = &self.context {
            contxt.borrow_mut().call_prologue_event();
            for block in self.blocks.iter() {
                block.borrow().generate(contxt.clone(), f)?;
            }
        }
        Ok(())
    }
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
    pub fn new(id: usize) -> Self {
        Self {
           id,
           block: None,
           insts_it: Vec::new(),
        }
    }

    pub fn band_block(&mut self, block: Pointer<BB>) {
        self.block = Some(block.clone());
    }

    pub fn get_block(&self) -> Pointer<BB> {
        self.block.clone().unwrap()
    }

    pub fn add_inst(&mut self, inst: Pointer<Box<dyn Instrs>>) {
        self.insts_it.push(inst.clone());
    }

    pub fn add_insts(&mut self, insts: Vec<Pointer<Box<dyn Instrs>>>) {
        self.insts_it.append(&mut insts.clone());
    }
}

impl IGlobalVar {
    pub fn new(name: String) -> Self {
        Self { name, value: IImm::new(0), init: false }
    }
    pub fn init(name: String, value: i32) -> Self {
        Self { name, value: IImm::new(value), init: true }
    }
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_init(&self) -> IImm {
        self.value
    }
}

pub trait GenerateAsm {
    fn generate(&self, _: Pointer<Context>, f: &mut File) -> Result<()> {
        writeln!(f, "to realize")?;
        Ok(())
    }
}

impl PartialEq for CurInstrInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for CurInstrInfo {}

impl Hash for CurInstrInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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

    fn set_pos(&mut self, pos: i32) {
        self.pos = pos
    } 
    fn set_size(&mut self, size: i32) {
        self.size = size
    }
}   

impl PartialEq for BB {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Eq for BB {}

impl Hash for BB {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}