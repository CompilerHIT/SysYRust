pub use std::io::Result;
// use std::collections::HashSet;
use std::vec;
use std::cmp::min;

pub use crate::backend::structs::{BB, Func, Context, GenerateAsm};
pub use crate::utility::{ScalarType, Pointer};
pub use crate::backend::asm_builder::AsmBuilder;
use crate::backend::operand::*;

use super::structs::StackSlot;

#[derive(Clone, PartialEq)]
pub enum Operand {
    Addr(Addr),
    IImm(IImm),
    FImm(FImm),
    Reg(Reg),
}

pub enum InstrsType {
    Binary,
    OpReg,
    ChangeSp,
    StackLoad,
    StackStore,
    Load,
    Store,
    Call,
    Branch,
    Ret
}

/// trait for instructs for asm
pub trait Instrs: GenerateAsm {
    fn get_type(&self) -> InstrsType;
    fn get_reg_use(&self) -> Vec<Reg> { vec![] }
    fn get_reg_def(&self) -> Vec<Reg> { vec![] }
    

    // TODO: maybe todo
    // for reg alloc
    // fn replace_reg() {}  // todo: add regs() to get all regs the inst use

    // for conditional branch

    // fn replace_value() {}
    // fn replace_def_value() {}
    // fn replace_use_value() {}

}

//TODO:浮点数运算
#[derive(Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    /// Shift left logical.
    Shl,
    /// Shift right logical.
    Shr,
    /// Shift right arithmetic.
    Sar,
}

/// 比较运算符
enum CmpOp {
    Ne,
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
}

/// dst: reg = lhs: operand op rhs: operand
/// 默认左操作数为寄存器，为此需要进行常量折叠 / 交换(前端完成？)
pub struct Binary {
    op: BinaryOp,
    dst: Reg,
    lhs: Operand,
    rhs: Operand,
}

//FIXME:考虑是否将对lhs与rhs的clone操作换为ref
/// 为二元运算定义创建方法与获取成员的方法
impl Binary {
    pub fn new(op: BinaryOp, dst: Reg, lhs: Operand, rhs: Operand) -> Self {
        Self {
            op,
            dst,
            lhs,
            rhs
        }
    }
    pub fn get_op(&self) -> BinaryOp {
        self.op
    }
    pub fn get_mr_op(&mut self) -> &mut BinaryOp {
        &mut self.op
    }
    pub fn get_lhs(&self) -> Operand {
        self.lhs.clone()
    }
    pub fn get_mr_lhs(&mut self) -> &mut Operand {
        &mut self.lhs
    }
    pub fn get_rhs(&self) -> Operand {
        self.rhs.clone()
    }
    pub fn get_mr_rhs(&mut self) -> &mut Operand {
        &mut self.rhs
    }
    pub fn get_dst(&self) -> Reg {
        self.dst
    }
    pub fn get_mr_dst(&mut self) -> &mut Reg {
        &mut self.dst
    }
    pub fn if_limm(&self) -> bool {
        match self.lhs {
            Operand::IImm(_) => true,
            _ => false,
        }
    }
    pub fn if_rimm(&self) -> bool {
        match self.rhs {
            Operand::IImm(_) => true,
            _ => false,
        }
    }
}

/// 实现Instr与GenerateAsm的trait
impl Instrs for Binary {
    fn get_type(&self) -> InstrsType {
        InstrsType::Binary
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        let mut regs: Vec<Reg> = Vec::new();
        match self.lhs {
            Operand::Reg(reg) => {
                regs.push(reg);
            }
            _ => {}
        }
        match self.rhs {
            Operand::Reg(reg) => {
                regs.push(reg);
            }
            _ => {}
        }
        regs.push(self.dst);
        regs
    }
}

pub enum SingleOp {
    // Li, Lui, MvReg
    Mov,
    Not,
    FNeg,
    I2F,
    F2I,
    // F2D,
    // D2F,
}

pub struct OpReg {
    op: SingleOp,
    dst: Reg,
    src: Operand
}

impl OpReg {
    pub fn new(op: SingleOp, dst: Reg, src: Operand) -> Self {
        Self {
            op,
            dst,
            src,
        }
    }
}

impl Instrs for OpReg {
    fn get_type(&self) -> InstrsType {
        InstrsType::OpReg
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        let mut regs = Vec::new();
        regs.push(self.dst);
        match self.src {
            Operand::Reg(reg) => {
                regs.push(reg)
            }
            _ => {}
        }
        regs
    }
}

//TODO:
// enum StackOp {
//     ParamLoad,
//     StackAddr,
//     StackLoad,
//     StackStore,
// }

/// addi sp (-)imm, check legal first
pub struct ChangeSp {
    offset: IImm
}

impl ChangeSp {
    pub fn new(offset: IImm) -> Self {
        Self {
            offset
        }
    }
    pub fn get_offset(&self) -> i32 {
        self.offset.get_data()
    }
}

impl Instrs for ChangeSp {
    fn get_type(&self) -> InstrsType {
        InstrsType::ChangeSp
    }    
}

