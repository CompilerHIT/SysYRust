use super::{instrs::*, operand, FILE_PATH};
use crate::backend::operand::ToString;
//FIXME: virtue id to real id
impl GenerateAsm for LIRInst { 
    fn generate(&self, context: ObjPtr<Context>, f: FILE_PATH) -> Result<()> {
        let mut builder = AsmBuilder::new(f.clone());
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
                };
                let dst = match self.get_dst(){
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("dst of binary op must be reg, to improve"),
                };
                let lhs = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("lhs of binary op must be reg, to improve"),
                };
                let rhs = match self.get_rhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    Operand::FImm(fimm) => fimm.to_string(),
                    Operand::IImm(iimm) => iimm.to_string(),
                    _ => panic!("rhs of binary op must be reg or imm, to improve"),
                };
                builder.op2(op, &dst, &lhs, &rhs)?;
                Ok(())
            },
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
                };
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg.to_string(),
                    _ => panic!("dst of single op must be reg, to improve"),
                };
                let src = match self.get_lhs() {
                    Operand::Reg(reg) => reg.to_string(),
                    Operand::IImm(iimm) => iimm.to_string(),
                    _ => panic!("src of single op must be reg or imm, to improve"),
                };
                builder.op1(op, &dst, &src)?;
                Ok(())
            },
            // InstrsType::ChangeSp => {
            //     let mut builder = AsmBuilder::new(f);
            //     let imm = self.get_change_sp_offset();
            //     builder.addi("sp", "sp", imm)?;
            //     Ok(())
            // },
            InstrsType::Load => {
                //FIXME: only call ld -- lw...to implement
                let mut builder = AsmBuilder::new(f.clone());
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
                    _ => panic!("src of load must be reg, to improve"),
                };
                
                builder.ld(&dst, &addr, offset.get_data(), false);
                Ok(())
            },
            InstrsType::Store => {
                //FIXME: only call sd -- sw...to implement
                let mut builder = AsmBuilder::new(f.clone());
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
                builder.sd(&src, &addr, offset.get_data(), false);
                Ok(())
            },

            InstrsType::StoreToStack => {
                let mut builder = AsmBuilder::new(f.clone());
                if !operand::is_imm_12bs(self.get_offset().get_data()) {
                    panic!("illegal offset");
                }
                let src = match self.get_lhs() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("src of store must be reg, to improve"),
                };
                let offset =  self.get_offset().get_data();
                match src.get_type() {
                    ScalarType::Int => builder.sd(&src.to_string(), "sp", offset, false)?,
                    ScalarType::Float => builder.sd(&src.to_string(), "sp", offset, true)?,
                    _ => panic!("illegal type"),
                }
                Ok(())
            },
            InstrsType::LoadFromStack | InstrsType::LoadParamFromStack => {
                let mut builder = AsmBuilder::new(f.clone());
                if !operand::is_imm_12bs(self.get_offset().get_data()) {
                    panic!("illegal offset");
                }
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("dst of load must be reg, to improve"),
                };
                // let inst_off = self.get_offset().
                let offset = self.get_offset().get_data();
                match dst.get_type() {
                    ScalarType::Int => builder.ld(&dst.to_string(), "sp", offset, false)?,
                    ScalarType::Float => builder.ld(&dst.to_string(), "sp", offset, true)?,
                    _ => panic!("illegal type"),
                }
                Ok(())
            },
            // 判断！是否需要多插入一条j，间接跳转到
            InstrsType::Branch(..) => {
                //TODO:
                Ok(())
            },
            InstrsType::Jump => {
                //TODO:
                Ok(())
            }
            InstrsType::Call => {
                //TODO:
                Ok(())
            },
            InstrsType::Ret(..) => {
                context.as_mut().call_epilogue_event();
                let mut builder = AsmBuilder::new(f.clone());
                builder.ret()?;
                Ok(())
            }
        }
        
    }
}