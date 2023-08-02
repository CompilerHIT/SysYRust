use super::{execute_stat::ExecuteStat, structs::*};

use crate::{
    backend::{
        instrs::{BinaryOp, CmpOp, InstrsType, LIRInst, Operand, SingleOp, BB},
        operand::Reg,
        regalloc::structs::RegUsedStat,
    },
    utility::ObjPtr,
};
use std::collections::{HashMap, HashSet};

///程序资源状态
#[derive(Clone)]
pub struct ProgramStat {
    ///记录寄存器中的值
    pub reg_val: HashMap<Reg, Value>,
    ///记录内存区域中的值
    pub mem_val: HashMap<Value, Value>,
    ///记录上一条指令执行之后的执行状态
    pub execute_stat: ExecuteStat,
}

impl ProgramStat {
    ///初始化程序状态 (初始化的时候sp有个默认值,默认sp开了个数组)
    pub fn new() -> ProgramStat {
        let mut program_stat = ProgramStat {
            reg_val: HashMap::new(),
            mem_val: HashMap::new(),
            execute_stat: ExecuteStat::NextInst,
        };
        program_stat
            .reg_val
            .insert(Reg::get_sp(), Value::Addr(("sp_init".to_string(), 80000)));
        program_stat
    }

    //申请函数栈空间
    pub fn alloc_stack_mem(&mut self, size: i64) {
        //给sp-size
        let addr = self.reg_val.get_mut(&Reg::get_sp()).unwrap();
        let addr = match addr {
            Value::Addr(v) => v,
            _ => unreachable!(),
        };
        addr.1 -= size;
    }
    //释放函数栈空间
    pub fn release_stack_mem(&mut self, size: i64) {
        //给sp +size
        let addr = self.reg_val.get_mut(&Reg::get_sp()).unwrap();
        let addr = match addr {
            Value::Addr(v) => v,
            _ => unreachable!(),
        };
        addr.1 += size;
    }

