use std::cmp::min;
use std::collections::HashSet;
use std::vec;

use crate::backend::operand::*;
use crate::backend::structs::{BB, Func, Context, GenerateAsm};
use crate::utility::{ScalarType, Pointer};

#[derive(Clone, Copy, PartialEq)]
enum Operand {
    Addr(Addr),
    IImm(IImm),
    FImm(FImm),
    Reg(Reg),
}

// trait for instructs for asm
pub trait Instrs: GenerateAsm {
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

pub trait LegalImm {
    fn is_legal_imm(&self) -> bool;
}

//TODO:浮点数运算
#[derive(Clone, Copy)]
enum BinaryOp {
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

enum CmpOp {
    Ne,
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
}

// dst: reg = lhs: operand op rhs: operand
pub struct Binary {
    op: BinaryOp,
    dst: Reg,
    lhs: Operand,
    rhs: Operand,
    def_regs: Vec<Reg>,
    use_regs: Vec<Reg>,
}

impl Binary {
    fn new(op: BinaryOp, dst: Reg, lhs: Operand, rhs: Operand) -> Self {
        Self {
            op,
            dst,
            lhs,
            rhs,
            def_regs: Vec::new(),
            use_regs: Vec::new(),
        }
    }
    fn get_op(&self) -> BinaryOp {
        self.op
    }
    fn get_mr_op(&mut self) -> &mut BinaryOp {
        &mut self.op
    }
    fn get_lhs(&self) -> Operand {
        self.lhs
    }
    fn get_mr_lhs(&mut self) -> &mut Operand {
        &mut self.lhs
    }
    fn get_rhs(&self) -> Operand {
        self.rhs
    }
    fn get_mr_rhs(&mut self) -> &mut Operand {
        &mut self.rhs
    }
    fn get_dst(&self) -> Reg {
        self.dst
    }
    fn get_mr_dst(&mut self) -> &mut Reg {
        &mut self.dst
    }
    fn if_limm(&self) -> bool {
        match self.lhs {
            Operand::IImm(_) => true,
            _ => false,
        }
    }
    fn if_rimm(&self) -> bool {
        match self.rhs {
            Operand::IImm(_) => true,
            _ => false,
        }
    }
}

impl Instrs for Binary {
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

// dst: reg = mv src: reg
pub struct MvReg {
    dst: Reg,
    src: Reg,
}

impl MvReg {
    pub fn new(dst: Reg, src: Reg) -> Self {
        Self {
            dst,
            src,
        }
    }
}

impl Instrs for MvReg {
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![self.src, self.dst]
    }
}

// dst: reg = li src: iimm
pub struct Li {
    dst: Reg,
    src: IImm
}

impl Li {
    pub fn new(dst: Reg, src: IImm) -> Self {
        Self {
            dst,
            src,
        }
    }
}

impl LegalImm for Li {
    fn is_legal_imm(&self) -> bool {
        self.src.is_imm_20bs()
    }
}

impl Instrs for Li {
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![self.dst]
    }
}

pub struct Lui {
    dst: Reg,
    src: IImm
}

impl Lui {
    pub fn new(dst: Reg, src: IImm) -> Self {
        Self {
            dst,
            src,
        }
    }
}

impl LegalImm for Lui {
    fn is_legal_imm(&self) -> bool {
        self.src.is_imm_20bs()
    }
}

impl Instrs for Lui {
    fn get_reg_def(&self) -> Vec<Reg> {
        vec![self.dst]
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        vec![self.dst]
    }
}

enum SingleOp {
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
    }
}

//TODO:
enum StackOp {
    ParamLoad,
    StackAddr,
    StackLoad,
    StackStore,
}


pub struct Load {
    dst: Reg,
    src: Reg,
    offset: u32
}

impl Load {
    pub fn new(dst: Reg, src: Reg, offset: u32) -> Self {
        Self {
            dst,
            src,
            offset,
        }
    }
}

impl Instrs for Load {
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
    offset: u32
}

impl Store {
    pub fn new(dst: Reg, src: Reg, offset: u32) -> Self {
        Self {
            dst,
            src,
            offset,
        }
    }
}

impl Instrs for Store {
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
    // fn create_reg_def(&self) -> HashSet<Reg> {
    //     let mut set: HashSet<Reg> = HashSet::new();
    //     let icnt: usize = min(self.iarg_cnt, ARG_REG_COUNT);
    //     let mut ni = icnt;
    //     while ni > 0 {
    //         set.insert(Reg::new(icnt - ni, ScalarType::Int));
    //         ni -= 1;
    //     }
    //     let fcnt: usize = min(self.farg_cnt, ARG_REG_COUNT);
    //     let mut nf = fcnt;
    //     while nf > 0 {
    //         set.insert(Reg::new(fcnt - nf, ScalarType::Float));
    //         nf -= 1;
    //     } 
    //     set
    // }
    // fn create_reg_use(&self) -> HashSet<Reg> {
    //     let mut set: HashSet<Reg> = HashSet::new();
    //     let cnt: usize = REG_COUNT;
    //     let mut n = cnt;
    //     while n > 0 {
    //         let ireg = Reg::new(cnt - n, ScalarType::Int);
    //         if ireg.is_caller_save() && !ireg.is_special() {
    //             set.insert(ireg);
    //         }
    //         let freg = Reg::new(cnt - n, ScalarType::Float);
    //         if freg.is_caller_save() {
    //             set.insert(freg);
    //         }
    //         n -= 1;
    //     }
    //     set
    // }
    fn get_reg_def(&self) -> Vec<Reg> {
        
    }
    fn get_reg_use(&self) -> Vec<Reg> {
        
    }
}

/* FIXME: whether need ret instr?
pub struct Return {}
*/


// impl Instrs for Call {}
// impl Instrs for Return {}
// impl Instrs for Load {}
// impl Instrs for Store {}
// impl Instrs for FToI {}
// impl Instrs for IToF {}
