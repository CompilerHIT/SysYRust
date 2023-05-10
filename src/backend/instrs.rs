pub use std::io::Result;
// use std::collections::HashSet;
use std::vec;
use std::cmp::min;

pub use crate::backend::structs::{BB, Func, Context, GenerateAsm};
pub use crate::utility::{ScalarType, Pointer};
pub use crate::backend::asm_builder::AsmBuilder;
use crate::backend::operand::*;

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

    Load,
    Store,
    Call,
    Branch,
    Ret
}

/// trait for instructs for asm
pub trait Instrs: GenerateAsm {
    fn get_type(&self) -> InstrsType;
    fn get_reg_use(&self) -> Vec<Reg>;
    fn get_reg_def(&self) -> Vec<Reg>;
    

    // TODO: maybe todo
    // for reg alloc
    // fn replace_reg() {}  // todo: add regs() to get all regs the inst use

    // for conditional branch

    // fn replace_value() {}
    // fn replace_def_value() {}
    // fn replace_use_value() {}

}

/// 判断是否是合法的立即数
pub trait LegalImm {
    fn is_legal_imm(&self) -> bool;
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
    def_regs: Vec<Reg>,
    use_regs: Vec<Reg>,
}

//FIXME:考虑是否将对lhs与rhs的clone操作换为ref
/// 为二元运算定义创建方法与获取成员的方法
impl Binary {
    pub fn new(op: BinaryOp, dst: Reg, lhs: Operand, rhs: Operand) -> Self {
        Self {
            op,
            dst,
            lhs,
            rhs,
            def_regs: Vec::new(),
            use_regs: Vec::new(),
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
    pub fn get_offset(&self) -> isize {
        self.offset.get_data()
    }
}

impl Instrs for ChangeSp {
    fn get_type(&self) -> InstrsType {
        InstrsType::ChangeSp
    }    
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![Reg::new(REG_SP, ScalarType::Int)]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![Reg::new(REG_SP, ScalarType::Int)]
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

impl Instrs for Return {
    fn get_type(&self) -> InstrsType {
        InstrsType::Ret
    }
    fn get_reg_def(&self) -> Vec<Reg> {
        match self.re_type {
            ScalarType::Int => vec![Reg::new(0, ScalarType::Int)],
            ScalarType::Float => vec![Reg::new(0, ScalarType::Float)],
        }
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        match self.re_type {
            ScalarType::Int => vec![Reg::new(0, ScalarType::Int)],
            ScalarType::Float => vec![Reg::new(0, ScalarType::Float)]
        }
    }
}



// impl Instrs for Call {}
// impl Instrs for Return {}
// impl Instrs for Load {}
// impl Instrs for Store {}
// impl Instrs for FToI {}
// impl Instrs for IToF {}

