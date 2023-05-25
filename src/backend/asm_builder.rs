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
use std::io::Result;
use std::fs::write;

/// Assembly builder.
pub struct AsmBuilder {
    f: String,
}

impl AsmBuilder {
    /// Creates a new assembly builder.
    pub fn new(f:  String) -> Self {
        Self { f }
    }

    pub fn li(&mut self, dest: &str, imm: i32) -> Result<()> {
        write(&self.f, format!("  li {dest}, {imm}\n"))
    }

    pub fn la(&mut self, dest: &str, address: &str) -> Result<()> {
        write(&self.f, format!("  la {dest}, {address}\n"))
    }

    pub fn mv(&mut self, dest: &str, src: &str) -> Result<()> {
        if dest != src {
            write(&self.f, format!("  mv {dest}, {src}\n"))
        } else {
            Ok(())
        }
    }

    pub fn ret(&mut self) -> Result<()> {
        write(&self.f, "  ret\n")
    }

    pub fn bz(&mut self, act: &str, cond: &str, label: &str) -> Result<()> {
        write(&self.f, format!("  {act} {cond}, {label}\n"))
    }

    pub fn neg(&mut self, dest: &str, src: &str) -> Result<()> {
        write(&self.f, format!("  neg {dest}, {src}\n"))
    }

    pub fn op2(&mut self, op: &str, dest: &str, lhs: &str, rhs: &str) -> Result<()> {
        write(&self.f, format!("  {op} {dest}, {lhs}, {rhs}\n"))
    }

    pub fn op1(&mut self, op: &str, dest: &str, src: &str) -> Result<()> {
        write(&self.f, format!("  {op} {dest}, {src}\n"))
    }

    pub fn addi(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        write(&self.f, format!("  addi {dest}, {opr}, {imm}\n"))
        
    }

    pub fn slli(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        write(&self.f, format!("  slli {dest}, {opr}, {imm}"))
    }

    pub fn srai(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        write(&self.f, format!("  srai {dest}, {opr}, {imm}\n"))
    }

    //TODO: optimize mul and div
    // pub fn muli(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
    //     if imm == 0 {
    //         self.mv(dest, "x0")
    //     } else if imm > 0 && (imm & (imm - 1)) == 0 {
    //         let mut shift = 0;
    //         let mut imm = imm >> 1;
    //         while imm != 0 {
    //             shift += 1;
    //             imm >>= 1;
    //         }
    //         self.slli(dest, opr, shift)
    //     } else {
    //         self.li(self.temp, imm)?;
    //         self.op2("mul", dest, opr, self.temp)
    //     }
    // }

    // pub fn divi(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
    //     if imm == 0 {
    //         panic!("div by zero!");
    //     } else if imm > 0 && (imm & (imm - 1)) == 0 {
    //         let mut shift: i32 = 0;
    //         let mut imm = imm >> 1;
    //         while imm != 0 {
    //             shift += 1;
    //             imm >>= 1;
    //         }
    //         self.srai(dest, opr, shift)?;
    //         Ok(())
    //     } else {
    //         // let sign = if imm < 0 { -1 } else { 1 };
    //         // let imm = sign * imm;
    //         // let mut tmp1 = String::from(dest);
    //         // tmp1.push_str("_tmp");
    //         // let mut tmp2 = String::from(dest);
    //         // tmp2.push_str("_tmp2");
    //         // self.li(tmp1.as_str(), imm)?;
    //         // self.op2("mul", tmp2.as_str(), opr, tmp1.as_str())?;
    //         // self.srai(dest, tmp2.as_str(), 31)?;
    //         // self.op2("add", dest, dest, opr)?;
    //         // self.op2("sub", dest, dest, tmp1.as_str())?;
    //         self.li(self.temp, imm)?;
    //         self.op2("div", dest, opr, self.temp);
    //         Ok(())
    //     }
    // }

    pub fn s(&mut self, src: &str, addr: &str, offset: i32, is_float: bool, is_double: bool) -> Result<()> {
        if !is_double {
            if is_float {
                write(&self.f, format!("	fsw {src}, {offset}({addr})\n"))
            } else {
                write(&self.f, format!("	sw {src}, {offset}({addr})\n"))
            }
        } else {
            if is_float {
                write(&self.f, format!("	fsw {src}, {offset}({addr})\n"))
            } else {
                write(&self.f, format!("	sw {src}, {offset}({addr})\n"))
            }
        }
    }

    pub fn l(&mut self, dest: &str, addr: &str, offset: i32, is_float: bool, is_double: bool) -> Result<()> {
        if !is_double {
            if is_float {
                write(&self.f, format!("	flw {dest}, {offset}({addr})\n"))
            } else {
                write(&self.f, format!("	lw {dest}, {offset}({addr})\n"))
            }
        } else {
            if is_float {
                write(&self.f, format!("	fld {dest}, {offset}({addr})\n"))
            } else {
                write(&self.f, format!("	ld {dest}, {offset}({addr})\n"))
            }
        }
    }

    pub fn j(&mut self, label: &str) -> Result<()> {
        write(&self.f, format!("  j {label}\n"))
    }

    pub fn call(&mut self, func: &str) -> Result<()> {
        write(&self.f, format!("	call {func}\n"))
    }

    pub fn show_func(&mut self, label: &str) -> Result<()> {
        write(&self.f, format!("{label}:\n"))
    }

    pub fn load_global(&mut self, tmp_reg: &str, target_reg: &str, global_label: &str, block_label: &str) -> Result<()> {
        write(&self.f, format!("	auipc   {tmp_reg}, %pcrel_hi({global_label})\n"))?;
        write(&self.f, format!("	addi    {target_reg}, {tmp_reg}, %pcrel_lo{block_label}\n"))
    }
    //TODO: for function
    // pub fn prologue(&mut self, func_name: &str, info: &FunctionInfo) -> Result<()> {
    //     // declaration
    //     write(self.f, "  .text")?;
    //     write(self.f, "  .globl {}", &func_name[1..])?;
    //     write(self.f, "{}:", &func_name[1..])?;
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
    //     write(self.f, "  ret")
    // }
}
