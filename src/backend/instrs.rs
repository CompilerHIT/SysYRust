use std::fs::File;
use std::io::Result;
// trait for instructs for asm
trait GenerateToAsm {
    type Target;
    fn generate(&self, f: &mut File) -> Result<Self::Target>;
}

struct Unary {
    
}

// TODO: IReg or FReg, IImm or FImm
#[derive(Clone, Copy)]
enum Value {
    Addr,
    Imm,
    Reg
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

struct Binary {
    op: BinaryOp,
    lhs: Value,
    rhs: Value
}

impl Binary {
    fn get_op(&self) -> BinaryOp {
        self.op
    }
    fn get_mr_op(&mut self) -> &mut BinaryOp {
        &mut self.op
    }
    fn get_lhs(&self) -> Value {
        self.lhs
    }
    fn get_mr_lhs(&mut self) -> &mut Value {
        &mut self.lhs
    }
    fn get_rhs(&self) -> Value {
        self.rhs
    }
    fn get_mr_rhs(&mut self) -> &mut Value {
        &mut self.rhs
    }
}

trait RegCond {
    fn reg_use() {}
    fn reg_def() {}
}

trait ReplaceInst {
    fn replace_value() {}
    fn replace_def_value() {}
    fn replace_use_value() {}
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
    args: Vec<Value>,
}

pub struct Return {

}

pub struct Load {

}

pub struct Store {

}

pub struct FToI {

}

pub struct IToF {
    
}