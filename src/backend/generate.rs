use super::{instrs::*, operand::ImmBs};
use crate::backend::operand::ToString;

impl GenerateAsm for LIRInst { 
    fn generate(&self, context: ObjPtr<Context>, f: &mut std::fs::File) -> Result<()> {
        let mut builder = AsmBuilder::new(f, "");
        match self.get_type() {
            InstrsType::Binary(op) => {
                let op = match op {
                    BinaryOp::Add => "add",
                    BinaryOp::Sub => "sub",
                    BinaryOp::Mul => "mul",
                    BinaryOp::Div => "div",
                    BinaryOp::Mod => "rem",
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
            InstrsType::OpReg(..) => {
                //TODO:
                Ok(())
            },
            InstrsType::ChangeSp => {
                let mut builder = AsmBuilder::new(f, "");
                let imm = self.get_change_sp_offset();
                builder.addi("sp", "sp", imm)?;
                Ok(())
            },
            InstrsType::Load => {
                //TODO:
                Ok(())
            },
            InstrsType::Store => {
                Ok(())
            },
            InstrsType::StoreToStack => {
                let mut builder = AsmBuilder::new(f, "");
                if !self.get_offset().is_imm_12bs() {
                    panic!("illegal offset");
                }
                let src = match self.get_lhs() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("src of store must be reg, to improve"),
                };
                let offset =  match self.get_dst() {
                    Operand::Addr(slot) => slot.as_ref().get_pos() + self.get_offset().get_data() 
                                    - context.as_ref().get_offset(),
                    _ => panic!("dst of store must be stackslot, to improve"),
                };
                match src.get_type() {
                    ScalarType::Int => builder.sd("sp", &src.to_string(), offset, false)?,
                    ScalarType::Float => builder.sd("sp", &src.to_string(), offset, true)?,
                    _ => panic!("illegal type"),
                }
                Ok(())
            },
            InstrsType::LoadFromStack => {
                let mut builder = AsmBuilder::new(f, "");
                if !self.get_offset().is_imm_12bs() {
                    panic!("illegal offset");
                }
                let dst = match self.get_dst() {
                    Operand::Reg(reg) => reg,
                    _ => panic!("dst of load must be reg, to improve"),
                };
                // let inst_off = self.get_offset().
                let offset = match self.get_lhs() {
                    Operand::Addr(slot) => slot.as_ref().get_pos() + self.get_offset().get_data() 
                                    - context.as_ref().get_offset(),
                    _ => panic!("src of load must be stackslot, to improve"),
                };
                match dst.get_type() {
                    ScalarType::Int => builder.ld(&dst.to_string(), "sp", offset, false)?,
                    ScalarType::Float => builder.ld(&dst.to_string(), "sp", offset, true)?,
                    _ => panic!("illegal type"),
                }
                Ok(())
            },
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
                context.as_ref().call_epilogue_event();
                let mut builder = AsmBuilder::new(f, "");
                builder.ret()?;
                Ok(())
            }
        }
        
    }
}