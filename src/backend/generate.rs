use super::instrs::*;
use crate::backend::operand::ToString;

impl GenerateAsm for Binary { 
    fn generate(&self, _: Pointer<Context>, f: &mut std::fs::File) -> Result<()> {
        let mut builder = AsmBuilder::new(f, "");
        let op = match self.get_op() {
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
        let dst = self.get_dst().to_string();
        
        let lhs = match self.get_lhs() {
            Operand::Reg(reg) => reg.to_string(),
            _ => panic!("lhs of binary op must be reg, to improve"),
        };
        let rhs = match self.get_rhs() {
            Operand::Reg(reg) => reg.to_string(),
            Operand::Addr(addr) => addr.to_string(),
            Operand::FImm(fimm) => fimm.to_string(),
            Operand::IImm(iimm) => iimm.to_string(),
        };
        builder.op2(op, &dst, &lhs, &rhs)?;
        Ok(())
    }
}

impl GenerateAsm for MvReg {
    //TODO:
}

impl GenerateAsm for Li {
    //TODO:
}

impl GenerateAsm for Lui {
    //TODO:
}

impl GenerateAsm for OpReg {
    //TODO:
}

impl GenerateAsm for Load {
    //TODO:
}

impl GenerateAsm for Store {
    //TODO:
}

impl GenerateAsm for Call {
    //TODO:
}