    ///吞入一条指令,修改程序状态
    pub fn consume_inst(&mut self, inst: &ObjPtr<LIRInst>) -> ExecuteStat {
        match inst.get_type() {
            InstrsType::Binary(op) => {
                let mut op_str = match op {
                    BinaryOp::Add => "add",
                    BinaryOp::Sub => "sub",
                    BinaryOp::Mul => "mul",
                    BinaryOp::Div => "div",
                    BinaryOp::Rem => "rem",
                    BinaryOp::And => "and",
                    BinaryOp::Or => "or",
                    BinaryOp::Xor => "xor",
                    BinaryOp::Slt => "slt",
                    BinaryOp::Shl => "sll",
                    BinaryOp::Shr => "srl",
                    BinaryOp::Sar => "sra",
                    BinaryOp::FCmp(cmp) => match cmp {
                        CmpOp::Eq => "eq",
                        CmpOp::Ne => "ne",
                        CmpOp::Lt => "lt",
                        CmpOp::Le => "le",
                        CmpOp::Gt => "gt",
                        CmpOp::Ge => "ge",
                        _ => unreachable!(),
                    },
                };
                let mut is_imm = match op_str {
                    "add" | "sub" | "and" | "or" | "xor" | "sll" | "srl" | "sra" | "slt" => true,
                    _ => false,
                };
                let mut is_double = inst.is_double();
                let dst_reg = inst.get_dst().drop_reg();
                match op {
                    // BinaryOp::Add => {
                    //     let lhs = inst.get_lhs().drop_reg();
                    //     let lhs = self.reg_val.get(&lhs);
                    //     if lhs.is_none() {
                    //         self.reg_val.insert(dst_reg, Value::Inst(*inst));
                    //     } else {
                    //         let rhs = inst.get_rhs();
                    //         match rhs {
                    //             _ => unreachable!(),
                    //         }
                    //         todo!()
                    //     }
                    // }
                    // BinaryOp::Sub => {
                    //     //对减法的计算
                    //     todo!()
                    // }
                    _ => {
                        self.reg_val.insert(dst_reg, Value::Inst(*inst));
                    }
                }
            }
            InstrsType::OpReg(op) => match op {
                SingleOp::F2I
                | SingleOp::I2F
                | SingleOp::LoadAddr
                | SingleOp::LoadFImm
                | SingleOp::Neg
                | SingleOp::Seqz
                | SingleOp::Snez => {
                    let def_reg = inst.get_def_reg();
                    if let Some(def_reg) = def_reg {
                        self.reg_val.insert(*def_reg, Value::Inst(*inst));
                    }
                }
                SingleOp::LoadAddr => {
                    ///把label加载进来
                    let label = inst.get_addr_label().unwrap();
                    let l_reg = inst.get_lhs().drop_reg();
                    self.reg_val.insert(l_reg, Value::Addr((label, 0)));
                }
                SingleOp::Li => {
                    //加载一个立即数
                    let def_reg = *inst.get_def_reg().unwrap();
                    let src = inst.get_lhs();
                    match src {
                        Operand::IImm(iimm) => {
                            let iimm = iimm.get_data();
                            let iimm: i64 = iimm as i64;
                            self.reg_val.insert(def_reg, Value::IImm(iimm));
                        }
                        Operand::FImm(fimm) => {
                            let fimm = fimm.get_data() as f64;
                            self.reg_val
                                .insert(def_reg, Value::FImm(format!("{}", fimm).to_string()));
                        }
                        _ => unreachable!(),
                    }
                }
                SingleOp::Mv => {
                    let dst_reg = inst.get_dst().drop_reg();
                    let src_reg = inst.get_lhs().drop_reg();
                    let old_val = self.reg_val.get(&src_reg);
                    match old_val {
                        Some(old_val) => {
                            self.reg_val.insert(dst_reg, old_val.clone());
                        }
                        None => {
                            self.reg_val.insert(dst_reg, Value::Inst(*inst));
                        }
                    }
                }
            },
            InstrsType::Load => {
                //从内存位置加载一个值
                //首先要判断该位置有没有值(以及是否是从一个未知地址加载值)
                let dst_reg = inst.get_dst().drop_reg();
                let addr = inst.get_lhs().drop_reg();
                let addr = self.reg_val.get(&addr);
                if addr.is_none() {
                    // unreachable!();
                    //从未知地址取值,则取值用指令表示
                    self.reg_val.insert(dst_reg, Value::Inst(*inst));
                } else if addr.unwrap().get_type() != ValueType::Addr {
                    self.reg_val.insert(dst_reg, Value::Inst(*inst));
                    // unreachable!();
                } else {
                    let offset = inst.get_offset().get_data() as i64;
                    let mut addr = addr.unwrap().get_addr().unwrap().clone();
                    addr.1 += offset;
                    let addr = Value::Addr(addr);
                    let val = self.mem_val.get(&addr);
                    if val.is_none() {
                        self.reg_val.insert(dst_reg, Value::Inst(*inst));
                    } else if let Some(val) = val {
                        self.reg_val.insert(dst_reg, val.clone());
                    } else {
                        unreachable!();
                    }
                }
            }
            InstrsType::Store => {
                //把值写入内存某区域
                //从内存位置加载一个值
                //首先要判断该位置有没有值(以及是否是从一个未知地址加载值)
                let dst_reg = inst.get_dst().drop_reg();
                let addr = inst.get_lhs().drop_reg();
                let addr = self.reg_val.get(&addr);
                let val = self.reg_val.get(&dst_reg);
                if addr.is_none() {
                    //往未知地址写值,则清空所有地址记录,让所有记录呈未知
                    self.mem_val.clear();
                } else if addr.unwrap().get_type() != ValueType::Addr {
                    self.mem_val.clear();
                } else {
                    let offset = inst.get_offset().get_data() as i64;
                    let mut addr = addr.unwrap().get_addr().unwrap().clone();
                    addr.1 += offset;
                    let addr = Value::Addr(addr);
                    match val {
                        Some(val) => {
                            self.mem_val.insert(addr, val.clone());
                        }
                        None => {
                            self.mem_val.insert(addr, Value::Inst(*inst));
                        }
                    }
                }
            }
            InstrsType::StoreToStack => {
                let src_reg = inst.get_dst().drop_reg();
                let offset = inst.get_stack_offset().get_data();
                let addr = self.reg_val.get(&Reg::get_sp());
                let mut addr = addr.unwrap().get_addr().unwrap().clone();
                addr.1 += offset as i64;
                let val = self.reg_val.get(&src_reg);
                if val.is_none() {
                    //如果值未知,清除对应位置的值以表示未知
                    self.mem_val.remove(&Value::Addr(addr));
                } else {
                    self.mem_val.insert(Value::Addr(addr), val.unwrap().clone());
                }
            }
            InstrsType::LoadFromStack => {
                let dst_reg = inst.get_dst().drop_reg();
                let offset = inst.get_stack_offset().get_data();
                let mut addr = self
                    .reg_val
                    .get(&Reg::get_sp())
                    .unwrap()
                    .get_addr()
                    .unwrap()
                    .clone();
                addr.1 += offset as i64;
                let val = self.mem_val.get(&Value::Addr(addr));
                if val.is_none() {
                    self.reg_val.insert(dst_reg, Value::Inst(*inst));
                } else {
                    self.reg_val.insert(dst_reg, val.unwrap().clone());
                }
            }

            InstrsType::LoadParamFromStack => {
                let dst_reg = inst.get_dst().drop_reg();
                self.reg_val.insert(dst_reg, Value::Inst(*inst));
            }
            InstrsType::StoreParamToStack => {
                //该指令的偏移的介绍并不确定,所以不能够确定会store到栈上的什么区域
                // 但是作为传递参数使用的情况(不会影响到sp中非传参部分区域的值)
                //所以当前可以忽略该指令的影响
            }
            // 判断！是否需要多插入一条j，间接跳转到
            InstrsType::Branch(cond) => {
                let lhs = inst.get_lhs().drop_reg();
                let lhs = self.reg_val.get(&lhs);
                let bb_label = inst.get_bb_label().unwrap();
                if lhs.is_none() {
                    self.execute_stat = ExecuteStat::MayJump(vec![bb_label]);
                } else {
                    let lhs = lhs.unwrap();
                    let mut if_jump = false;
                    match cond {
                        CmpOp::Eqz => {
                            if let Value::IImm(val) = lhs {
                                if val == &0 {
                                    if_jump = true
                                }
                            }
                        }
                        _ => {
                            let rhs = inst.get_rhs().drop_reg();
                            let rhs = self.reg_val.get(&rhs);
                            if let Some(rhs) = rhs {
                                match cond {
                                    CmpOp::Eq => {
                                        if lhs == rhs {
                                            if_jump = true
                                        }
                                    }
                                    CmpOp::Ne => {
                                        if lhs != rhs {
                                            if_jump = true
                                        }
                                    }
                                    CmpOp::Lt => {
                                        if lhs < rhs {
                                            if_jump = true
                                        }
                                    }
                                    CmpOp::Le => {
                                        if lhs <= rhs {
                                            if_jump = true
                                        }
                                    }
                                    CmpOp::Gt => {
                                        if lhs > rhs {
                                            if_jump = true
                                        }
                                    }
                                    CmpOp::Ge => {
                                        if lhs >= rhs {
                                            if_jump = true
                                        }
                                    }
                                    _ => unreachable!(),
                                }
                            }
                        }
                    };
                    if if_jump {
                        self.execute_stat = ExecuteStat::Jump(bb_label);
                    } else {
                        self.execute_stat = ExecuteStat::MayJump(vec![bb_label]);
                    }
                }
            }
            InstrsType::Jump => {
                let label = inst.get_bb_label().unwrap();
                self.execute_stat = ExecuteStat::Jump(label);
            }
            InstrsType::Call => {
                //注意 , call对于 a0 寄存器的影响跟跳转关系有关,需要外部单独处理
                self.execute_stat = ExecuteStat::Call(inst.get_func_name().unwrap());
            }
            InstrsType::Ret(..) => {
                //遇到返回指令(返回返回操作)
                self.execute_stat = ExecuteStat::Ret;
            }
        }
        self.execute_stat.clone()
    }

