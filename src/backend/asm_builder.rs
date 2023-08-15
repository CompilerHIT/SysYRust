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

    pub fn ret(&mut self) {
        writeln!(self.f, "    ret").unwrap()
    }

    pub fn op2(
        &mut self,
        op: &str,
        dest: &str,
        lhs: &str,
        rhs: &str,
        is_imm: bool,
        is_double: bool,
    ) {
        match op {
            "fne.s" => {
                writeln!(self.f, "    feq.s {dest}, {lhs}, {rhs}").unwrap();
                writeln!(self.f, "    xori {dest}, {dest}, 1").unwrap()
            }
            "fgt.s" => {
                writeln!(self.f, "    fle.s {dest}, {lhs}, {rhs}").unwrap();
                writeln!(self.f, "    xori {dest}, {dest}, 1").unwrap()
            }
            "fge.s" => {
                writeln!(self.f, "    flt.s {dest}, {rhs}, {lhs}").unwrap();
                writeln!(self.f, "    xori {dest}, {dest}, 1").unwrap()
            }
            _ => {
                if is_imm {
                    if op == "or" || op == "xor" || op == "and" || op == "slt" || is_double {
                        writeln!(self.f, "    {op}i {dest}, {lhs}, {rhs}").unwrap()
                    } else {
                        writeln!(self.f, "    {op}iw {dest}, {lhs}, {rhs}").unwrap()
                    }
                } else {
                    if op == "or"
                        || op == "xor"
                        || op == "and"
                        || op == "slt"
                        || op == "sh2add"
                        || is_double
                    {
                        writeln!(self.f, "    {op} {dest}, {lhs}, {rhs}").unwrap()
                    } else {
                        writeln!(self.f, "    {op}w {dest}, {lhs}, {rhs}").unwrap()
                    }
                }
            }
        }
    }

    pub fn op1(&mut self, op: &str, dest: &str, src: &str) {
        //FIXME: 为何一定使用rtz？
        if op == "fcvt.w.s" {
            writeln!(self.f, "    {op} {dest}, {src}, rtz").unwrap()
        } else {
            if op == "addiw" {
                writeln!(self.f, "    addiw {dest}, zero, {src}").unwrap()
            } else {
                writeln!(self.f, "    {op} {dest}, {src}").unwrap()
            }
        }
    }

    pub fn addi(&mut self, dest: &str, opr: &str, imm: i32) {
        writeln!(self.f, "    addi {dest}, {opr}, {imm}").unwrap();
    }

    pub fn s(&mut self, src: &str, addr: &str, offset: i32, is_float: bool, is_double: bool) {
        if is_float {
            writeln!(self.f, "	fsw {src}, {offset}({addr})").unwrap();
        } else {
            if !is_double {
                writeln!(self.f, "	sw {src}, {offset}({addr})").unwrap();
            } else {
                writeln!(self.f, "	sd {src}, {offset}({addr})").unwrap();
            }
        }
    }

    pub fn l(&mut self, dest: &str, addr: &str, offset: i32, is_float: bool, is_double: bool) {
        if is_float {
            writeln!(self.f, "	flw {dest}, {offset}({addr})").unwrap();
        } else {
            if !is_double {
                writeln!(self.f, "	lw {dest}, {offset}({addr})").unwrap();
            } else {
                writeln!(self.f, "	ld {dest}, {offset}({addr})").unwrap();
            }
        }
    }

    pub fn b(&mut self, cond: &str, lhs: &str, rhs: &str, label: &str) {
        writeln!(self.f, "    b{cond}    {lhs}, {rhs}, {label}").unwrap();
    }

    pub fn beqz(&mut self, reg: &str, label: &str) {
        writeln!(self.f, "    beqz {reg}, {label}").unwrap();
    }

    pub fn bnez(&mut self, reg: &str, label: &str) {
        writeln!(self.f, "    bnez {reg}, {label}").unwrap();
    }

    pub fn j(&mut self, label: &str) {
        writeln!(self.f, "	j {label}").unwrap()
    }

    pub fn call(&mut self, func: &str) {
        writeln!(self.f, "	call {func}").unwrap()
    }

    pub fn show_func(&mut self, label: &str) {
        writeln!(self.f, "	.text").unwrap();
        writeln!(self.f, "	.align	1").unwrap();
        writeln!(self.f, "	.globl	{label}").unwrap();
        writeln!(self.f, "    .type {label}, @function").unwrap();
        writeln!(self.f, "{label}:").unwrap();
    }

    pub fn show_block(&mut self, label: &str) {
        writeln!(self.f, "{label}:").unwrap()
    }

    pub fn print_array(&mut self, array: &Vec<i32>, name: String, size: i32) {
        writeln!(self.f, "{name}:").unwrap();
        for i in array {
            writeln!(self.f, "	.word	{i}").unwrap();
        }
        let zeros = size - array.len() as i32;
        if zeros > 0 {
            writeln!(self.f, "	.zero	{n}", n = zeros * NUM_SIZE).unwrap();
        }
    }

    pub fn print_farray(&mut self, array: &Vec<f32>, name: String, size: i32) {
        writeln!(self.f, "{name}:").unwrap();
        for i in array {
            writeln!(self.f, "	.word	{i}").unwrap();
        }
        let zeros = size - array.len() as i32;
        if zeros > 0 {
            writeln!(self.f, "	.zero	{n}", n = zeros * NUM_SIZE).unwrap();
        }
    }
}
