use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::function::Function;
use crate::ir::ir_type;
use crate::backend::instrs::{LIRInst, InstrsType, SingleOp};
use crate::backend::operand::{Reg, IImm, FImm};
use crate::backend::asm_builder::AsmBuilder;
use crate::backend::module::AsmModule;

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

#[derive(Clone, PartialEq)]
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

    insts: Vec<ObjPtr<LIRInst>>,

    in_edge: Vec<ObjPtr<BB>>,
    out_edge: Vec<ObjPtr<BB>>,

    live_use: HashSet<Reg>,
    live_def: HashSet<Reg>,
    live_in: HashSet<Reg>,
    live_out: HashSet<Reg>,

    insts_mpool: ObjPool<LIRInst>,
}

#[derive(Clone)]
pub struct CurInstrInfo {
    block: Option<ObjPtr<BB>>,
    insts_it: Vec<ObjPtr<LIRInst>>,
    reg_id: i32,
}

// #[derive(Clone)]
pub struct Func {
    label: String,
    blocks: Vec<ObjPtr<BB>>,
    stack_addr: Vec<ObjPtr<StackSlot>>,
    caller_stack_addr: Vec<ObjPtr<StackSlot>>,
    params: Vec<ObjPtr<Reg>>,
    entry: Option<BB>,

    reg_def: Vec<HashSet<CurInstrInfo>>,
    reg_use: Vec<HashSet<CurInstrInfo>>,
    fregs: HashSet<Reg>,

    context: Option<ObjPtr<Context>>,