    pub fn miss_certain_mem(&mut self, mem_base: &str) {
        let mut to_rm = HashSet::new();
        for (addr, _) in self.mem_val.iter() {
            let base_label = addr.get_addr().unwrap().0.clone();
            if base_label == mem_base {
                to_rm.insert(addr.clone());
            }
        }
        for to_rm in to_rm {
            self.mem_val.remove(&to_rm);
        }
    }

    ///吞入一个块,修改程序状态
    ///返回将要 跳转到的块/函数状态
    pub fn consume_block(&mut self, bb: &ObjPtr<BB>) -> ExecuteStat {
        if self.execute_stat != ExecuteStat::NextInst {
            return self.execute_stat.clone();
        }
        for inst in bb.insts.iter() {
            let execute_stat = self.consume_inst(inst);
            if execute_stat != ExecuteStat::NextInst {
                break;
            }
        }
        self.execute_stat.clone()
    }

    ///判断两个寄存器的值是否是相同的
    /// 如果两个寄存器的值相同,返回true
    /// 如果其中任何一个寄存器的值为未知,或者两个寄存器的值不同，返回false
    pub fn is_equal(&mut self, reg1: &Reg, reg2: &Reg) -> bool {
        if !self.reg_val.contains_key(reg1) || !self.reg_val.contains_key(reg2) {
            return false;
        }
        let v1 = self.reg_val.get(reg1).unwrap();
        let v2 = self.reg_val.get(reg2).unwrap();
        v1 == v2
    }
}

impl ProgramStat {
    pub fn get_val_from_reg(&self, reg: &Reg) -> Option<Value> {
        let val = self.reg_val.get(reg);
        match val {
            Some(val) => Some(val.clone()),
            None => None,
        }
    }
}
