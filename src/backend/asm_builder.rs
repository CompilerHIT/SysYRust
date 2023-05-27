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

/// Assembly builder.
pub struct AsmBuilder {

}

impl AsmBuilder {
    /// Creates a new assembly builder.
    pub fn new() -> Self {
        Self {
        }    
    }

    pub fn ret(&mut self){
        print!("  ret\n")
    }

    

    pub fn op2(&mut self, op: &str, dest: &str, lhs: &str, rhs: &str, is_imm: bool) {
        if is_imm {
            print!("  {op}i {dest}, {lhs}, {rhs}\n");
        } else {
            print!("  {op} {dest}, {lhs}, {rhs}\n");
        }
    }

    pub fn op1(&mut self, op: &str, dest: &str, src: &str, is_imm: bool) {
        if is_imm{
            print!("  {op}i {dest}, {src}\n");
        } else {
            print!("  {op} {dest}, {src}\n");
        }
    }

    pub fn addi(&mut self, dest: &str, opr: &str, imm: i32) {
        print!("  addi {dest}, {opr}, {imm}\n");
        
    }

    pub fn slli(&mut self, dest: &str, opr: &str, imm: i32) {
        print!("  slli {dest}, {opr}, {imm}");
    }

    pub fn srai(&mut self, dest: &str, opr: &str, imm: i32) {
        print!("  srai {dest}, {opr}, {imm}\n");
    }

    //TODO: optimize mul and div
    // pub fn muli(&mut self, dest: &str, opr: &str, imm: i32) {
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

    // pub fn divi(&mut self, dest: &str, opr: &str, imm: i32) {
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
    //         
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
    //         
    //     }
    // }

    pub fn s(&mut self, src: &str, addr: &str, offset: i32, is_float: bool, is_double: bool) {
        if !is_double {
            if is_float {
                print!("	fsw {src}, {offset}({addr})\n");
            } else {
                print!("	sw {src}, {offset}({addr})\n");
            }
        } else {
            if is_float {
                print!("	fsw {src}, {offset}({addr})\n");
            } else {
                print!("	sw {src}, {offset}({addr})\n");
            }
        }
    }

    pub fn l(&mut self, dest: &str, addr: &str, offset: i32, is_float: bool, is_double: bool) {
        if !is_double {
            if is_float {
                print!("	flw {dest}, {offset}({addr})\n");
            } else {
                print!("	lw {dest}, {offset}({addr})\n");
            }
        } else {
            if is_float {
                print!("	fld {dest}, {offset}({addr})\n");
            } else {
                print!("	ld {dest}, {offset}({addr})\n");
            }
        }
    }

    pub fn b(&mut self, cond: &str, lhs: &str, rhs: &str, label: &str) {
        print!("    {cond}    {lhs}, {rhs}, {label}\n");
    }

    pub fn j(&mut self, label: &str) {
        print!("	j {label}\n");
    }

    pub fn call(&mut self, func: &str) {
        print!("	call {func}\n");
    }

    pub fn show_func(&mut self, label: &str) {
        print!("{label}:\n");
    }

    pub fn load_global(&mut self, tmp_reg: &str, target_reg: &str, global_label: &str, block_label: &str) {
        print!("	auipc   {tmp_reg}, %pcrel_hi({global_label})\n");
        print!("	addi    {target_reg}, {tmp_reg}, %pcrel_lo{block_label}\n");
    }

    pub fn print_array(&mut self, array: &Vec<i32>, name: String) {
        print!(".{name}:\n");
        for i in array {
            print!("	.word	{i}\n");
        }
        
    }
    //TODO: for function
    // pub fn prologue(&mut self, func_name: &str, info: &FunctionInfo) {
    //     // declaration
    //     print!(self)?;
    //     print!(self {}", &func_name[1..])?;
    //     print!unc_name[1..])?;
    //     // prologue
    //     let offset = info.sp_offset() as i32;
    //     if offset != 0 {
    //         self.addi("sp", "sp", -offset)?;
    //         if !info.is_leaf() {
    //             self.sd("ra", "sp", offset - 8)?;
    //         }
    //     }
    //     
    // }

    // pub fn epilogue(&mut self, info: &FunctionInfo) {
    //     let offset = info.sp_offset() as i32;
    //     if offset != 0 {
    //         if !info.is_leaf() {
    //             self.ld("ra", "sp", offset - 8)?;
    //         }
    //         self.addi("sp", "sp", offset)?;
    //     }
    //     print!(self    // ;
}