pub struct Load {
    dst: Reg,
    src: Reg,
    offset: IImm
}

impl Load {
    pub fn new(dst: Reg, src: Reg, offset: IImm) -> Self {
        Self {
            dst,
            src,
            offset,
        }
    }
}

impl Instrs for Load {
    fn get_type(&self) -> InstrsType {
        InstrsType::Load
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![self.src, self.dst]
    }
}

pub struct Store {
    dst: Reg,
    src: Reg,
    offset: IImm
}

impl Store {
    pub fn new(dst: Reg, src: Reg, offset: IImm) -> Self {
        Self {
            dst,
            src,
            offset,
        }
    }
}

impl Instrs for Store {
    fn get_type(&self) -> InstrsType {
        InstrsType::Store
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![self.src, self.dst]
    }
}

pub struct StackLoad {
    src: Pointer<StackSlot>,
    dst: Reg,
    offset: IImm,
}

impl StackLoad {
    pub fn new(src: Pointer<StackSlot>, dst: Reg, offset: IImm) -> Self {
        Self { src, dst, offset }
    }
    pub fn set_offset(&mut self, offset: IImm) {
        self.offset = offset;
    }
    pub fn get_offset(&self) -> IImm {
        self.offset
    }
    pub fn get_src(&self) -> Pointer<StackSlot> {
        self.src.clone()
    }
    pub fn get_dst(&self) -> Reg {
        self.dst
    }
}

impl Instrs for StackLoad {
    fn get_type(&self) -> InstrsType {
        InstrsType::StackLoad
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![self.dst]
    }
}

pub struct StackStore {
    src: Reg,
    dst: Pointer<StackSlot>,
    offset: IImm
}

impl StackStore {
    pub fn new(src: Reg, dst: Pointer<StackSlot>, offset: IImm) -> Self {
        Self { src, dst, offset }
    }
    pub fn set_offset(&mut self, offset: IImm) {
        self.offset = offset;
    }
    pub fn get_offset(&self) -> IImm {
        self.offset
    }
    pub fn get_src(&self) -> Reg {
        self.src
    }
    pub fn get_dst(&self) -> Pointer<StackSlot> {
        self.dst.clone()
    }
}

impl Instrs for StackStore {
    fn get_type(&self) -> InstrsType {
        InstrsType::StackStore
    }
    
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![self.src]
    }
}

pub struct Bz {
    Cond: CmpOp,
    src: Reg,
    label: Operand,
}


pub struct Call {
    // block: BB,
    // callee: Func,
    // args: Vec<Operand>,
    lable: String,
    iarg_cnt: usize,
    farg_cnt: usize,
}
impl Instrs for Call {
    fn get_type(&self) -> InstrsType {
        InstrsType::Call
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        let mut set = Vec::new();
        let icnt: usize = min(self.iarg_cnt, ARG_REG_COUNT);
        let mut ni = icnt;
        while ni > 0 {
            set.push(Reg::new(icnt - ni, ScalarType::Int));
            ni -= 1;
        }
        let fcnt: usize = min(self.farg_cnt, ARG_REG_COUNT);
        let mut nf = fcnt;
        while nf > 0 {
            set.push(Reg::new(fcnt - nf, ScalarType::Float));
            nf -= 1;
        } 
        set
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        let mut set = Vec::new();
        let cnt: usize = REG_COUNT;
        let mut n = cnt;
        while n > 0 {
            let ireg = Reg::new(cnt - n, ScalarType::Int);
            if ireg.is_caller_save() && !ireg.is_special() {
                set.push(ireg);
            }
            let freg = Reg::new(cnt - n, ScalarType::Float);
            if freg.is_caller_save() {
                set.push(freg);
            }
            n -= 1;
        }
        set
    }
}

//FIXME: whether need ret instr? 
pub struct Return {
    re_type: ScalarType,
}

impl Return {
    pub fn new(re_type: ScalarType) -> Self {
        Self {
            re_type,
        }
    }
}

impl Instrs for Return {
    fn get_type(&self) -> InstrsType {
        InstrsType::Ret
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        match self.re_type {
            ScalarType::Int => vec![Reg::new(0, ScalarType::Int)],
            ScalarType::Float => vec![Reg::new(0, ScalarType::Float)],
            ScalarType::Void => vec![],
        }
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        match self.re_type {
            ScalarType::Int => vec![Reg::new(0, ScalarType::Int)],
            ScalarType::Float => vec![Reg::new(0, ScalarType::Float)],
            ScalarType::Void => vec![],
        }
    }
}

// pub fn process_change_sp_instrs(offset: &IImm) -> Option<Pointer<Box<dyn Instrs>>> {
//     if offset.get_data() == 0 {
//         return None;
//     }
//     if offset.()
// }

// impl Instrs for Call {}
// impl Instrs for Return {}
// impl Instrs for Load {}
// impl Instrs for Store {}
// impl Instrs for FToI {}
// impl Instrs for IToF {}

