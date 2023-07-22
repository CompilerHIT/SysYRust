use std::cmp::min;
use std::collections::{HashMap, HashSet};
pub use std::io::Result;
use std::{fmt, vec};

pub use crate::backend::asm_builder::AsmBuilder;
pub use crate::backend::block::BB;
pub use crate::backend::func::Func;
use crate::backend::operand::*;
pub use crate::backend::structs::{Context, GenerateAsm};
// use crate::log_file;
pub use crate::utility::{ObjPtr, ScalarType};

use super::block::FLOAT_BASE;

#[derive(Clone, PartialEq, Debug)]
pub enum Operand {
    IImm(IImm),
    FImm(FImm),
    Reg(Reg),
    Addr(String),
}

//TODO:浮点数运算
/// 二元运算符
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Slt,
    /// Shift left logical.
    Shl,
    /// Shift right logical.
    Shr,
    /// Shift right arithmetic.
    Sar,
    /// 执行带符号整数高位乘法操作
    FCmp(CmpOp),
}

/// 单目运算符
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SingleOp {
    Li,
    // Lui,
    Mv,
    Neg,
    //FIXME: whether fnot exist
    I2F,
    F2I,
    LoadAddr,
    // F2D,
    // D2F,
    Seqz,
    Snez,
    LoadFImm,
}

/// 比较运算符
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CmpOp {
    Ne,
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
    Eqz,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    // LoadGlobal,
}

#[derive(Debug, Clone)]
pub struct LIRInst {
    inst_type: InstrsType,
    // 0:Dst, 1...n:Srcs
    pub operands: Vec<Operand>,
    // param cnts in call instruction: (ints, floats)
    param_cnt: (i32, i32),
    // call指令的跳转到函数
    double: bool,
    float: bool,
    func_type: ScalarType,
}

// 实现个fmt display trait
impl fmt::Display for LIRInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let kind;
        match self.get_type() {
            InstrsType::Binary(op) => {
                kind = match op {
                    BinaryOp::Add => "add",
                    BinaryOp::Sub => "sub",
                    BinaryOp::Mul => "mul",
                    BinaryOp::Div => "div",
                    BinaryOp::Rem => "rem",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                    BinaryOp::Xor => "xor",
                    BinaryOp::Shl => "sll",
                    BinaryOp::Shr => "srl",
                    BinaryOp::Sar => "sra",
                    BinaryOp::Slt => "slt",
                    BinaryOp::FCmp(..) => "fcmp",
                };
            }
            InstrsType::OpReg(op) => {
                kind = match op {
                    SingleOp::Li => "li",
                    // SingleOp::Lui => "lui",
                    SingleOp::Mv => "mv",
                    SingleOp::Neg => "neg",
                    SingleOp::I2F => "fcvt.s.w",
                    SingleOp::F2I => "fcvt.w.s",
                    SingleOp::LoadAddr => "la",
                    SingleOp::Seqz => "seqz",
                    SingleOp::Snez => "snez",
                    SingleOp::LoadFImm => "fmv.w.x",
                };
            }
            InstrsType::Load | InstrsType::LoadParamFromStack | InstrsType::LoadFromStack => {
                kind = "load";
            }
            InstrsType::Store | InstrsType::StoreToStack | InstrsType::StoreParamToStack => {
                kind = "store";
            }
            // 判断！是否需要多插入一条j，间接跳转到
            InstrsType::Branch(cmp_op) => {
                kind = match cmp_op {
                    CmpOp::Eq => "eq",
                    CmpOp::Ne => "ne",
                    CmpOp::Lt => "lt",
                    CmpOp::Le => "le",
                    CmpOp::Gt => "gt",
                    CmpOp::Ge => "ge",
                    CmpOp::Eqz => "eqz",
                };
            }
            InstrsType::Jump => {
                kind = "jump";
            }
            InstrsType::Call => {
                kind = "call";
            }
            InstrsType::Ret(..) => {
                kind = "ret";
            }
        }
        let mut def = Vec::new();
        let mut use_reg_id = Vec::new();
        self.get_reg_def().iter().for_each(|e| {
            def.push(e.to_string(true));
        });
        self.get_reg_use().iter().for_each(|e| {
            use_reg_id.push(e.to_string(true));
        });
        write!(f, "{:?} def:{:?} use:{:?}", kind, def, use_reg_id)
    }
}

impl LIRInst {
    // 通用
    pub fn new(inst_type: InstrsType, operands: Vec<Operand>) -> Self {
        Self {
            inst_type,
            operands,
            param_cnt: (0, 0),
            double: false,
            float: false,
            func_type: ScalarType::Void,
        }
    }
    pub fn get_type(&self) -> InstrsType {
        self.inst_type
    }

