use std::collections::{HashMap, HashSet};

use crate::{
    backend::{
        instrs::{Func, InstrsType, SingleOp},
        operand::Reg,
        regalloc::structs::RegUsedStat,
    },
    frontend::preprocess,
};

use super::*;

///进行了寄存器合并的分配,在最后的最后进行
pub fn alloc_with_merge(func: &mut Func) {
    //首先对寄存器除了特殊寄存器以外的寄存器使用进行p2v
    //然后统计合并机会
    //然后重新分配,从小度开始合并
    //直到合无可合则结束合并
    //availables 为能够使用的寄存器
    let availables: HashSet<Reg> = { todo!() };
    let regs_to_decolor = Reg::get_all_recolorable_regs();
    let per_process = |func: &mut Func| -> bool {
        func.p2v_pre_handle_call(regs_to_decolor.clone());
        //分析有合并机会的寄存器对,
        let mut mergables: HashSet<(Reg, Reg)>;
        //分析约束

        //选择可合并寄存器中可分配寄存器数量最多的

        //尝试合并,直到某次合并成功

        return false;
    };
    while per_process(func) {
        //重分配结束后,尝试使用寄存器替换load store
    }

    //带寄存器合并的分配方式结束后,开始执行减少
}

//分析虚拟寄存器的合并机会
pub fn analyse_mergable(func: &Func) -> HashSet<(Reg, Reg)> {
    let mut mergables: HashSet<(Reg, Reg)> = HashSet::new();
    //分析可以合并的虚拟寄存器
    func.calc_live_base();
    for bb in func.blocks.iter() {
        Func::analyse_inst_with_live_now_backorder(
            *bb,
            &mut |inst, live_now| match inst.get_type() {
                InstrsType::OpReg(SingleOp::Mv) => {
                    let reg_use = inst.get_lhs().drop_reg();
                    let reg_def = inst.get_def_reg().unwrap();
                    if live_now.contains(&reg_use) {
                        return;
                    }
                    if reg_use.is_physic() || reg_use.is_physic() {
                        return;
                    }
                    mergables.insert((reg_use, *reg_def));
                    mergables.insert((*reg_def, reg_use));
                }
                _ => (),
            },
        );
    }
    return mergables;
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
