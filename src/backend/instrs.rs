pub use std::io::Result;
use std::collections::HashMap;
use std::vec;
use std::cmp::min;

pub use crate::backend::structs::{Context, GenerateAsm};
pub use crate::backend::block::BB;
pub use crate::backend::func::Func;
pub use crate::utility::{ScalarType, ObjPtr};
pub use crate::backend::asm_builder::AsmBuilder;
use crate::backend::operand::*;

#[derive(Clone, PartialEq)]
pub enum Operand {
    IImm(IImm),
    FImm(FImm),
    Reg(Reg),
    Addr(String)
}

//TODO:浮点数运算
/// 二元运算符
#[derive(Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
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

/// 单目运算符
#[derive(Copy, Clone)]
pub enum SingleOp {
    Li,
    Lui,
    IMv,
    FMv,
    INot,
    INeg,
    //FIXME: whether fnot exist
    FNot,
    FNeg,
    I2F,
    F2I,
    LoadAddr,
    // F2D,
    // D2F,
}

/// 比较运算符
#[derive(Copy, Clone)]
pub enum CmpOp {
    Ne,
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Copy, Clone)]
pub enum InstrsType {
    // dst: reg = lhs: operand op rhs: operand
    // 默认左操作数为寄存器，为此需要进行常量折叠 / 交换(前端完成？)
    Binary(BinaryOp),
    // dst: reg = op src: operand
    OpReg(SingleOp),
    // addi sp (-)imm, check legal first
    // ChangeSp,
    // src: stackslot, dst: reg, offset: iimm  
    LoadFromStack,
    // src: reg, dst: stackslot, offset: iimm
    StoreToStack,
    // src: stackslot, dst: reg, offset: iimm
    LoadParamFromStack,
    StoreParamToStack,
    // dst: reg, src: iimm(reg)
    Load,
    // dst: iimm(reg), src: reg
    Store,
    // call "funcname"
    Call,
    // bcmop src1: reg, src2: reg, block
    Branch(CmpOp),
    // j block
    Jump,
    Ret(ScalarType),
}

pub struct LIRInst {
    inst_type: InstrsType,
    // 0:Dst, 1...n:Srcs
    operands: Vec<Operand>,
    // param cnts in call instruction: (ints, floats)
    param_cnt: (i32, i32),
    // call指令的跳转到函数
    func: Option<ObjPtr<Func>>,
    func_name: String,
    double: bool,
}

impl LIRInst {
    // 通用
    pub fn new(inst_type: InstrsType, operands: Vec<Operand>) -> Self {
        Self { inst_type, operands, param_cnt: (0, 0), func: None, func_name: String::new(), double: false }
    }
    pub fn get_type(&self) -> InstrsType {
        self.inst_type
    }

    pub fn get_dst(&self) -> &Operand {
        &self.operands[0]
    }
    pub fn get_dst_mut(&mut self) -> &mut Operand {
        &mut self.operands[0]
    }
    // lhs一定存在
    pub fn get_lhs(&self) -> &Operand {
        &self.operands[1]
    }
    pub fn get_lhs_mut(&mut self) -> &mut Operand {
        &mut self.operands[1]
    }
    fn is_rhs_exist(&self) -> bool {
        if self.operands.len() < 3 { false } else { true }
    }
    // rhs不一定存在
    pub fn get_rhs(&self) -> &Operand {
        if !self.is_rhs_exist() {
            panic!("Error call for instr's rhs");
        } else {
            &self.operands[2]
        }
    }
    pub fn get_rhs_mut(&mut self) -> &mut Operand {
        if !self.is_rhs_exist() {
            panic!("Error call for instr's rhs");
        } else {
            &mut self.operands[2]
        }
    }

    // mapping virtual reg_id to physic reg_id
    pub fn v_to_phy(&mut self, regs: Vec<Reg>, map: HashMap<i32, i32>) {
        for operand in &self.operands {
            match operand {
                Operand::Reg(mut reg) => {
                    if let Some(id) = map.get(&reg.get_id()) {
                        reg.map_id(id.clone());
                    } else {
                        panic!("not find physic mapping");
                    }
                }
                _ => {}
            }
        }
    }