    pub fn replace_only_use_reg(&mut self, old_reg: &Reg, new_reg: &Reg) {
        if self.is_rhs_exist() && self.get_rhs().drop_reg() == *old_reg {
            *self.get_rhs_mut() = Operand::Reg(*new_reg);
        }
        if self.is_lhs_exist() && self.get_lhs().drop_reg() == *old_reg {
            *self.get_lhs_mut() = Operand::Reg(*new_reg);
        }
    }

    pub fn replace_only_def_reg(&mut self, old_reg: &Reg, new_reg: &Reg) {
        if self.operands.len() > 0 && self.get_dst().drop_reg() == *old_reg {
            *self.get_dst_mut() = Operand::Reg(*new_reg);
        }
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
    pub fn is_lhs_exist(&self) -> bool {
        if self.operands.len() < 2 {
            false
        } else {
            true
        }
    }

    pub fn is_rhs_exist(&self) -> bool {
        if self.operands.len() < 3 {
            false
        } else {
            true
        }
    }
    // rhs不一定存在
    pub fn get_rhs(&self) -> &Operand {
        if !self.is_rhs_exist() {
            panic!("Error call for instr's rhs, instr is {:?}", self);
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

    // mapping virtual reg_id to physic reg_id, 物理寄存器不映射
    pub fn v_to_phy(&mut self, map: HashMap<i32, i32>, tmp_vars: HashSet<Reg>) {
        let mut index = 0;
        loop {
            if index >= self.operands.len() {
                break;
            }
            match self.operands[index] {
                Operand::Reg(ref mut reg) => {
                    if !reg.is_physic() && !tmp_vars.contains(reg) {
                        if let Some(new) = map.get(&reg.get_id()) {
                            self.operands[index] = Operand::Reg(Reg::new(*new, reg.get_type()));
                        }
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }
    pub fn replace_reg(&mut self, old_reg: &Reg, new_reg: &Reg) {
        let mut index = 0;
        loop {
            if index >= self.operands.len() {
                break;
            }
            match self.operands[index] {
                Operand::Reg(ref mut reg) => {
                    if reg == old_reg {
                        self.operands[index] = Operand::Reg(*new_reg);
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }

    pub fn is_spill(&self, id: &HashSet<i32>) -> HashSet<Reg> {
        let mut res = HashSet::new();
        for op in &self.operands {
            match op {
                Operand::Reg(reg) => {
                    if id.contains(&reg.get_id()) {
                        res.insert(*reg);
                    }
                }
                _ => {}
            }
        }
        res
    }

    pub fn replace(&mut self, old: i32, new: i32) {
        let mut index = 0;
        loop {
            if index >= self.operands.len() {
                break;
            }
            match self.operands[index] {
                Operand::Reg(ref mut reg) => {
                    if reg.get_id() == old {
                        self.operands[index] = Operand::Reg(Reg::new(new, reg.get_type()));
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }

    pub fn replace_op(&mut self, operands: Vec<Operand>) {
        self.operands = operands;
    }

    pub fn replace_kind(&mut self, inst_type: InstrsType) {
        self.inst_type = inst_type;
    }

    pub fn replace_label(&mut self, label: String) {
        self.operands[0] = Operand::Addr(label);
    }

    pub fn get_regs(&self) -> Vec<Reg> {
        let mut out = Vec::with_capacity(3);
        let mut used = HashSet::new();
        self.get_reg_def().iter().for_each(|e| {
            if used.contains(&e.get_id()) {
                return;
            }
            used.insert(e.get_id());
            out.push(*e);
        });
        self.get_reg_use().iter().for_each(|e| {
            if used.contains(&e.get_id()) {
                return;
            }
            used.insert(e.get_id());
            out.push(*e);
        });
        out
    }

    // instr's def/use regs
    pub fn get_reg_def(&self) -> Vec<Reg> {
        match self.inst_type {
            InstrsType::Binary(..)
            | InstrsType::OpReg(..)
            | InstrsType::Load
            | InstrsType::LoadFromStack
            | InstrsType::LoadParamFromStack => match self.operands[0] {
                Operand::Reg(dst_reg) => vec![dst_reg],
                _ => panic!(
                    "dst must be reg, but actually is {:?}, at LIRInst:{:?}",
                    self.operands[0], self
                ),
            },
            InstrsType::Call => {
                // let mut set = Vec::new();
                // let cnt: i32 = REG_COUNT;
                // let mut n = cnt;
                // while n > 0 {
                //     let ireg = Reg::new(cnt - n, ScalarType::Int);
                //     if ireg.is_caller_save() {
                //         set.push(ireg);
                //     }
                //     let freg = Reg::new(cnt - n + FLOAT_BASE, ScalarType::Float);
                //     if freg.is_caller_save() {
                //         set.push(freg);
                //     }
                //     n -= 1;
                // }
                // set
                if self.func_type == ScalarType::Int {
                    vec![Reg::new(10, self.func_type)]
                } else if self.func_type == ScalarType::Float {
                    vec![Reg::new(10 + FLOAT_BASE, self.func_type)]
                } else {
                    vec![]
                }
            }
            InstrsType::StoreToStack
            | InstrsType::StoreParamToStack
            | InstrsType::Jump
            | InstrsType::Branch(..)
            | InstrsType::Store => vec![],

            InstrsType::Ret(re_type) => match re_type {
                // ScalarType::Int => vec![Reg::new(10, ScalarType::Int)],
                // ScalarType::Float => vec![Reg::new(10 + FLOAT_BASE, ScalarType::Float)],
                // ScalarType::Void => vec![],
                _ => vec![],
            },
        }
    }
    pub fn get_reg_use(&self) -> Vec<Reg> {
        match self.inst_type {
            InstrsType::Binary(..)
            | InstrsType::OpReg(..)
            | InstrsType::Load
            | InstrsType::LoadFromStack
            | InstrsType::Branch(..)
            | InstrsType::Jump
            | InstrsType::LoadParamFromStack => {
                let regs = self.operands.clone();
                let mut res = Vec::new();
                for (i, operand) in regs.iter().enumerate() {
                    if *operand == *self.get_dst() && i == 0 {
                        continue;
                    }
                    match operand {
                        Operand::Reg(reg) => res.push(*reg),
                        _ => {}
                    }
                }
                res
            }
            InstrsType::Store | InstrsType::StoreParamToStack | InstrsType::StoreToStack => {
                let mut regs = self.operands.clone();
                let mut res = Vec::new();
                while let Some(operand) = regs.pop() {
                    match operand {
                        Operand::Reg(reg) => res.push(reg),
                        _ => {}
                    }
                }
                res
            }
            InstrsType::Call => {
                let mut set = Vec::new();
                let (iarg_cnt, farg_cnt) = self.param_cnt;
                let mut ni = 0;
                while ni < min(iarg_cnt, ARG_REG_COUNT) {
                    // if
                    set.push(Reg::new(ni + 10, ScalarType::Int));
                    ni += 1;
                }
                let mut nf = 0;
                while nf < min(farg_cnt, ARG_REG_COUNT) {
                    set.push(Reg::new(nf + FLOAT_BASE + 10, ScalarType::Float));
                    nf += 1;
                }
                set
                // vec![]
            }
            InstrsType::Ret(re_type) => match re_type {
                // ScalarType::Float => vec![Reg::new(10 + FLOAT_BASE, ScalarType::Float)],
                // ScalarType::Int => vec![Reg::new(10, ScalarType::Int)],
                _ => vec![],
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

    pub fn set_call_type(&mut self, func_type: ScalarType) {
        self.func_type = func_type;
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
        if !(self.get_type() == InstrsType::Load || self.get_type() == InstrsType::Store) {
            panic!("only support load/store, inst: {:?}", self);
        }
        match self.operands[2] {
            Operand::IImm(offset) => offset,
            _ => unreachable!("only support imm sp offset"),
        }
    }

    // Branch, Jump, Call:
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
            _ => unreachable!("only support imm sp offset, {:?}", self),
        }
    }

    pub fn set_double(&mut self) {
        self.double = true;
    }
    pub fn is_double(&self) -> bool {
        self.double
    }

    pub fn set_float(&mut self) {
        self.float = true;
    }

    pub fn is_float(&self) -> bool {
        self.float
    }

    // LoadGlobal
    pub fn get_global_var_str(&self, origin: bool) -> String {
        if origin {
            match self.operands[1] {
                Operand::Addr(ref name) => name.clone(),
                _ => unreachable!("only support global var"),
            }
        } else {
            match self.operands[2] {
                Operand::Addr(ref name) => name.clone(),
                _ => unreachable!("only support global var"),
            }
        }
    }
}

///j,b,la等 类型指令 获取label
impl LIRInst {
    ///如果是b型/j型 指令 ,则能够获取块号,否则返回None
    pub fn get_bb_label(&self) -> Option<String> {
        match self.get_type() {
            InstrsType::Branch(_) | InstrsType::Jump => Some(self.get_label().drop_addr()),
            _ => None,
        }
    }

    ///如果是la指令,则能够获取全局地址label,否则返回None
    pub fn get_addr_label(&self) -> Option<String> {
        match self.get_type() {
            InstrsType::OpReg(SingleOp::LoadAddr) | InstrsType::Jump => {
                Some(self.get_label().drop_addr())
            }
            _ => None,
        }
    }
    ///如果是call指令,则能够获取函数名,否则返回None
    pub fn get_func_name(&self) -> Option<String> {
        match self.get_type() {
            InstrsType::Call => Some(self.get_label().drop_addr()),
            _ => None,
        }
    }
}
