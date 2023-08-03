use std::collections::HashSet;

use crate::{
    backend::{instrs::Func, operand::Reg},
    frontend::preprocess,
};

use super::*;

///进行了寄存器合并的分配,在最后的最后进行
pub fn alloc_with_merge(func: &mut Func) {
    //首先对寄存器除了特殊寄存器以外的寄存器使用进行p2v
    //然后统计合并机会
    //然后重新分配,从小度开始合并
    //直到合无可合则结束合并
    let regs_to_decolor = Reg::get_all_recolorable_regs();
    let per_process = |func: &mut Func| -> bool {
        func.p2v_pre_handle_call(regs_to_decolor.clone());
        //分析有合并机会的寄存器对
        let mut mergables: HashSet<(Reg, Reg)>;

        return false;
    };
    while per_process(func) {}
}

//分析虚拟寄存器的合并机会
pub fn analyse_mergable(func: &Func) -> HashSet<(Reg, Reg)> {
    let mut mergables: HashSet<(Reg, Reg)>;
    todo!()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_let() {
        // let mut mm: HashSet<i32>;
        // assert!(mm.len() == 0);
    }
}