    blocks_mpool: ObjPool<BB>,
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
            insts_mpool: ObjPool::new(),
        }
    }

    // 删除BB时清空当前块维护的内存池
    pub fn del(&self) {
        self.insts_mpool.free_all()
    }

    pub fn construct(&mut self, block: ObjPtr<BasicBlock>, next_block: ObjPtr<BB>) {
        let mut ir_block_inst = block.as_ref().get_head_inst();
        loop {
            let inst_ref = ir_block_inst.as_ref();
            // translate ir to lir, use match
            match inst_ref.get_kind() {
                InstKind::Return => {
                    match inst_ref.get_ir_type() {
                        ir_type::IrType::Void => self.insts.push(
                            self.insts_mpool.put(
                                LIRInst::new(InstrsType::Ret(ScalarType::Void), vec![])
                            )
                        ),
                        ir_type::IrType::Int => {
                            let src = inst_ref.get_return_value();
                            let src_operand = self.resolvOperand(src);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::OpReg(SingleOp::Mov), vec![src_operand])
                                )
                            );
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::Ret(ScalarType::Int), vec![])
                                )
                            );
                        },
                        ir_type::IrType::Float => {
                            //TODO:
                        },
                        _ => panic!("cannot reach, Return false")
                    }
                }
                _ => {
                // TODO: ir translation.
                }
            }
            if let Some(ir_block_inst) = ir_block_inst.as_ref().get_next() {} 
            else {
                break;
            }
        }
    }

    pub fn push_back(&mut self, inst: ObjPtr<LIRInst>) {
        self.insts.push(inst);
    }

    pub fn push_back_list(&mut self, inst: &mut Vec<ObjPtr<LIRInst>>) {
        self.insts.append(inst);
    }

    fn resolvOperand(&self, src: ObjPtr<Inst>) -> Operand {
        //TODO: ObjPtr match
        match src.as_ref().get_kind() {
            //TODO: resolve different kind of operand
            InstKind::ConstInt(iimm) => resolveConstInt(iimm),
            InstKind::ConstFloat(fimm) => resolveConstFloat(fimm),
            InstKind::GlobalConstInt(_) => resolveGlobalVar(src),
            InstKind::GlobalConstFloat(_) => resolveGlobalVar(src),
            _ => {}
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
    fn generate(&self, context: ObjPtr<Context>,f: &mut File) -> Result<()> {
        if self.called {
            writeln!(f, "{}:", self.label)?;
        }

        for inst in self.insts.iter() {
            inst.as_ref().generate(context.clone(), f)?;
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

            blocks_mpool: ObjPool::new(),
        }
    }

    // 或许不会删除函数？
    pub fn del(&self) {
        self.blocks_mpool.free_all()
    }

    pub fn construct(&mut self, module: &AsmModule) {
        //FIXME: temporary
        let func_map = module.get_funcs();
        for (name, func_p) in func_map {
            self.label = name.to_string();
            // more infos to add
            let show = format!(".entry_{name}");
            self.entry = Some(BB::new(&show));
            // 需要遍历block的接口
            // self.borrow_mut().blocks.push(Pointer::new(self.entry));
            
        }
    }

    // 移除指定id的寄存器的使用信息
    pub fn del_inst_reg(&mut self, cur_info: &CurInstrInfo, inst: ObjPtr<LIRInst>) {
        for reg in inst.as_ref().get_reg_use() {
            self.reg_use[reg.get_id() as usize].remove(cur_info);
        }
        for reg in inst.as_ref().get_reg_def() {
            self.reg_def[reg.get_id() as usize].remove(cur_info);
        }
    }

    // 添加指定id的寄存器的使用信息
    pub fn add_inst_reg(&mut self, cur_info: &CurInstrInfo, inst: ObjPtr<LIRInst>) {
        for reg in inst.as_ref().get_reg_use() {
            self.reg_use[reg.get_id() as usize].insert(cur_info.clone());
        }
        for reg in inst.as_ref().get_reg_def() {
            self.reg_def[reg.get_id() as usize].insert(cur_info.clone());
        }
    }

    pub fn calc_live(&mut self) {
        let mut queue : VecDeque<(ObjPtr<BB>, Reg)> = VecDeque::new();
        for block in self.blocks.clone().iter() {
            block.as_mut().live_use.clear();
            block.as_mut().live_def.clear();
            for it in block.as_ref().insts.iter().rev() {
                for reg in it.as_ref().get_reg_def().into_iter() {
                    if reg.is_virtual() || reg.is_allocable() {
                        block.as_mut().live_use.remove(&reg);
                        block.as_mut().live_def.insert(reg);
                    }
                }
                for reg in it.as_ref().get_reg_use().into_iter() {
                    if reg.is_virtual() || reg.is_allocable() {
                        block.as_mut().live_def.remove(&reg);
                        block.as_mut().live_use.insert(reg);
                    }
                }
            }
            for reg in block.as_ref().live_use.iter() {
                queue.push_back((block.clone(), reg.clone()));
            }
            block.as_mut().live_in = block.as_ref().live_use.clone();
            block.as_mut().live_out.clear();
        }
        while let Some(value) = queue.pop_front() {
            let (block, reg) = value;
            for pred in block.as_ref().in_edge.iter() {
                if pred.as_mut().live_out.insert(reg) {
                    if pred.as_mut().live_def.take(&reg) == None && pred.as_mut().live_in.insert(reg) {
                        queue.push_back((pred.clone(), reg));
                    }
                }
            }
        }
    }

    pub fn allocate_reg(&mut self, f: &'static mut File) {
        // 函数返回地址保存在ra中
        //FIXME: 暂时使用固定的寄存器ra、a0与s0，即r1, r8, r10
        //FIXME:暂时只考虑int型
        let reg_int = vec![Reg::new(1, ScalarType::Int), Reg::new(8, ScalarType::Int)];

        let mut stack_size = 0;
        for it in self.stack_addr.iter().rev() {
            it.as_mut().set_pos(stack_size);
            stack_size += it.as_ref().get_size();
        }

        let mut reg_int_res = Vec::from(reg_int);
        let mut reg_int_res_cl = reg_int_res.clone();
        let reg_int_size = reg_int_res.len();
        
        //TODO:栈对齐 - 调用func时sp需按16字节对齐

        let mut offset = stack_size;
        let mut f1 = f.try_clone().unwrap();
        if let Some(contxt) = &self.context {
            contxt.as_mut().set_prologue_event(move||{
                let mut builder = AsmBuilder::new(f);
                // addi sp -stack_size
                builder.addi("sp", "sp", -offset);
                for src in reg_int_res.iter() {
                    offset -= 8;
                    builder.sd(&src.to_string(), "sp", offset, false);
                }
            });
            let mut offset = stack_size;
            contxt.as_mut().set_epilogue_event(move||{
                let mut builder = AsmBuilder::new(&mut f1);
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
    fn generate(&self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        AsmBuilder::new(f).show_func(&self.label);
        if let Some(contxt) = &self.context {
            contxt.as_mut().call_prologue_event();
            for block in self.blocks.iter() {
                block.as_ref().generate(contxt.clone(), f)?;
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
    pub fn new(reg_id: i32) -> Self {
        Self {
           reg_id,
           block: None,
           insts_it: Vec::new(),
        }
    }

    pub fn band_block(&mut self, block: ObjPtr<BB>) {
        self.block = Some(block);
    }

    pub fn get_block(&self) -> Option<ObjPtr<BB>> {
        self.block
    }

    pub fn add_inst(&mut self, inst: ObjPtr<LIRInst>) {
        self.insts_it.push(inst.clone());
    }

    pub fn add_insts(&mut self, insts: Vec<ObjPtr<LIRInst>>) {
        self.insts_it.append(&mut insts.clone());
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
    fn generate(&self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        writeln!(f, "unreachable")?;
        Ok(())
    }
}

impl PartialEq for CurInstrInfo {
    fn eq(&self, other: &Self) -> bool {
        self.reg_id == other.reg_id
    }
}

impl Eq for CurInstrInfo {}

impl Hash for CurInstrInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.reg_id.hash(state);
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