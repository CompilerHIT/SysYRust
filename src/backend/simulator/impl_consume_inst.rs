use crate::{
    backend::{
        instrs::{CmpOp, InstrsType, LIRInst, Operand, SingleOp},
        operand::Reg,
    },
    ir::instruction::Inst,
    utility::ScalarType,
};

use super::{
    execute_stat::ExecuteStat,
    structs::{Value, ValueType},
    *,
};

///简单值传递 , mv, li,la等
impl ProgramStat {
    pub fn consume_mv(&mut self, inst: &ObjPtr<LIRInst>) {
        debug_assert!(inst.get_type() == InstrsType::OpReg(crate::backend::instrs::SingleOp::Mv));
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

    pub fn consume_li(&mut self, inst: &ObjPtr<LIRInst>) {
        debug_assert!(inst.get_type() == InstrsType::OpReg(crate::backend::instrs::SingleOp::Li));
        //加载一个立即数
        let def_reg = inst.get_def_reg().unwrap();
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
            //用来做除法优化的情况不考虑
            Operand::Addr(_) => {
                self.reg_val.insert(def_reg, Value::Inst(*inst));
            }
            _ => unreachable!(),
        }
    }

    pub fn consume_la(&mut self, inst: &ObjPtr<LIRInst>) {
        debug_assert!(inst.get_type() == InstrsType::OpReg(SingleOp::LoadAddr));
        //把label加载进来
        let label = inst.get_lhs().drop_addr();
        let l_reg = inst.get_dst().drop_reg();
        self.reg_val.insert(l_reg, Value::Addr((label, 0)));
    }
}

///关于内存的访问,
impl ProgramStat {
    pub fn consume_load(&mut self, inst: &ObjPtr<LIRInst>) {
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

    pub fn consume_store(&mut self, inst: &ObjPtr<LIRInst>) {
        debug_assert!(inst.get_type() == InstrsType::Store);
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

    pub fn consume_store_to_stack(&mut self, inst: &ObjPtr<LIRInst>) {
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

    pub fn consume_load_from_stack(&mut self, inst: &ObjPtr<LIRInst>) {
        debug_assert!(inst.get_type() == InstrsType::LoadFromStack);
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

    //对这个没法分析
    pub fn consume_load_param_from_stack(&mut self, inst: &ObjPtr<LIRInst>) {
        let dst_reg = inst.get_dst().drop_reg();
        self.reg_val.insert(dst_reg, Value::Inst(*inst));
    }
    //对这个没法分析
    pub fn consume_store_param_to_stack(&mut self, inst: &ObjPtr<LIRInst>) {}
}

///关于执行流的变化:call,branch,jump等
impl ProgramStat {
    pub fn consume_call(&mut self, inst: &ObjPtr<LIRInst>) {
        //注意 , call对于 a0 寄存器的影响跟跳转关系有关,需要外部单独处理
        //TODO,暂时无条件认为call指令会清空已有值关系和内存关系

        debug_assert!(inst.get_type() == InstrsType::Call);
        //交给外部考虑调用call时造成的寄存器原值变化
        for reg in Reg::get_all_not_specials() {
            self.reg_val.remove(&reg);
        }
        if let Some(def_reg) = inst.get_def_reg() {
            self.reg_val.insert(def_reg, Value::Inst(*inst));
        }
        //不去分析函数调用对于内存空间的影响,认为调用前后所有内存空间原值都失效
        self.mem_val.clear();
        self.execute_stat = ExecuteStat::Call(inst.get_func_name().unwrap());
    }
    pub fn consume_ret(&mut self, inst: &ObjPtr<LIRInst>) {
        self.execute_stat = ExecuteStat::Ret;
    }

    pub fn consume_jump(&mut self, inst: &ObjPtr<LIRInst>) {
        let label = inst.get_bb_label().unwrap();
        self.execute_stat = ExecuteStat::Jump(label);
    }

    pub fn consume_branch(&mut self, inst: &ObjPtr<LIRInst>) {
        if let InstrsType::Branch(cond) = inst.get_type() {
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
        } else {
            unreachable!();
        }
    }
}

///关于计算
impl ProgramStat {
    pub fn consume_add(&mut self, inst: &ObjPtr<LIRInst>) {
        debug_assert!(inst.get_type() == InstrsType::Binary(crate::backend::instrs::BinaryOp::Add));
        let def_reg = inst.get_def_reg().unwrap();
        let lhs = inst.get_lhs().drop_reg();
        let rhs = inst.get_rhs();
        let lhs = self.reg_val.get(&lhs);
        if let Some(l_val) = lhs {
            match rhs {
                Operand::Reg(rhs) => {
                    if let Some(r_val) = self.reg_val.get(rhs) {
                        let new_v = Value::add(r_val, l_val);
                        if new_v.is_none() {
                            self.reg_val.insert(def_reg, Value::Inst(*inst));
                        } else {
                            self.reg_val.insert(def_reg, new_v.unwrap());
                        }
                    } else {
                        self.reg_val.insert(def_reg, Value::Inst(*inst));
                    }
                }
                Operand::IImm(rhs) => {
                    let r_val = Value::IImm(rhs.get_data() as i64);
                    let new_v = Value::add(l_val, &r_val);
                    if new_v.is_none() {
                        self.reg_val.insert(def_reg, Value::Inst(*inst));
                    } else {
                        self.reg_val.insert(def_reg, new_v.unwrap());
                    }
                }
                _ => {
                    self.reg_val.insert(def_reg, Value::Inst(*inst));
                }
            }
        } else {
            self.reg_val.insert(def_reg, Value::Inst(*inst));
        }
    }
}
