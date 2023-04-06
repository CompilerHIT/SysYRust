use std::collections::HashSet;
use std::fs::File;
use crate::backend::operand::*;


#[derive(Clone, Copy, PartialEq)]
enum Operand {
    Addr(Addr),
    IImm(IImm),
    FImm(FImm),
    Reg(Reg)
}

// trait for instructs for asm
pub trait Instrs {
    fn create_reg_use(&self) -> HashSet<Reg>;
    fn create_reg_def(&self) -> HashSet<Reg>;
    // fn replace_value() {}
    // fn replace_def_value() {}
    // fn replace_use_value() {}
    fn generate(&self, f: &mut File) -> String {
        String::from("todo")
    }
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

pub struct Binary {
    op: BinaryOp,
    dst: Reg,
    lhs: Operand,
    rhs: Operand
}

impl Binary {
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
}

//TODO:
enum StackOp {
    ParamLoad,
    StackAddr,
    StackLoad,
    StackStore
}

pub struct Call {
    // block: Block,
    // callee: Function,
    // func_name: String,
    // param_cnt: usize,
    // float_param_cnt: usize,
    args: Vec<Operand>,
}

pub struct Return {

}

pub struct Load {

}

pub struct Store {

}

pub struct MvReg {
    dst: Reg,
    src: Reg
}

pub struct MvIImm {
    dst: Reg,
    src: IImm
}

pub struct MvFImm {
    dst: Reg,
    src: FImm
}

pub struct Bz {
    Cond: CmpOp,
    src: Reg,
    label: Operand
}

pub struct IUnary {
    src: IImm
}
pub struct FUnary {
    src: FImm
}

pub struct FToI {

}

pub struct IToF {
    
}


impl Instrs for Binary {
    fn create_reg_def(&self) -> HashSet<Reg> {
        let mut set: HashSet<Reg> = HashSet::new();
        set.insert(self.get_dst());
        set
    }
    fn create_reg_use(&self) -> HashSet<Reg> {
        let mut set: HashSet<Reg> = HashSet::new();
        let lhs = self.get_lhs();
        let rhs = self.get_rhs();
        match lhs {
            Operand::Reg(reg) => {
                set.insert(reg);
            }
            _ => {}
        }
        match rhs {
            Operand::Reg(reg) => {
                set.insert(reg);
            }
            _ => {}
        }
        set.insert(self.get_dst());
        set
    }
}
// impl Instrs for Call {}
// impl Instrs for Return {}
// impl Instrs for Load {}
// impl Instrs for Store {}
// impl Instrs for FToI {}
// impl Instrs for IToF {}