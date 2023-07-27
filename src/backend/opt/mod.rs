use std::collections::HashMap;

pub use crate::backend::block::BB;
use crate::backend::instrs::*;
use crate::backend::module::AsmModule;
use crate::backend::operand;
use crate::backend::operand::*;
use crate::backend::BackendPool;
use crate::log;
pub use crate::utility::ObjPtr;

mod block_pass;
mod clear_pass;
mod peephole_pass;

pub struct BackendPass {
    pub module: ObjPtr<AsmModule>,
}

impl BackendPass {
    pub fn new(module: ObjPtr<AsmModule>) -> Self {
        Self { module }
    }

    pub fn run_pass(&mut self, pool: &mut BackendPool) {
        self.block_pass_pre_clear(pool);
        self.clear_pass(pool);
        // 清除无用指令之后开始栈空间重排
        // self.rearrange_stack_slot();
        self.block_pass();
        self.peephole_pass(pool);
    }

    pub fn run_addition_block_pass(&mut self) {
        // 清除空块(包括entry块)
        self.clear_empty_block();
        // jump的目标块如果紧邻，则删除jump语句
        self.clear_useless_jump();
    }

    pub fn opt_gep(&mut self) {
        self.module.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|block| {
                //获取块内所有load和store指令的位置，ls前有三个指令计算地址
                let ls_pos: Vec<usize> = block
                    .insts
                    .iter()
                    .filter(|inst| {
                        inst.get_type() == InstrsType::Load || inst.get_type() == InstrsType::Store
                    })
                    .map(|inst| block.insts.iter().position(|i| i == inst).unwrap())
                    .filter(|pos| *pos >= 3 && is_sl(*block, *pos))
                    .collect();

                // 将相同基地址的l和s合并为一组
                let mut ls_group_index: HashMap<Reg, Vec<usize>> = HashMap::new();
                // 过滤条件：基地址相同，计算偏移量为常数
                for pos in ls_pos.iter() {
                    let addr = match block.insts[pos - 3].get_lhs() {
                        Operand::Reg(reg) => reg,
                        _ => continue,
                    };
                    match block.insts[pos - 3].get_rhs() {
                        Operand::IImm(imm) => imm.get_data(),
                        _ => continue,
                    };
                    ls_group_index.entry(*addr).or_insert(Vec::new()).push(*pos);
                }

                // 对每一组进行优化
                let mut rm_pos: Vec<ObjPtr<LIRInst>> = Vec::new();
                for (_, poses) in ls_group_index.iter_mut() {
                    // 只计算每组中第一条指令的偏移量
                    let first_offset = match block.insts[poses[0] - 3].get_rhs() {
                        Operand::IImm(imm) => imm.get_data(),
                        _ => unreachable!("offset must be imm"),
                    };
                    let addr = block.insts[poses[0]].get_lhs();
                    poses.remove(0);
                    // 其他偏移由根据第一条指令的偏移计算
                    for pos in poses.iter() {
                        let inst = block.insts[*pos];
                        let offset = match block.insts[*pos - 3].get_rhs() {
                            Operand::IImm(imm) => imm.get_data(),
                            _ => unreachable!("offset must be imm"),
                        };
                        let new_offset = offset - first_offset;
                        inst.as_mut().replace_op(vec![
                            inst.get_dst().clone(),
                            addr.clone(),
                            Operand::IImm(IImm::new(new_offset * 4)),
                        ]);
                        let inst1 = block.insts[*pos - 1];
                        let inst2 = block.insts[*pos - 2];
                        let inst3 = block.insts[*pos - 3];
                        rm_pos.append(&mut vec![inst1, inst2, inst3]);
                    }
                }

                // 删除指令
                let new_insts = block
                    .insts
                    .iter()
                    .filter(|inst| !rm_pos.contains(inst))
                    .map(|x| *x)
                    .collect();
                block.as_mut().insts = new_insts;
            });
        })
    }
}

fn is_sl(block: ObjPtr<BB>, pos: usize) -> bool {
    let inst1 = block.insts[pos - 1];
    let inst2 = block.insts[pos - 2];
    let inst3 = block.insts[pos - 3];
    if inst1.get_type() == InstrsType::Binary(BinaryOp::Add)
        && inst2.get_type() == InstrsType::Binary(BinaryOp::Shl)
        && inst3.get_type() == InstrsType::Binary(BinaryOp::Add)
    {
        return true;
    }
    false
}