    // instr's def/use regs
    pub fn get_reg_def(&self) -> Vec<Reg> {
        match self.inst_type {
            InstrsType::Binary(..) | InstrsType::OpReg(..) | InstrsType::Load | InstrsType::Store |
            InstrsType::LoadFromStack | InstrsType::LoadParamFromStack =>
            { 
                match self.operands[0] {
                    Operand::Reg(dst_reg) => vec![dst_reg],
                    _ => panic!("dst must be reg")
                }
            },
            InstrsType::Call => {
                let mut set = Vec::new();
                let cnt: i32 = REG_COUNT;
                let mut n = cnt;
                while n > 0 {
                    let ireg = Reg::new(cnt - n, ScalarType::Int);
                    if ireg.is_caller_save() {
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
            InstrsType::StoreToStack | InstrsType::StoreParamToStack | InstrsType::Jump | InstrsType::Branch(..) => vec![],

            InstrsType::Ret(re_type) => {
                match re_type {
                    ScalarType::Int => vec![Reg::new(10, ScalarType::Int)],
                    ScalarType::Float => vec![Reg::new(10, ScalarType::Float)],
                    ScalarType::Void => vec![],
                }
            }
        }
    }
    pub fn get_reg_use(&self) -> Vec<Reg> {
        match self.inst_type {
            InstrsType::Binary(..) | InstrsType::OpReg(..) | InstrsType::Load |
            InstrsType::Store | InstrsType::LoadFromStack | InstrsType::StoreToStack | InstrsType::Branch(..) |
            InstrsType::Jump | InstrsType::LoadParamFromStack | InstrsType::StoreParamToStack => {
                let mut regs = self.operands.clone();
                let mut res = Vec::new();
                while let Some(operand) = regs.pop() {
                    match operand {
                        Operand::Reg(reg) => res.push(reg),
                        _ => {}
                    }
                }           
                res
            },
            InstrsType::Call => {
                let mut set = Vec::new();
                let (iarg_cnt, farg_cnt) = self.param_cnt;
                let mut ni = 0;
                while ni < min(iarg_cnt, REG_COUNT) {
                    // if 
                    set.push(Reg::new(ni, ScalarType::Int));
                    ni += 1;
                }
                let mut nf = 0;
                while nf < min(farg_cnt, REG_COUNT) {
                    set.push(Reg::new(nf, ScalarType::Float));
                    nf += 1;
                } 
                set
            }
            InstrsType::Ret(re_type) => {
                match re_type {
                    ScalarType::Int => vec![Reg::new(10, ScalarType::Int)],
                    ScalarType::Float => vec![Reg::new(10, ScalarType::Float)],
                    ScalarType::Void => vec![],
                }
            },
        }
    }

    // 对特定指令执行的操作
    // Call:
    pub fn set_param_cnts(&mut self, int_cnt: i32, float_cnt: i32) {
        self.param_cnt = (int_cnt, float_cnt);
    }

    pub fn get_param_cnts(&self) -> (i32, i32) {
        self.param_cnt
    }

    // // ChangeSp:
    // pub fn get_change_sp_offset(&self) -> i32 {
    //     match self.operands[0] {
    //         Operand::IImm(offset) => offset.get_data(),
    //         _ => panic!("only support imm sp offset"),
    //     }
    // }

    // Load, Store:
    pub fn set_offset(&mut self, offset: IImm) {
        self.operands[2] = Operand::IImm(offset);
    }
    pub fn get_offset(&self) -> IImm {
        match self.operands[2] {
            Operand::IImm(offset) => offset,
            _ => unreachable!("only support imm sp offset"),
        }
    }

    // Branch, Jump:
    pub fn get_label(&self) -> &Operand {
        &self.operands[0]
    }

    // LoadFromStack, StoreToStack
    pub fn set_stack_offset(&mut self, offset: IImm) {
        self.operands[1] = Operand::IImm(offset);
    }
    pub fn get_stack_offset(&self) -> IImm {
        match self.operands[1] {
            Operand::IImm(offset) => offset,
            _ => unreachable!("only support imm sp offset"),
        }
    }

    pub fn set_double(&mut self) {
        self.double = true;
    }
    pub fn is_double(&self) -> bool {
        self.double
    }

    // LoadParamFromStack(include alloca):
    // pub fn set_true_offset(&mut self, offset: i32) {
    //     self.operands[1] = Operand::IImm(IImm::new(offset));
    // }
    // pub fn get_true_offset(&self) -> IImm {
    //     match self.operands[1].clone() {
    //         Operand::IImm(iimm) => iimm,
    //         _ => unreachable!("only support imm sp offset"),
    //     }
    // }

    // TODO: maybe todo
    // for reg alloc
    // fn replace_reg() {}  // todo: add regs() to get all regs the inst use

    // for conditional branch

    // fn replace_value() {}
    // fn replace_def_value() {}
    // fn replace_use_value() {}
}


//TODO: maybe
// enum StackOp {
//     ParamLoad,
//     StackAddr,
//     StackLoad,
//     StackStore,
// }