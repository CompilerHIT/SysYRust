use super::{instrs::*, operand::is_imm_12bs};
use crate::backend::operand::ToString;
use std::fs::File;
impl GenerateAsm for LIRInst {
    fn generate(&mut self, context: ObjPtr<Context>, f: &mut File) -> Result<()> {
        let mut builder = AsmBuilder::new(f);
        let row = context.is_row;
        match self.get_type() {
            InstrsType::Binary(op) => {
                let mut op = match op {
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
                    BinaryOp::FCmp(cmp) => {
                        match cmp {
                            CmpOp::Eq => "eq",
                            CmpOp::Ne => "ne",
                            CmpOp::Lt => "lt",
                            CmpOp::Le => "le",
                            CmpOp::Gt => "gt",
                            CmpOp::Ge => "ge",
                            _ => unreachable!()
                        }
                    }
                };
                let mut is_imm = match op {
                    "add" | "sub" | "and" | "or" | "xor" | "sll" | "srl" | "sra" | "slt" => true,
                    _ => false,
                };
                let mut is_double = self.is_double();
                let fop = &format!("f{}.s", op);
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            op = fop;
                            is_double = true;
                        }
                        reg.to_string(row)
                    }
                    _ => panic!("dst of binary op must be reg, to improve"),
                };
                let lhs = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(row),
                    _ => panic!("lhs of binary op must be reg, to improve"),
                };
                let rhs = match self.get_rhs() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            is_double = true;
                            op = fop;
                        }
                        is_imm = false;
                        reg.to_string(row)
                    }
                    Operand::FImm(fimm) => {
                        op = fop;
                        is_double = true;
                        fimm.to_string()
                    }
                    Operand::IImm(iimm) => iimm.to_string(),
                    _ => panic!("rhs of binary op must be reg or imm, to improve"),
                };
                builder.op2(op, &dst, &lhs, &rhs, is_imm, is_double)?;
                Ok(())
            }
            InstrsType::OpReg(op) => {
                let mut op = match op {
                    SingleOp::Li => "li",
                    SingleOp::IMv => "mv",
                    SingleOp::FMv => "fmv.s",
                    SingleOp::INeg => "neg",
                    SingleOp::FNeg => "fneg.s",
                    SingleOp::I2F => "fcvt.s.w",
                    SingleOp::F2I => "fcvt.w.s",
                    SingleOp::LoadAddr => "la",
                    SingleOp::Abs => "abs",
                    SingleOp::Seqz => "seqz",
                    SingleOp::Snez => "snez",
                    SingleOp::LoadFImm => "fmv.w.x",
                };
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg.to_string(row),
                    _ => panic!("dst of single op must be reg, to improve"),
                };
                let src = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(row),
                    Operand::IImm(iimm) => {
                        if is_imm_12bs(iimm.get_data()) && op == "li" {
                            op = "addiw";
                        }
                        iimm.to_string()
                    },
                    Operand::FImm(fimm) => fimm.to_string(),
                    Operand::Addr(addr) => addr.to_string(),
                };
                builder.op1(op, &dst, &src)?;
                Ok(())
            }
            InstrsType::Load => {
                let mut builder = AsmBuilder::new(f);
                let offset = self.get_offset();
                // if !operand::is_imm_12bs(offset.get_data()) {
                //     panic!("illegal offset");
                // }
                let mut is_float = self.is_float();
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            is_float = true;
                        }
                        reg.to_string(row)
                    }
                    _ => panic!("dst of load must be reg, to improve"),
                };
                let addr = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(row),
                    _ => {
                        panic!("src of load must be reg, to improve");
                    }
                };

                builder.l(&dst, &addr, offset.get_data(), is_float, self.is_double())?;
                Ok(())
            }
            InstrsType::Store => {
                let mut builder = AsmBuilder::new(f);
                let offset = self.get_offset();
                // if !operand::is_imm_12bs(offset.get_data()) {
                //     panic!("illegal offset, {:?}", self);
                // }
                let mut is_float = self.is_float();
                let src = match self.get_dst() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            is_float = true;
                        }

                        reg.to_string(row)
                    }
                    _ => panic!("src of store must be reg, to improve"),
                };
                let addr = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(row),
                    _ => panic!(
                        "dst of store must be reg, but is {:?} as Inst:{:?}",
                        self.get_dst(),
                        self
                    ),
                };
                builder.s(&src, &addr, offset.get_data(), is_float, self.is_double())?;
                Ok(())
            }

            InstrsType::StoreToStack => {
                let mut builder = AsmBuilder::new(f);
                // if !operand::is_imm_12bs(self.get_stack_offset().get_data()) {
                //     panic!("illegal offset");
                // }
                let mut is_float = self.is_float();
                let src = match self.get_dst() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            is_float = true;
                        }
                        reg
                    }
                    _ => panic!("src of store must be reg, to improve"),
                };
                let offset = self.get_stack_offset().get_data();
                //FIXME: 判断寄存器中存的是否是地址，如果只是简单的数值，则可以使用sw替代
                //FIXME: *4 or *8
                builder.s(
                    &src.to_string(row),
                    "sp",
                    offset,
                    is_float,
                    self.is_double(),
                )?;
                Ok(())
            }
            InstrsType::LoadFromStack => {
                let mut builder = AsmBuilder::new(f);
                // if !operand::is_imm_12bs(self.get_stack_offset().get_data()) {
                //     panic!("illegal offset");
                // }
                let mut is_float = self.is_float();
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            is_float = true;
                        }

                        reg
                    }
                    _ => panic!("dst of load must be reg, to improve"),
                };
                // let inst_off = self.get_offset().
                //FIXME: *4 or *8
                let offset = self.get_stack_offset().get_data();
                builder.l(
                    &dst.to_string(row),
                    "sp",
                    offset,
                    is_float,
                    self.is_double(),
                )?;
                Ok(())
            }
            InstrsType::LoadParamFromStack => {
                let mut builder = AsmBuilder::new(f);

                let true_offset =
                    context.as_ref().get_offset() - self.get_stack_offset().get_data();
                // if !operand::is_imm_12bs(true_offset) {
                //     panic!("illegal offset");
                // }
                let mut is_float = self.is_float();
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            is_float = true;
                        }

                        reg
                    }
                    _ => panic!("dst of load must be reg, to improve"),
                };

                builder.l(
                    &dst.to_string(row),
                    "sp",
                    true_offset,
                    is_float,
                    self.is_double(),
                )?;
                Ok(())
            }

            InstrsType::StoreParamToStack => {
                let mut builder = AsmBuilder::new(f);
                let true_offset =
                    context.as_ref().get_offset() - self.get_stack_offset().get_data();
                // if !operand::is_imm_12bs(true_offset) {
                //     panic!("illegal offset");
                // }
                let mut is_float = self.is_float();
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => {
                        if reg.get_type() == ScalarType::Float {
                            is_float = true;
                        }
                        reg
                    }
                    _ => panic!("dst of load must be reg, to improve"),
                };

                builder.s(
                    &dst.to_string(row),
                    "sp",
                    true_offset,
                    is_float,
                    self.is_double(),
                )?;
                Ok(())
            }
            // 判断！是否需要多插入一条j，间接跳转到
            InstrsType::Branch(cond) => {
                let mut builder = AsmBuilder::new(f);
                let label = match self.get_label() {
                    Operand::Addr(label) => label.to_string(),
                    _ => unreachable!("branch block's label must be string"),
                };
                let cond = match cond {
                    CmpOp::Eq => "eq",
                    CmpOp::Ne => "ne",
                    CmpOp::Lt => "lt",
                    CmpOp::Le => "le",
                    CmpOp::Gt => "gt",
                    CmpOp::Ge => "ge",
                    CmpOp::Nez => "nez",
                };
                let lhs = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(row),
                    _ => unreachable!("branch block's lhs must be reg"),
                };
                if cond != "nez" {
                    let rhs = match self.get_rhs() {
                        Operand::Reg(reg) => reg.to_string(row),
                        _ => unreachable!("branch block's rhs must be reg"),
                    };
                    builder.b(cond, &lhs, &rhs, &label)?;
                } else {
                    builder.bnez(&lhs, &label)?;
                }
                Ok(())
            }
            InstrsType::Jump => {
                let mut builder = AsmBuilder::new(f);
                let label = match self.get_label() {
                    Operand::Addr(label) => label.to_string(),
                    _ => unreachable!("jump block's label must be string"),
                };
                builder.j(&label)?;
                Ok(())
            }
            InstrsType::Call => {
                //TODO:
                let mut builder = AsmBuilder::new(f);
                let func_name = match self.get_label() {
                    Operand::Addr(label) => label.to_string(),
                    _ => unreachable!("call block's label must be string"),
                };
                builder.call(&func_name)?;
                Ok(())
            }
            InstrsType::Ret(..) => {
                context.as_mut().call_epilogue_event();
                let mut builder = AsmBuilder::new(f);
                builder.ret()?;
                Ok(())
            }
        }
    }
}
