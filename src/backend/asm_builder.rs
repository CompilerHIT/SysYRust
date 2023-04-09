/* TODO:
    code generator，将lir(tac)转为汇编形式代码
    实现指令： 
            伪指令：li, la, mv, bz, j, ret, neg, call ...
            op2(2 operand), op1(1 operand),
            ld, sd, addi,
            slli: 优化muli, srli: 优化divi,
            bz: beqz, bnez, blez, bgez, bltz, bgtz
            j
    注：只支持生成lw、sw指令，因为sysy只有int与float型而没有long型 
        =>> 很粗暴的对l/s指令使用ld, sd
*/

// FIXME: divi使用srli替代
// FIXME: 是否使用addiw而非addi
// use super::func::FunctionInfo;
use std::fs::File;
use std::io::{Result, Write};

/// Assembly builder.
pub struct AsmBuilder<'f> {
    f: &'f mut File,
    temp: &'static str,
}

impl<'f> AsmBuilder<'f> {
    /// Creates a new assembly builder.
    pub fn new(f: &'f mut File, temp: &'static str) -> Self {
        Self { f, temp }
    }

    pub fn li(&mut self, dest: &str, imm: i32) -> Result<()> {
        writeln!(self.f, "  li {dest}, {imm}")
    }

    pub fn la(&mut self, dest: &str, address: &str) -> Result<()> {
        writeln!(self.f, "  la {dest}, {address}")
    }

    pub fn mv(&mut self, dest: &str, src: &str) -> Result<()> {
        if dest != src {
            writeln!(self.f, "  mv {dest}, {src}")
        } else {
            Ok(())
        }
    }

    pub fn ret(&mut self) -> Result<()> {
        writeln!(self.f, "  ret")
    }

    pub fn bz(&mut self, act: &str, cond: &str, label: &str) -> Result<()> {
        writeln!(self.f, "  {act} {cond}, {label}")
    }

    pub fn neg(&mut self, dest: &str, src: &str) -> Result<()> {
        writeln!(self.f, "  neg {dest}, {src}")
    }

    pub fn op2(&mut self, op: &str, dest: &str, lhs: &str, rhs: &str) -> Result<()> {
        writeln!(self.f, "  {op} {dest}, {lhs}, {rhs}")
    }

    pub fn op1(&mut self, op: &str, dest: &str, src: &str) -> Result<()> {
        writeln!(self.f, "  {op} {dest}, {src}")
    }

    pub fn addi(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        if (-2048..=2047).contains(&imm) {
            writeln!(self.f, "  addi {dest}, {opr}, {imm}")
        } else {
            self.li(self.temp, imm)?;
            writeln!(self.f, "  add {dest}, {opr}, {}", self.temp)
        }
    }

    pub fn slli(&mut self, dest: &str, opr: &str, imm: usize) -> Result<()> {
        writeln!(self.f, "  slli {dest}, {opr}, {imm}")
    }

    pub fn srai(&mut self, dest: &str, opr: &str, imm: usize) -> Result<()> {
        writeln!(self.f, "  srai {dest}, {opr}, {imm}")
    }

    //TODO: optimize mul and div
    pub fn muli(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        if imm == 0 {
            self.mv(dest, "x0")
        } else if imm > 0 && (imm & (imm - 1)) == 0 {
            let mut shift = 0;
            let mut imm = imm >> 1;
            while imm != 0 {
                shift += 1;
                imm >>= 1;
            }
            self.slli(dest, opr, shift)
        } else {
            self.li(self.temp, imm)?;
            self.op2("mul", dest, opr, self.temp)
        }
    }

    pub fn divi(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        if imm == 0 {
            panic!("div by zero!");
        } else if imm > 0 && (imm & (imm - 1)) == 0 {
            let mut shift: usize = 0;
            let mut imm = imm >> 1;
            while imm != 0 {
                shift += 1;
                imm >>= 1;
            }
            self.srai(dest, opr, shift)?;
            Ok(())
        } else {
            // let sign = if imm < 0 { -1 } else { 1 };
            // let imm = sign * imm;
            // let mut tmp1 = String::from(dest);
            // tmp1.push_str("_tmp");
            // let mut tmp2 = String::from(dest);
            // tmp2.push_str("_tmp2");
            // self.li(tmp1.as_str(), imm)?;
            // self.op2("mul", tmp2.as_str(), opr, tmp1.as_str())?;
            // self.srai(dest, tmp2.as_str(), 31)?;
            // self.op2("add", dest, dest, opr)?;
            // self.op2("sub", dest, dest, tmp1.as_str())?;
            self.li(self.temp, imm)?;
            self.op2("div", dest, opr, self.temp);
            Ok(())
        }
    }

    pub fn sd(&mut self, src: &str, addr: &str, offset: i32) -> Result<()> {
        if (-2048..=2047).contains(&offset) {
            writeln!(self.f, "  sd {src}, {offset}({addr})")
        } else {
            self.addi(self.temp, addr, offset)?;
            writeln!(self.f, "  sd {src}, 0({})", self.temp)
        }
    }

    pub fn ld(&mut self, dest: &str, addr: &str, offset: i32) -> Result<()> {
        if (-2048..=2047).contains(&offset) {
            writeln!(self.f, "  ld {dest}, {offset}({addr})")
        } else {
            self.addi(self.temp, addr, offset)?;
            writeln!(self.f, "  ld {dest}, 0({})", self.temp)
        }
    }

    pub fn j(&mut self, label: &str) -> Result<()> {
        writeln!(self.f, "  j {label}")
    }

    pub fn call(&mut self, func: &str) -> Result<()> {
        writeln!(self.f, "  call {func}")
    }

    //TODO: for function
    // pub fn prologue(&mut self, func_name: &str, info: &FunctionInfo) -> Result<()> {
    //     // declaration
    //     writeln!(self.f, "  .text")?;
    //     writeln!(self.f, "  .globl {}", &func_name[1..])?;
    //     writeln!(self.f, "{}:", &func_name[1..])?;
    //     // prologue
    //     let offset = info.sp_offset() as i32;
    //     if offset != 0 {
    //         self.addi("sp", "sp", -offset)?;
    //         if !info.is_leaf() {
    //             self.sd("ra", "sp", offset - 8)?;
    //         }
    //     }
    //     Ok(())
    // }

    // pub fn epilogue(&mut self, info: &FunctionInfo) -> Result<()> {
    //     let offset = info.sp_offset() as i32;
    //     if offset != 0 {
    //         if !info.is_leaf() {
    //             self.ld("ra", "sp", offset - 8)?;
    //         }
    //         self.addi("sp", "sp", offset)?;
    //     }
    //     writeln!(self.f, "  ret")
    // }
}
