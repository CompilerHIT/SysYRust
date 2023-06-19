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
use std::io::prelude::*;
use std::io::Result;

use super::block::NUM_SIZE;

/// Assembly builder.
pub struct AsmBuilder<'f> {
    f: &'f File,
}

impl<'f> AsmBuilder<'f> {
    /// Creates a new assembly builder.
    pub fn new(f: &'f mut File) -> Self {
        Self { f }
    }

    pub fn ret(&mut self) -> Result<()> {
        writeln!(self.f, "    ret")
    }

    pub fn op2(&mut self, op: &str, dest: &str, lhs: &str, rhs: &str, is_imm: bool, is_double: bool) -> Result<()> {
        //FIXME: mul使用w是否有超过32位的用例
        if is_imm {
            if op == "or" || op == "xor" || op == "and" || op == "slt" || is_double {
                writeln!(self.f, "    {op}i {dest}, {lhs}, {rhs}")
            } else {
                writeln!(self.f, "    {op}iw {dest}, {lhs}, {rhs}")
            }
        } else {
            if op == "or" || op == "xor" || op == "and" || op == "slt" || op == "mulhs" || is_double {
                writeln!(self.f, "    {op} {dest}, {lhs}, {rhs}")
            } else {
                writeln!(self.f, "    {op}w {dest}, {lhs}, {rhs}")
            }
        }
    }

    pub fn op1(&mut self, op: &str, dest: &str, src: &str) -> Result<()> {
        writeln!(self.f, "    {op} {dest}, {src}")
    }

    pub fn addi(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        writeln!(self.f, "    addi {dest}, {opr}, {imm}")
    }

    pub fn slli(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        writeln!(self.f, "    slli {dest}, {opr}, {imm}")
    }

    pub fn srai(&mut self, dest: &str, opr: &str, imm: i32) -> Result<()> {
        writeln!(self.f, "    srai {dest}, {opr}, {imm}")
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

    pub fn s(
        &mut self,
        src: &str,
        addr: &str,
        offset: i32,
        is_float: bool,
        is_double: bool,
    ) -> Result<()> {
        if !is_double {
            if is_float {
                writeln!(self.f, "	fsw {src}, {offset}({addr})")
            } else {
                writeln!(self.f, "	sw {src}, {offset}({addr})")
            }
        } else {
            if is_float {
                writeln!(self.f, "	fsd {src}, {offset}({addr})")
            } else {
                writeln!(self.f, "	sd {src}, {offset}({addr})")
            }
        }
    }

    pub fn l(
        &mut self,
        dest: &str,
        addr: &str,
        offset: i32,
        is_float: bool,
        is_double: bool,
    ) -> Result<()> {
        if !is_double {
            if is_float {
                writeln!(self.f, "	flw {dest}, {offset}({addr})")
            } else {
                writeln!(self.f, "	lw {dest}, {offset}({addr})")
            }
        } else {
            if is_float {
                writeln!(self.f, "	fld {dest}, {offset}({addr})")
            } else {
                writeln!(self.f, "	ld {dest}, {offset}({addr})")
            }
        }
    }

    pub fn b(&mut self, cond: &str, lhs: &str, rhs: &str, label: &str) -> Result<()> {
        writeln!(self.f, "    b{cond}    {lhs}, {rhs}, {label}")
    }

    pub fn bnez(&mut self, reg: &str, label: &str) -> Result<()> {
        writeln!(self.f, "    bnez {reg}, {label}")
    }

    pub fn j(&mut self, label: &str) -> Result<()> {
        writeln!(self.f, "	j {label}")
    }

    pub fn call(&mut self, func: &str) -> Result<()> {
        writeln!(self.f, "	call {func}")
    }

    pub fn show_func(&mut self, label: &str) -> Result<()> {
        writeln!(self.f, "	.text");
        writeln!(self.f, "	.align	1");
        writeln!(self.f, "	.globl	{label}");
        writeln!(self.f, "    .type {label}, @function");
        writeln!(self.f, "{label}:")
    }

    pub fn show_block(&mut self, label: &str) -> Result<()> {
        writeln!(self.f, "{label}:")
    }

    // pub fn load_global(&mut self, reg: &str, global_label: &str, block_label: &str) -> Result<()> {
    //     writeln!(self.f, "{block_label}");
    //     writeln!(self.f, "	auipc   {reg}, %pcrel_hi({global_label})")?;
    //     writeln!(self.f, "	addi    {reg}, {reg}, %pcrel_lo({block_label})")
    // }

    pub fn print_array(&mut self, array: &Vec<i32>, name: String, size: i32) -> Result<()> {
        writeln!(self.f, "{name}:")?;
        if array.len() == 0 {
            for i in 0..size {
                writeln!(self.f, "	.word	0")?;
            }
        }
        for i in array {
            writeln!(self.f, "	.word	{i}")?;
        }
        Ok(())
    }
}
