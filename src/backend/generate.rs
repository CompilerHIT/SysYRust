use super::{instrs::*, operand};
use crate::backend::operand::ToString;
use std::fs::File;
//FIXME: virtue id to real id
impl GenerateAsm for LIRInst {
    fn generate(&mut self, context: ObjPtr<Context>, f: &mut File) -> Result<()> {
        let mut builder = AsmBuilder::new(f);
        match self.get_type() {
            InstrsType::Binary(op) => {
                let op = match op {
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
                    BinaryOp::Mulhs => "mulhs",
                };
                let mut is_imm = false;
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("dst of binary op must be reg, to improve"),
                };
                let lhs = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("lhs of binary op must be reg, to improve"),
                };
                let rhs = match self.get_rhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    Operand::FImm(fimm) => {
                        is_imm = true;
                        fimm.to_string()
                    }
                    Operand::IImm(iimm) => {
                        is_imm = true;
                        iimm.to_string()
                    }
                    _ => panic!("rhs of binary op must be reg or imm, to improve"),
                };
                builder.op2(op, &dst, &lhs, &rhs, is_imm)?;
                Ok(())
            }
            InstrsType::OpReg(op) => {
                let op = match op {
                    SingleOp::Li => "li",
                    SingleOp::Lui => "lui",
                    SingleOp::IMv => "mv",
                    SingleOp::FMv => "fmv",
                    SingleOp::INot => "not",
                    SingleOp::INeg => "neg",
                    SingleOp::FNot => "fnot",
                    SingleOp::FNeg => "fneg",
                    SingleOp::I2F => "fcvt.s.w",
                    SingleOp::F2I => "fcvt.w.s",
                    SingleOp::LoadAddr => "la",
                };
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("dst of single op must be reg, to improve"),
                };
                let src = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    Operand::IImm(iimm) => iimm.to_string(),
                    Operand::Addr(addr) => addr.to_string(),
                    _ => unreachable!("src of single op must be reg or imm, to improve"),
                };
                builder.op1(op, &dst, &src)?;
                Ok(())
            }
            // InstrsType::ChangeSp => {
            //     let mut builder = AsmBuilder::new(f);
            //     let imm = self.get_change_sp_offset();
            //     builder.addi("sp", "sp", imm)?;
            //     Ok(())
            // },
            InstrsType::Load => {
                let mut builder = AsmBuilder::new(f);
                let offset = self.get_offset();
                if !operand::is_imm_12bs(offset.get_data()) {
                    panic!("illegal offset");
                }
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("dst of load must be reg, to improve"),
                };
                let addr = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => { println!("load reg: {:?}", self.get_lhs());panic!("src of load must be reg, to improve");},
                };

                builder.l(&dst, &addr, offset.get_data(), self.is_float(), self.is_double())?;
                Ok(())
            }
            InstrsType::Store => {
                let mut builder = AsmBuilder::new(f);
                let offset = self.get_offset();
                if !operand::is_imm_12bs(offset.get_data()) {
                    panic!("illegal offset");
                }
                let src = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("src of store must be reg, to improve"),
                };
                let addr = match self.get_dst() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("dst of store must be reg, to improve"),
                };
                builder.s(&src, &addr, offset.get_data(), self.is_float(), self.is_double())?;
                Ok(())
            }

            InstrsType::StoreToStack => {
                let mut builder = AsmBuilder::new(f);
                if !operand::is_imm_12bs(self.get_stack_offset().get_data()) {
                    panic!("illegal offset");
                }
                let src = match self.get_lhs() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("src of store must be reg, to improve"),
                };
                let offset = self.get_stack_offset().get_data();
                //FIXME: 判断寄存器中存的是否是地址，如果只是简单的数值，则可以使用sw替代
                //FIXME: *4 or *8
                builder.s(&src.to_string(), "sp", offset, self.is_float(), self.is_double())?;
                Ok(())
            }
            InstrsType::LoadFromStack => {
                let mut builder = AsmBuilder::new(f);
                if !operand::is_imm_12bs(self.get_stack_offset().get_data()) {
                    panic!("illegal offset");
                }
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("dst of load must be reg, to improve"),
                };
                // let inst_off = self.get_offset().
                //FIXME: *4 or *8
                let offset = self.get_stack_offset().get_data();
                builder.l(&dst.to_string(), "sp", offset, self.is_float(), self.is_double())?;
                Ok(())
            }
            InstrsType::LoadParamFromStack => {
                let mut builder = AsmBuilder::new(f);

                let true_offset =
                    context.as_ref().get_offset() - self.get_stack_offset().get_data();
                if !operand::is_imm_12bs(true_offset) {
                    panic!("illegal offset");
                }
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("dst of load must be reg, to improve"),
                };

                builder.l(&dst.to_string(), "sp", true_offset, self.is_float(), self.is_double())?;
                Ok(())
            }

            InstrsType::StoreParamToStack => {
                let mut builder = AsmBuilder::new(f);
                let true_offset =
                    context.as_ref().get_offset() - self.get_stack_offset().get_data();
                if !operand::is_imm_12bs(true_offset) {
                    panic!("illegal offset");
                }
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("dst of load must be reg, to improve"),
                };

                builder.s(&dst.to_string(), "sp", true_offset, self.is_float(), self.is_double())?;
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
                };
                let lhs = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => unreachable!("branch block's lhs must be reg"),
                };
                let rhs = match self.get_rhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => unreachable!("branch block's rhs must be reg"),
                };
                builder.b(cond, &lhs, &rhs, &label)?;
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
            // InstrsType::LoadGlobal => {
            //     let mut builder = AsmBuilder::new(f);
            //     let dst = match self.get_dst() {
            //         Operand::Reg(reg) => reg,
            //         _ => panic!("dst of load must be reg, to improve"),
            //     };
            //     builder.load_global(
            //         &dst.to_string(),
            //         &self.get_global_var_str(true),
            //         &self.get_global_var_str(false),
            //     )?;
            //     Ok(())
            // }
        }
        //InstrsType::GenerateArray => {
        //TODO: generate array
        // .LC + {array_num}    .word {array_num} ...
        //   Ok(())
        //}
    }
}
