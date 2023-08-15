use super::{execute_stat::ExecuteStat, structs::*};

use crate::{
    backend::{
        instrs::{InstrsType, LIRInst, SingleOp, BB},
        operand::Reg,
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

        //给zero寄存器一个0值
        program_stat.reg_val.insert(Reg::get_zero(), Value::IImm(0));

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
            InstrsType::Binary(..) => {
                // let mut op_str = match op {
                //     BinaryOp::Add => "add",
                //     BinaryOp::Sub => "sub",
                //     BinaryOp::Mul => "mul",
                //     BinaryOp::Div => "div",
                //     BinaryOp::Rem => "rem",
                //     BinaryOp::And => "and",
                //     BinaryOp::Or => "or",
                //     BinaryOp::Xor => "xor",
                //     BinaryOp::Slt => "slt",
                //     BinaryOp::Shl => "sll",
                //     BinaryOp::Shr => "srl",
                //     BinaryOp::Sar => "sra",
                //     BinaryOp::FCmp(cmp) => match cmp {
                //         CmpOp::Eq => "eq",
                //         CmpOp::Ne => "ne",
                //         CmpOp::Lt => "lt",
                //         CmpOp::Le => "le",
                //         CmpOp::Gt => "gt",
                //         CmpOp::Ge => "ge",
                //         _ => unreachable!(),
                //     },
                // };
                // let mut is_imm = match op_str {
                //     "add" | "sub" | "and" | "or" | "xor" | "sll" | "srl" | "sra" | "slt" => true,
                //     _ => false,
                // };
                // let mut is_double = inst.is_double();
                // let def_reg = inst.get_dst().drop_reg();
                self.consume_calc(*inst);
            }
            InstrsType::OpReg(op) => match op {
                SingleOp::F2I
                | SingleOp::I2F
                | SingleOp::LoadFImm
                | SingleOp::Seqz
                | SingleOp::Snez
                | SingleOp::Neg => {
                    let def_reg = inst.get_def_reg();
                    if let Some(def_reg) = def_reg {
                        self.reg_val.insert(def_reg, Value::Inst(*inst));
                    }
                }
                SingleOp::LoadAddr => {
                    self.consume_la(inst);
                }
                SingleOp::Li => {
                    self.consume_li(inst);
                }
                SingleOp::Mv => {
                    self.consume_mv(inst);
                }
            },
            InstrsType::Load => {
                self.consume_load(inst);
            }
            InstrsType::Store => {
                self.consume_store(inst);
            }
            InstrsType::StoreToStack => {
                self.consume_store_to_stack(inst);
            }
            InstrsType::LoadFromStack => {
                self.consume_load_from_stack(inst);
            }
            InstrsType::LoadParamFromStack => {
                self.consume_load_param_from_stack(inst);
            }
            InstrsType::StoreParamToStack => {
                //该指令的偏移的介绍并不确定,所以不能够确定会store到栈上的什么区域
                // 但是作为传递参数使用的情况(不会影响到sp中非传参部分区域的值)
                //所以当前可以忽略该指令的影响
                self.consume_store_param_to_stack(inst);
            }
            // 判断！是否需要多插入一条j，间接跳转到
            InstrsType::Branch(_) => self.consume_branch(inst),
            InstrsType::Jump => {
                self.consume_jump(inst);
            }
            InstrsType::Call => {
                self.consume_call(inst);
            }
            InstrsType::Ret(..) => {
                //遇到返回指令(返回返回操作)
                self.consume_ret(inst);
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
