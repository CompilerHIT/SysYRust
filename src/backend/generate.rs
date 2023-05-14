use super::{instrs::*, operand::ImmBs};
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

impl GenerateAsm for OpReg {
    //TODO:
}

impl GenerateAsm for ChangeSp {
    fn generate(&self, _: Pointer<Context>, f: &mut std::fs::File) -> Result<()> {
        let mut builder = AsmBuilder::new(f, "");
        let imm = self.get_offset();
        builder.addi("sp", "sp", imm)?;
        Ok(())
    }
}

impl GenerateAsm for Load {
    //TODO:
}

impl GenerateAsm for Store {
    //TODO:
}

impl GenerateAsm for StackLoad {
    fn generate(&self, context: Pointer<Context>, f: &mut std::fs::File) -> Result<()> {
        let mut builder = AsmBuilder::new(f, "");
        if !self.get_offset().is_imm_12bs() {
            panic!("illegal offset");
        }
        let dst = self.get_dst();
        // let inst_off = self.get_offset().
        let offset = self.get_src().borrow().get_pos() + self.get_offset().get_data()
                            - context.borrow().get_offset();
        match dst.get_type() {
            ScalarType::Int => builder.ld(&dst.to_string(), "sp", offset, false)?,
            ScalarType::Float => builder.ld(&dst.to_string(), "sp", offset, true)?,
            _ => panic!("illegal type"),
        }
        Ok(())
    }
}

impl GenerateAsm for StackStore {
    fn generate(&self, context: Pointer<Context>, f: &mut std::fs::File) -> Result<()> {
        let mut builder = AsmBuilder::new(f, "");
        if !self.get_offset().is_imm_12bs() {
            panic!("illegal offset");
        }
        let src = self.get_src();
        let offset = self.get_dst().borrow().get_pos() + self.get_offset().get_data() 
                            - context.borrow().get_offset();
        match src.get_type() {
            ScalarType::Int => builder.sd("sp", &src.to_string(), offset, false)?,
            ScalarType::Float => builder.sd("sp", &src.to_string(), offset, true)?,
            _ => panic!("illegal type"),
        }
        Ok(())
    }
}

impl GenerateAsm for Call {
    //TODO:
}

impl GenerateAsm for Return {
    fn generate(&self, context: Pointer<Context>, f: &mut std::fs::File) -> Result<()> {
        context.borrow_mut().call_epilogue_event();
        let mut builder = AsmBuilder::new(f, "");
        builder.ret()?;
        Ok(())
    }
}