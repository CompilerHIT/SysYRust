// 用于debug,模拟对寄存器分配结果的使用

use std::{fmt::format, fs::File};

use crate::{
    backend::{block, instrs::Func, operand::Reg},
    log_file,
};

use super::structs::FuncAllocStat;

pub struct Simulator {}

impl Simulator {
    // 模拟
    pub fn simulate(func: &Func, alloc_stat: &FuncAllocStat) {
        let file = "./logs/simulate.txt";
        // 遍历func
        for block in func.blocks.iter() {
            log_file!(file, "\n{}", block.label);
            log_file!(
                file,
                "live in:{:?}",
                block
                    .live_in
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<Vec<i32>>()
            );
            log_file!(
                file,
                "live out:{:?}",
                block
                    .live_out
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<Vec<i32>>()
            );
            for inst in block.insts.iter() {
                let mut reg_def = Vec::new();
                let mut reg_use = Vec::new();
                inst.get_reg_def().iter().for_each(|reg: &Reg| {
                    if !reg.is_virtual() {
                        reg_def.push(format!("{}", reg.get_id()));
                    } else {
                        if alloc_stat.spillings.contains(&reg.get_id()) {
                            reg_def.push(format!("{}(spill)", reg.get_id()));
                        } else {
                            reg_def.push(format!(
                                "{}({})",
                                reg.get_id(),
                                alloc_stat.dstr.get(&reg.get_id()).unwrap()
                            ));
                        }
                    }
                });
                inst.get_reg_use().iter().for_each(|reg: &Reg| {
                    if !reg.is_virtual() {
                        reg_use.push(format!("{}", reg.get_id()));
                    } else {
                        if alloc_stat.spillings.contains(&reg.get_id()) {
                            reg_use.push(format!("{}(spill)", reg.get_id()));
                        } else {
                            reg_use.push(format!(
                                "{}({})",
                                reg.get_id(),
                                alloc_stat.dstr.get(&reg.get_id()).unwrap()
                            ));
                        }
                    }
                });
                log_file!(
                    file,
                    "{:?} def:{:?} use:{:?}",
                    inst.get_type(),
                    reg_def,
                    reg_use
                );
            }
        }
    }
}
