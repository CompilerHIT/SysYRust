use super::*;

//定义中转者
enum TmpHolder {
    Reg(Reg),
    StackOffset(i32),
}
// 把一个寄存器的值抛出到中转者手中

impl Func {
    pub fn handle_call_v2(
        &mut self,
        pool: &mut BackendPool,
        callees_used: &mut HashMap<String, HashSet<Reg>>,
        callers_used: &HashMap<String, HashSet<Reg>>,
        callees_be_saved: &HashMap<String, HashSet<Reg>>,
    ) {
        let mut available_tmp_regs: RegUsedStat = RegUsedStat::init_unavailable();
        if self.label != "main" {
            for reg in callees_used.get(self.label.as_str()).unwrap() {
                available_tmp_regs.release_reg(reg.get_color());
            }
            for reg in callers_used.get(self.label.as_str()).unwrap() {
                available_tmp_regs.release_reg(reg.get_color());
            }
        } else {
            for reg in Reg::get_all_not_specials() {
                available_tmp_regs.release_reg(reg.get_color());
            }
        }
        for reg in Reg::get_all_specials_with_s0() {
            available_tmp_regs.use_reg(reg.get_color());
        }
        //遇到使用了的callers_used寄存器,就要保存保存到栈上或者保存到一个临时可用寄存器中
        //当遇到了临时可用寄存器的使用者,或者遇到这个值要使用的时候才把这个寄存器的值归还回来
    }
}
