use std::cmp::max;
use std::collections::LinkedList;
pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
use std::fs::OpenOptions;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
use std::io::Write;
use std::vec::Vec;

use biheap::BiHeap;

use super::instrs::InstrsType;
use super::operand::IImm;
use super::regalloc::structs::RegUsedStat;
use super::{block, structs::*, BackendPool};
use crate::backend::asm_builder::AsmBuilder;
use crate::backend::instrs::{LIRInst, Operand};
use crate::backend::module::AsmModule;
use crate::backend::operand::{Reg, ARG_REG_COUNT};
use crate::backend::regalloc::regalloc;
use crate::backend::{block::*, func, operand};
// use crate::backend::regalloc::simulate_assign;
use crate::backend::regalloc::{
    easy_ls_alloc::Allocator, regalloc::Regalloc, structs::FuncAllocStat,
};
use crate::container::bitmap::Bitmap;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::ir_type::IrType;
use crate::utility::{ObjPtr, ScalarType};
use crate::{config, log_file};

#[derive(Clone)]
pub struct Func {
    pub is_extern: bool,
    pub label: String,
    pub blocks: Vec<ObjPtr<BB>>,
    pub stack_addr: LinkedList<StackSlot>,
    pub params: Vec<ObjPtr<Inst>>,
    pub param_cnt: (i32, i32), // (int, float)

    pub entry: Option<ObjPtr<BB>>,

    reg_def: Vec<HashSet<CurInstrInfo>>,
    reg_use: Vec<HashSet<CurInstrInfo>>,
    reg_num: i32,
    // fregs: HashSet<Reg>,
    pub context: ObjPtr<Context>,

    pub reg_alloc_info: FuncAllocStat,
    pub spill_stack_map: HashMap<Reg, StackSlot>,

    pub slot_to_rearrange: HashMap<IImm, StackSlot>,

    pub const_array: HashSet<IntArray>,
    pub float_array: HashSet<FloatArray>,
    //FIXME: resolve float regs
    pub callee_saved: HashSet<Reg>,
    pub caller_saved: HashMap<Reg, Reg>,
    pub caller_saved_len: i32,
}

/// reg_num, stack_addr, caller_stack_addr考虑借助回填实现
/// 是否需要caller_stack_addr？caller函数sp保存在s0中
impl Func {
    pub fn new(name: &str, context: ObjPtr<Context>) -> Self {
        Self {
            is_extern: false,
            label: name.to_string(),
            blocks: Vec::new(),
            stack_addr: LinkedList::new(),
            params: Vec::new(),
            param_cnt: (0, 0),
            entry: None,
            reg_def: Vec::new(),
            reg_use: Vec::new(),
            reg_num: 0,
            // fregs: HashSet::new(),
            context,

            reg_alloc_info: FuncAllocStat::new(),
            spill_stack_map: HashMap::new(),
            slot_to_rearrange: HashMap::new(),

            const_array: HashSet::new(),
            float_array: HashSet::new(),
            callee_saved: HashSet::new(),
            caller_saved: HashMap::new(),
            caller_saved_len: 0,
        }
    }

    pub fn construct(
        &mut self,
        module: &AsmModule,
        ir_func: &Function,
        func_seq: i32,
        pool: &mut BackendPool,
    ) {
        let mut info = Mapping::new();
        // 处理全局变量&数组
        let globl = &module.global_var_list;
        globl.iter().for_each(|(inst, var)| {
            info.val_map
                .insert(inst.clone(), Operand::Addr(var.get_name().clone()));
        });

        // entry shouldn't generate for asm, called label for entry should always be false
        let label = &self.label;
        let mut entry = pool.put_block(BB::new(&format!(".entry_{label}")));
        entry.showed = false;
        self.entry = Some(entry);
        self.blocks.push(self.entry.unwrap());

        //判断是否是外部函数
        if ir_func.is_empty_bb() {
            self.is_extern = true;
            return;
        }

        // 第一遍pass
        let fblock = ir_func.get_head();
        let mut ir_block_set: HashSet<ObjPtr<BasicBlock>> = HashSet::new();
        let first_block = pool.put_block(BB::new(&label));
        info.ir_block_map.insert(fblock, first_block);
        info.block_ir_map.insert(first_block, fblock);
        ir_block_set.insert(fblock);

        let mut tmp = VecDeque::new();
        tmp.push_back(fblock);

        let mut block_seq = 0;
        self.blocks.push(first_block);
        let mut visited: HashSet<ObjPtr<BasicBlock>> = HashSet::new();
        while let Some(fblock) = tmp.pop_front() {
            let next_blocks = fblock.as_ref().get_next_bb();
            next_blocks.iter().for_each(|block| {
                if visited.insert(block.clone()) {
                    tmp.push_back(block.clone())
                }
            });
            if block_seq == 0 {
                block_seq += 1;
                continue;
            }
            if ir_block_set.insert(fblock) {
                let label = format!(".LBB{func_seq}_{block_seq}");
                let block = pool.put_block(BB::new(&label));
                info.ir_block_map.insert(fblock, block);
                info.block_ir_map.insert(block, fblock);
                self.blocks.push(block);
                block_seq += 1;
            }
        }
        self.handle_parameters(ir_func);
        // 第二遍pass
        let first_block = info.ir_block_map.get(&ir_func.get_head()).unwrap();
        self.entry.unwrap().as_mut().out_edge.push(*first_block);
        first_block.as_mut().in_edge.push(self.entry.unwrap());
        let mut i = 0;
        let mut index = 0;
        let this = pool.put_func(self.clone());
        loop {
            if index >= self.blocks.len() {
                break;
            }
            let block = self.blocks[index];
            if block != self.entry.unwrap() {
                if i == 0 {
                    block.as_mut().showed = false;
                }
                let basicblock = info.block_ir_map.get(&block).unwrap();
                if i + 1 < self.blocks.len() {
                    let next_block = Some(self.blocks[i + 1]);
                    block
                        .as_mut()
                        .construct(this, *basicblock, next_block, &mut info, pool);
                } else {
                    block
                        .as_mut()
                        .construct(this, *basicblock, None, &mut info, pool);
                }
                i += 1;
            }
            index += 1;
        }
        // 第三遍pass，拆phi
        let mut size = 0;
        for block in self.blocks.iter() {
            if block.insts.len() == 0 {
                continue;
            }
            let mut index = block.insts.len() - 1;
            let mut insert_before = false;
            loop {
                match block.insts[index].get_type() {
                    InstrsType::Ret(..) | InstrsType::Branch(..) | InstrsType::Jump => {
                        if index == 0 {
                            insert_before = true;
                            break;
                        }
                        index -= 1;
                    }
                    _ => {
                        break;
                    }
                }
            }
            if !insert_before {
                index += 1;
            }
            if let Some(target) = info.phis_to_block.get_mut(&block.label) {
                for inst in target.iter() {
                    block.as_mut().insts.insert(index, *inst);
                }
            }
            let mut phis = block.phis.clone();
            while let Some(inst) = phis.pop() {
                block.as_mut().insts.insert(0, inst);
            }
            size += block.insts.len();
        }
        self.update(this);
    }

    // 移除指定id的寄存器的使用信息
    pub fn del_inst_reg(&mut self, cur_info: &CurInstrInfo, inst: ObjPtr<LIRInst>) {
        for reg in inst.as_ref().get_reg_use() {
            self.reg_use[reg.get_id() as usize].remove(cur_info);
        }
        for reg in inst.as_ref().get_reg_def() {
            self.reg_def[reg.get_id() as usize].remove(cur_info);
        }
    }

    // 添加指定id的寄存器的使用信息
    pub fn add_inst_reg(&mut self, cur_info: &CurInstrInfo, inst: ObjPtr<LIRInst>) {
        for reg in inst.as_ref().get_reg_use() {
            self.reg_use[reg.get_id() as usize].insert(cur_info.clone());
        }
        for reg in inst.as_ref().get_reg_def() {
            self.reg_def[reg.get_id() as usize].insert(cur_info.clone());
        }
    }

    pub fn build_reg_info(&mut self) {
        self.reg_def.clear();
        self.reg_use.clear();
        self.reg_def.resize(self.reg_num as usize, HashSet::new());
        self.reg_use.resize(self.reg_num as usize, HashSet::new());
        let mut p: CurInstrInfo = CurInstrInfo::new(0);
        for block in self.blocks.clone() {
            p.band_block(block);
            for inst in block.as_ref().insts.iter() {
                p.insts_it = Some(*inst);
                self.add_inst_reg(&p, *inst);
                p.pos += 1;
            }
        }
    }

    pub fn calc_live(&self) {
        //TODO, 去除allocable限制!
        let calc_live_file = "callive.txt";
        // fs::remove_file(calc_live_file);
        log_file!(
            calc_live_file,
            "-----------------------------------cal live func:{}---------------------------",
            self.label
        );
        // 打印函数里面的寄存器活跃情况
        let printinterval = || {
            let mut que: VecDeque<ObjPtr<BB>> = VecDeque::new();
            let mut passed_bb = HashSet::new();
            que.push_front(self.entry.unwrap());
            passed_bb.insert(self.entry.unwrap());
            log_file!(calc_live_file, "func:{}", self.label);
            while !que.is_empty() {
                let cur_bb = que.pop_front().unwrap();
                log_file!(calc_live_file, "block {}:", cur_bb.label);
                log_file!(calc_live_file, "live in:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_in);
                log_file!(calc_live_file, "live out:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_out);
                log_file!(calc_live_file, "live use:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_use);
                log_file!(calc_live_file, "live def:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_def);
                for next in cur_bb.out_edge.iter() {
                    if passed_bb.contains(next) {
                        continue;
                    }
                    passed_bb.insert(*next);
                    que.push_back(*next);
                }
            }
        };

        // 计算公式，live in 来自于所有前继的live out的集合 + 自身的live use
        // live out等于所有后继块的live in的集合与 (自身的livein 和live def的并集) 的交集
        // 以块为遍历单位进行更新
        // TODO 重写
        // 首先计算出live def和live use
        if self.label == "main" {
            log!("to");
        }

        let mut queue: VecDeque<(ObjPtr<BB>, Reg)> = VecDeque::new();
        for block in self.blocks.iter() {
            log_file!(calc_live_file, "block:{}", block.label);
            block.as_mut().live_use.clear();
            block.as_mut().live_def.clear();
            for it in block.as_ref().insts.iter().rev() {
                log_file!(calc_live_file, "{}", it.as_ref());
                for reg in it.as_ref().get_reg_def().into_iter() {
                    block.as_mut().live_use.remove(&reg);
                    block.as_mut().live_def.insert(reg);
                }
                for reg in it.as_ref().get_reg_use().into_iter() {
                    block.as_mut().live_def.remove(&reg);
                    block.as_mut().live_use.insert(reg);
                }
            }
            log_file!(
                calc_live_file,
                "live def:{:?},live use:{:?}",
                block
                    .live_def
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<Vec<i32>>(),
                block
                    .live_use
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<Vec<i32>>()
            );
            //
            for reg in block.as_ref().live_use.iter() {
                queue.push_back((block.clone(), reg.clone()));
            }
            block.as_mut().live_in = block.as_ref().live_use.clone();
            block.as_mut().live_out.clear();
        }

        //然后计算live in 和live out
        while let Some(value) = queue.pop_front() {
            let (block, reg) = value;
            log_file!(
                calc_live_file,
                "block {} 's ins:{:?}",
                block.label,
                block
                    .in_edge
                    .iter()
                    .map(|b| &b.label)
                    .collect::<HashSet<&String>>()
            );
            for pred in block.as_ref().in_edge.iter() {
                if pred.as_mut().live_out.insert(reg) {
                    if pred.as_mut().live_def.contains(&reg) {
                        continue;
                    }
                    if pred.as_mut().live_in.insert(reg) {
                        queue.push_back((pred.clone(), reg));
                    }
                }
            }
        }

        log_file!(calc_live_file,"-----------------------------------after count live in,live out----------------------------");
        printinterval();
    }

    pub fn allocate_reg(&mut self) {
        // 函数返回地址保存在ra中
        self.calc_live();
        // for bb in self.blocks.iter() {
        //     if self.label != "float_eq" {
        //         continue;
        //     }
        //     for inst in bb.insts.iter() {
        //         inst.get_regs().iter().for_each(|r| {
        //             log!("{:?}", inst);
        //             log!("{}", inst.as_ref());
        //             log!("{}", r);
        //         })
        //     }
        // }
        let mut allocator = crate::backend::regalloc::easy_ls_alloc::Allocator::new();
        // let mut allocator = crate::backend::regalloc::easy_gc_alloc::Allocator::new();
        // let mut allocator = crate::backend::regalloc::opt_gc_alloc2::Allocator::new();
        // let mut allocator = crate::backend::regalloc::opt_gc_alloc::Allocator::new();
        // let mut allocator = crate::backend::regalloc::base_alloc::Allocator::new();
        let mut alloc_stat = allocator.alloc(self);

        // 评价估计结果
        log_file!(
            "000_eval_alloc.txt",
            "func:{},alloc_cost:{}",
            self.label,
            regalloc::eval_alloc(self, &alloc_stat.dstr, &alloc_stat.spillings)
        );

        log_file!(
            "calout.txt",
            "dstr,num:{} :{:?},\nspillings,num:{}:{:?}",
            alloc_stat.dstr.len(),
            alloc_stat.dstr,
            alloc_stat.spillings.len(),
            alloc_stat.spillings
        );
        let file_path = config::get_file_path().unwrap();
        if alloc_stat.spillings.len() == 0 {
            log_file!(
                "bestalloc.txt",
                "func: {}-{}",
                file_path.to_owned(),
                self.label
            );
        } else {
            log_file!(
                "unbestalloc.txt",
                "func:{}-{},dstr/spill:{}",
                file_path.to_owned(),
                self.label,
                alloc_stat.dstr.len() as f32 / alloc_stat.spillings.len() as f32
            );
        }
        let check_alloc_path = "./check_alloc.txt";
        log_file!(check_alloc_path, "{:?}", self.label);
        regalloc::check_alloc_v2(&self, &alloc_stat.dstr, &alloc_stat.spillings);
        // log_file!(
        //     check_alloc_path,
        //     "{:?}",
        //     regalloc::check_alloc(self, &alloc_stat.dstr, &alloc_stat.spillings)
        // );
        // TODO
        // simulate_assign::Simulator::simulate(&self, &alloc_stat);

        self.reg_alloc_info = alloc_stat;
        self.context.as_mut().set_reg_map(&self.reg_alloc_info.dstr);
        // log!("dstr map info{:?}", self.reg_alloc_info.dstr);
        // log!("spills:{:?}", self.reg_alloc_info.spillings);

        // let stack_size = self.max_params * ADDR_SIZE;
        // log!("set stack size:{}", stack_size);
        // self.context.as_mut().set_offset(stack_size);
    }

    fn handle_parameters(&mut self, ir_func: &Function) {
        let mut iparam: Vec<_> = ir_func
            .get_parameter_list()
            .iter()
            .filter(|param| {
                param.as_ref().get_param_type() == IrType::Int
                    || param.get_param_type() == IrType::IntPtr
                    || param.get_param_type() == IrType::FloatPtr
            })
            .map(|param| param.clone())
            .collect();
        let mut fparam: Vec<_> = ir_func
            .get_parameter_list()
            .iter()
            .filter(|param| param.as_ref().get_param_type() == IrType::Float)
            .map(|param| param.clone())
            .collect();
        self.param_cnt = (iparam.len() as i32, fparam.len() as i32);
        self.params.append(&mut iparam);
        self.params.append(&mut fparam);

        let mut offset = 0;
        let overflow_param =
            max(0, self.param_cnt.0 - ARG_REG_COUNT) + max(0, self.param_cnt.1 - ARG_REG_COUNT);
        offset = overflow_param * ADDR_SIZE;
        let mut slot = StackSlot::new(offset, offset);
        assert!(self.stack_addr.is_empty());
        self.stack_addr.push_front(StackSlot::new(0, 0));
        slot.set_fix();
        self.stack_addr.push_front(slot);
    }

    pub fn get_first_block(&self) -> ObjPtr<BB> {
        self.blocks[1].clone()
    }

    ///做了 spill操作以及caller save和callee save的保存和恢复
    pub fn handle_spill(&mut self, pool: &mut BackendPool, f: &mut File) {
        let this = pool.put_func(self.clone());
        for block in self.blocks.iter() {
            block
                .as_mut()
                .handle_spill(this, &self.reg_alloc_info.spillings, pool);
        }
        for block in self.blocks.iter() {
            block.as_mut().save_reg(this, pool);
        }
        self.update(this);
        self.save_callee(pool, f);
    }

    pub fn handle_overflow(&mut self, pool: &mut BackendPool) {
        let this = pool.put_func(self.clone());
        for block in self.blocks.iter() {
            block.as_mut().handle_overflow(this, pool);
        }
        self.update(this);
    }

    fn update(&mut self, func: ObjPtr<Func>) {
        let func_ref = func.as_ref();
        self.blocks = func_ref.blocks.clone();
        self.stack_addr = func_ref.stack_addr.clone();
        self.spill_stack_map = func_ref.spill_stack_map.clone();
        self.const_array = func_ref.const_array.clone();
        self.float_array = func_ref.float_array.clone();
        self.callee_saved = func_ref.callee_saved.clone();
        self.caller_saved = func_ref.caller_saved.clone();
        self.caller_saved_len = func_ref.caller_saved_len;
    }

    /// 为要保存的callee save寄存器开栈,然后开栈以及处理 callee save的保存和恢复
    pub fn save_callee(&mut self, pool: &mut BackendPool, f: &mut File) {
        let mut callee_map: HashMap<Reg, StackSlot> = HashMap::new();
        if self.label == "main" {
            self.build_stack_info(f, callee_map, true);
            return;
        }
        for id in self.callee_saved.iter() {
            let pos = self.stack_addr.front().unwrap().get_pos() + ADDR_SIZE;
            let slot = StackSlot::new(pos, ADDR_SIZE);
            self.stack_addr.push_front(slot);
            //FIXME: resolve float regs
            callee_map.insert(*id, slot);
        }
        self.build_stack_info(f, callee_map, false);
    }

    ///进行开栈操作和callee的save和restore操作
    fn build_stack_info(&mut self, f: &mut File, map: HashMap<Reg, StackSlot>, is_main: bool) {
        // 当完成callee save寄存器存栈后，可以得知开栈大小
        let mut f1 = match f.try_clone() {
            Ok(f) => f,
            Err(e) => panic!("Error: {}", e),
        };
        let mut f2 = match f.try_clone() {
            Ok(f) => f,
            Err(e) => panic!("Error: {}", e),
        };
        let mut stack_size = self.context.get_offset();

        // log!("stack size: {}", stack_size);

        if let Some(addition_stack_info) = self.stack_addr.front() {
            stack_size += addition_stack_info.get_pos();
        }
        if let Some(slot) = self.stack_addr.back() {
            stack_size += slot.get_pos() + slot.get_size();
        };
        // log!("stack: {:?}", self.stack_addr);
        stack_size += self.caller_saved_len * ADDR_SIZE;
        // log!("caller saved: {}", self.caller_saved.len());
        //栈对齐 - 调用func时sp需按16字节对齐
        stack_size = stack_size / 16 * 16 + 16;
        let (icnt, fcnt) = self.param_cnt;
        self.context.as_mut().set_offset(stack_size - ADDR_SIZE);

        let ra = Reg::new(1, ScalarType::Int);
        let map_clone = map.clone();

        self.context.as_mut().set_prologue_event(move || {
            let mut builder = AsmBuilder::new(&mut f1);
            // addi sp -stack_size
            if operand::is_imm_12bs(stack_size) {
                builder.addi("sp", "sp", -stack_size);
                builder.s(
                    &ra.to_string(false),
                    "sp",
                    stack_size - ADDR_SIZE,
                    false,
                    true,
                );
                if !is_main {
                    for (reg, slot) in map.iter() {
                        let is_float = reg.get_type() == ScalarType::Float;
                        let of = stack_size - ADDR_SIZE - slot.get_pos();
                        builder.s(&reg.to_string(false), "sp", of, is_float, true);
                    }
                }
            } else {
                builder.op1("li", "s0", &stack_size.to_string());
                builder.op2("sub", "sp", "sp", "s0", false, true);
                builder.op2("add", "s0", "s0", "sp", false, true);
                builder.s(&ra.to_string(false), "s0", -ADDR_SIZE, false, true);

                let mut first = true;
                let mut start = 0;
                if !is_main {
                    for (reg, slot) in map.iter() {
                        let is_float = reg.get_type() == ScalarType::Float;
                        if operand::is_imm_12bs(slot.get_pos()) {
                            builder.s(
                                &reg.to_string(false),
                                "s0",
                                -(slot.get_pos() + ADDR_SIZE),
                                is_float,
                                true,
                            );
                        } else if operand::is_imm_12bs(stack_size - slot.get_pos() - ADDR_SIZE) {
                            builder.s(
                                &reg.to_string(false),
                                "sp",
                                stack_size - ADDR_SIZE - slot.get_pos(),
                                is_float,
                                true,
                            );
                        } else {
                            if first {
                                let offset = stack_size - ADDR_SIZE - slot.get_pos();
                                builder.op1("li", "s0", &offset.to_string());
                                builder.op2("add", "s0", "s0", "sp", false, true);
                                first = false;
                            }
                            builder.s(&reg.to_string(false), "s0", -start, is_float, true);
                            start += ADDR_SIZE;
                        }
                    }
                }
            }
        });

        self.context.as_mut().set_epilogue_event(move || {
            let mut builder = AsmBuilder::new(&mut f2);

            if operand::is_imm_12bs(stack_size) {
                if !is_main {
                    for (reg, slot) in map_clone.iter() {
                        let is_float = reg.get_type() == ScalarType::Float;
                        let of = stack_size - ADDR_SIZE - slot.get_pos();
                        builder.l(&reg.to_string(false), "sp", of, is_float, true);
                    }
                }
                builder.l(
                    &ra.to_string(false),
                    "sp",
                    stack_size - ADDR_SIZE,
                    false,
                    true,
                );
                builder.addi("sp", "sp", stack_size);
            } else {
                builder.op1("li", "s0", &stack_size.to_string());
                builder.op2("add", "sp", "s0", "sp", false, true);
                builder.l(&ra.to_string(false), "sp", -ADDR_SIZE, false, true);

                let mut first = true;
                let mut start = 0;
                if !is_main {
                    for (reg, slot) in map_clone.iter() {
                        let is_float = reg.get_type() == ScalarType::Float;
                        if operand::is_imm_12bs(slot.get_pos()) {
                            builder.l(
                                &reg.to_string(false),
                                "sp",
                                -(slot.get_pos() + ADDR_SIZE),
                                is_float,
                                true,
                            );
                        } else if operand::is_imm_12bs(stack_size - slot.get_pos() - ADDR_SIZE) {
                            builder.l(
                                &reg.to_string(false),
                                "sp",
                                stack_size - slot.get_pos() - ADDR_SIZE,
                                is_float,
                                true,
                            );
                        } else {
                            if first {
                                let offset = stack_size - slot.get_pos() - ADDR_SIZE;
                                builder.op1("li", "s0", &offset.to_string());
                                builder.op2("add", "s0", "s0", "sp", false, true);
                                first = false;
                            }
                            builder.l(&reg.to_string(false), "sp", -start, is_float, true);
                            start += ADDR_SIZE;
                        }
                    }
                }
            }
        });
    }

    pub fn generate_row(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        AsmBuilder::new(f).show_func(&self.label)?;
        // self.context.as_mut().call_prologue_event();
        let mut size = 0;
        for block in self.blocks.iter() {
            size += block.insts.len();
        }
        for block in self.blocks.iter() {
            block.as_mut().generate_row(self.context, f)?;
        }
        Ok(())
    }
}

/// 打印函数当前的汇编形式
impl Func {
    pub fn print_func(&self) {
        log!("func:{}", self.label);
        for block in self.blocks.iter() {
            log!("\tblock:{}", block.label);
            for inst in block.insts.iter() {
                log!("\t\t{}", inst.to_string());
            }
        }
    }
}

impl GenerateAsm for Func {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        if self.const_array.len() > 0 || self.float_array.len() > 0 {
            writeln!(f, "	.data\n   .align  3")?;
        }
        for mut a in self.const_array.clone() {
            a.generate(self.context, f)?;
        }
        for mut a in self.float_array.clone() {
            a.generate(self.context, f)?;
        }
        AsmBuilder::new(f).show_func(&self.label)?;
        self.context.as_mut().call_prologue_event();
        let mut size = 0;
        for block in self.blocks.iter() {
            size += block.insts.len();
        }
        // log!("tatol {}", size);
        for block in self.blocks.iter() {
            block.as_mut().generate(self.context, f)?;
        }
        writeln!(f, "	.size	{}, .-{}", self.label, self.label)?;
        Ok(())
    }
}

/// 从函数中提取信息
impl Func {
    // 实现一些关于函数信息的估计和获取的方法

    // 估计寄存器数量
    pub fn estimate_num_regs(&self) -> usize {
        let mut out = 0;
        self.blocks.iter().for_each(|bb| out += bb.insts.len());
        return out;
    }
    // 获取指令数量
    pub fn num_insts(&self) -> usize {
        let mut out = 0;
        self.blocks.iter().for_each(|bb| out += bb.insts.len());
        return out;
    }

    // 获取寄存器数量
    pub fn num_regs(&self) -> usize {
        let mut passed: Bitmap = Bitmap::with_cap(1000);
        let mut out = 0;
        self.blocks.iter().for_each(|bb| {
            bb.insts.iter().for_each(|inst| {
                for reg in inst.get_reg_def() {
                    let id = reg.get_id() << 1
                        | match reg.get_type() {
                            ScalarType::Float => 0,
                            ScalarType::Int => 1,
                            _ => panic!("unleagal"),
                        };
                    if passed.contains(id as usize) {
                        continue;
                    }
                    passed.insert(id as usize);
                    out += 1;
                }
            })
        });
        return out;
    }

    // 获取所有虚拟寄存器和用到的物理寄存器
    pub fn draw_all_virtual_regs(&self) -> HashSet<Reg> {
        let mut passed = HashSet::new();
        let mut out = self.blocks.iter().for_each(|bb| {
            bb.insts.iter().for_each(|inst| {
                for reg in inst.get_regs() {
                    if reg.is_physic() {
                        continue;
                    }
                    passed.insert(reg);
                }
            })
        });
        passed
    }
}

/// handle spill2: handle spill过程中对spill寄存器用到的栈进行重排
/// func的handle spill v2能够与v1 完美替换
impl Func {
    /// 为spilling 寄存器预先分配空间 的 handle spill
    pub fn handle_spill_v2(&mut self, pool: &mut BackendPool, f: &mut File) {
        let this = pool.put_func(self.clone());
        // 首先给这个函数分配spill的空间
        self.assign_stack_slot_for_spill();
        for block in self.blocks.iter() {
            block
                .as_mut()
                .handle_spill(this, &self.reg_alloc_info.spillings, pool);
        }
        for block in self.blocks.iter() {
            block.as_mut().save_reg(this, pool);
        }
        self.update(this);
        self.save_callee(pool, f);
    }

    /// 为了分配spill的虚拟寄存器所需的栈空间使用的而构建冲突图
    fn build_interferench_for_assign_stack_slot_for_spill(&mut self) -> HashMap<Reg, HashSet<Reg>> {
        let mut out: HashMap<Reg, HashSet<Reg>> = HashMap::new();
        self.calc_live();
        for bb in self.blocks.iter() {
            //
            let bb = *bb;
            let mut live_now: HashSet<Reg> = HashSet::new();
            for reg in bb.live_out.iter() {
                if !self.reg_alloc_info.spillings.contains(&reg.get_id()) {
                    continue;
                }
                if !out.contains_key(reg) {
                    out.insert(*reg, HashSet::new());
                }
                live_now.insert(*reg);
            }
            for inst in bb.insts.iter().rev() {
                for reg in inst.get_reg_def() {
                    if !self.reg_alloc_info.spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    if !out.contains_key(&reg) {
                        out.insert(reg, HashSet::new());
                    }
                    for live in live_now.iter() {
                        if reg == *live {
                            continue;
                        }
                        out.get_mut(&reg).unwrap().insert(*live);
                        out.get_mut(live).unwrap().insert(reg);
                    }
                }
                for reg in inst.get_reg_def() {
                    live_now.remove(&reg);
                }
                for reg in inst.get_reg_use() {
                    if !self.reg_alloc_info.spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    live_now.insert(reg);
                }
            }
        }
        out
    }

    /// 分析spill空间之间的冲突关系,进行紧缩
    fn assign_stack_slot_for_spill(&mut self) {
        // 统计所有spill寄存器的使用次数,根据寄存器数量更新其值
        // 首先给存储在物理寄存器中的值的空间
        // for reg in RegUsedStat::new().get_available_freg() {
        //     let last_slot = self.stack_addr.back().unwrap();
        //     let pos = last_slot.get_pos() + last_slot.get_size();
        //     let stack_slot = StackSlot::new(pos, ADDR_SIZE);
        //     self.stack_addr.push_back(stack_slot);
        //     let reg = Reg::new(reg, ScalarType::Float);
        //     self.spill_stack_map.insert(reg, stack_slot);
        //     self.physic_stack_map.insert(reg, stack_slot);
        // }
        // for reg in RegUsedStat::new().get_available_ireg() {
        //     let last_slot = self.stack_addr.back().unwrap();
        //     let pos = last_slot.get_pos() + last_slot.get_size();
        //     let stack_slot = StackSlot::new(pos, ADDR_SIZE);
        //     self.stack_addr.push_back(stack_slot);
        //     let reg = Reg::new(reg, ScalarType::Int);
        //     self.spill_stack_map.insert(reg, stack_slot);
        //     self.physic_stack_map.insert(reg, stack_slot);
        // }
        // 给spill的寄存器空间,如果出现重复的情况,则说明后端可能空间存在冲突
        // 建立spill寄存器之间的冲突关系(如果两个spill的寄存器之间是相互冲突的,则它们不能够共享相同内存)
        let mut spill_coes: HashMap<i32, i32> = HashMap::new();
        let mut id_to_regs: HashMap<i32, Reg> = HashMap::new();
        let spillings = &self.reg_alloc_info.spillings;
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_reg_use() {
                    if reg.is_physic() {
                        continue;
                    }
                    if !spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    id_to_regs.insert(reg.get_id(), reg);
                    spill_coes.insert(
                        reg.get_id(),
                        spill_coes.get(&reg.get_id()).unwrap_or(&0) + 1,
                    );
                }
                for reg in inst.get_reg_def() {
                    if reg.is_physic() {
                        continue;
                    }
                    if !spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    id_to_regs.insert(reg.get_id(), reg);
                    spill_coes.insert(
                        reg.get_id(),
                        spill_coes.get(&reg.get_id()).unwrap_or(&0) + 1,
                    );
                }
            }
        }
        // 桶排序
        let mut buckets: HashMap<i32, LinkedList<Reg>> = HashMap::new();
        let mut order: BiHeap<i32> = BiHeap::new();
        for id in spillings {
            if !buckets.contains_key(id) {
                order.push(*id);
                buckets.insert(*id, LinkedList::new());
            }
            let reg = id_to_regs.get(id).unwrap();
            buckets.get_mut(id).unwrap().push_back(*reg);
        }

        // 使用一个表记录之前使用过的空间,每次分配空间的时候可以复用之前使用过的空间,只要没有冲突
        // 如果有冲突则 需要开辟新的空间
        let mut slots: LinkedList<StackSlot> = LinkedList::new();
        let inter_graph: HashMap<Reg, HashSet<Reg>> =
            self.build_interferench_for_assign_stack_slot_for_spill();
        // 优先给使用次数最多的spill寄存器分配内存空间
        while !order.is_empty() {
            let id = order.pop_max().unwrap();
            let lst = buckets.get_mut(&id).unwrap();
            while !lst.is_empty() {
                let toassign = lst.pop_front().unwrap();
                if self.spill_stack_map.contains_key(&toassign) {
                    unreachable!()
                }
                // 首先在已经分配的空间里面寻找可复用的空间
                // 首先记录冲突的空间
                let mut inter_slots: HashSet<StackSlot> = HashSet::new();
                for reg in inter_graph.get(&toassign).unwrap() {
                    if !self.spill_stack_map.contains_key(reg) {
                        continue;
                    }
                    let stack_slot = self.spill_stack_map.get(reg).unwrap();
                    inter_slots.insert(*stack_slot);
                }

                // 然后遍历已经分配的空间
                let mut num = slots.len();
                let mut slot_for_toassign: Option<StackSlot> = Option::None;
                while num > 0 {
                    num -= 1;
                    let old_slot = slots.pop_front().unwrap();
                    slots.push_back(old_slot);
                    if inter_slots.contains(&old_slot) {
                        continue;
                    }
                    slot_for_toassign = Some(old_slot);
                }
                if slot_for_toassign.is_none() {
                    let last_slot = self.stack_addr.back().unwrap();
                    let pos = last_slot.get_pos() + last_slot.get_size();
                    let stack_slot = StackSlot::new(pos, ADDR_SIZE);
                    self.stack_addr.push_back(stack_slot);
                    slot_for_toassign = Some(stack_slot);
                    slots.push_back(stack_slot);
                }
                self.spill_stack_map
                    .insert(toassign, slot_for_toassign.unwrap());
            }
        }
    }
}

//关于无用指令消除的实现
impl Func {
    ///移除无用指令
    pub fn remove_unuse_inst(&mut self) {
        //TOCHECK
        // 移除mv va va 类型指令
        for bb in self.blocks.iter() {
            let mut index = 0;
            while index < bb.insts.len() {
                let inst = bb.insts[index];
                if inst.operands.len() < 2 {
                    index += 1;
                    continue;
                }
                let dst = inst.get_dst();
                let src = inst.get_lhs();
                if inst.get_type() == InstrsType::OpReg(super::instrs::SingleOp::Mv) && dst == src {
                    bb.as_mut().insts.remove(index);
                } else {
                    index += 1;
                }
            }
        }
        // 移除无用def
        self.remove_unuse_def();
    }

    ///移除无用def指令
    pub fn remove_unuse_def(&mut self) {
        //
        loop {
            self.calc_live();
            let mut ifFinish = true;
            for bb in self.blocks.iter() {
                let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::with_capacity(bb.insts.len());
                let mut to_removed: HashSet<usize> = HashSet::new();
                let mut live_now: HashSet<Reg> = HashSet::new();
                bb.live_out.iter().for_each(|reg| {
                    live_now.insert(*reg);
                });
                for (index, inst) in bb.insts.iter().enumerate().rev() {
                    for reg in inst.get_reg_def() {
                        if !live_now.contains(&reg) {
                            to_removed.insert(index);
                            ifFinish = false;
                            break;
                        }
                        live_now.remove(&reg);
                    }
                    if to_removed.contains(&index) {
                        continue;
                    }
                    for reg in inst.get_reg_use() {
                        live_now.insert(reg);
                    }
                }
                for (index, inst) in bb.insts.iter().enumerate() {
                    if to_removed.contains(&index) {
                        log_file!(
                            "remove_unusedef.txt",
                            ":{}-{}:{}",
                            self.label,
                            bb.label,
                            inst.to_string()
                        );
                        continue;
                    }
                    new_insts.push(*inst);
                }
                bb.as_mut().insts = new_insts;
            }

            if ifFinish {
                break;
            }
        }
    }
}

/// handle spill v3实现
impl Func {
    ///p_to_v
    /// 把函数中分配到物理寄存器的虚拟寄存器改为使用虚拟寄存器
    pub fn p2v(&mut self) {}

    ///精细化的handle spill:
    ///
    ///遇到spilling寄存器的时候:
    /// * 优先使用available的寄存器
    ///     其中,优先使用caller save的寄存器
    ///     ,再考虑使用callee save的寄存器.
    /// * 如果要使用unavailable的寄存器,才需要进行spill操作来保存和恢复原值
    ///     优先使用caller save的寄存器,
    /// * 一定要spill到内存上的时候,使用递增的slot,把slot记录到数组的表中,等待重排
    pub fn handle_spill_v3(&mut self, pool: &mut BackendPool) {
        self.calc_live();
        let this = pool.put_func(self.clone());
        for bb in self.blocks.iter() {
            bb.as_mut().handle_spill_v3(&this.reg_alloc_info, pool);
        }
    }

    ///在handle spill之后调用
    /// 返回该函数使用了哪些callee saved的寄存器
    pub fn draw_used_callees(&self) -> HashSet<Reg> {
        let mut callees: HashSet<Reg> = HashSet::new();
        for bb in self.blocks.iter() {
            for inst in bb.insts.iter() {
                for reg in inst.get_regs() {
                    if reg.is_callee_save() {
                        callees.insert(reg);
                    }
                }
            }
        }
        callees
    }

    /// 在handle spill之后调用
    /// 生成call前后需要保存的caller save寄存器的信息
    /// 是否需要保存caller save,在于被调用函数是否使用了caller save寄存器
    /// 所以被调用函数应该尽量地不使用需要保存地caller save寄存器
    /// 遇到新的函数返回新的函数
    pub fn analyse_for_handle_call(
        &mut self,
        pool: &mut BackendPool,
        base_funcs: &HashMap<String, ObjPtr<Func>>,
        call_info: &mut HashMap<String, HashMap<Bitmap, ObjPtr<Func>>>,
    ) -> Vec<ObjPtr<Func>> {
        let mut new_funcs: Vec<ObjPtr<Func>> = Vec::new();
        todo!();
        new_funcs
    }

    /// 根据信息进行call的插入
    pub fn handle_call(&mut self, call_info: &HashMap<String, HashMap<Bitmap, ObjPtr<Func>>>) {
        self.calc_live();
    }
}

///handle call v3的实现
impl Func {
    ///配合v3系列的module.build
    /// 实现了自适应函数调用
    pub fn handle_call_v3(
        &mut self,
        pool: &mut BackendPool,
        callers_used: &HashMap<String, HashSet<Reg>>,
    ) {
        ///
        self.calc_live();
        let mut slots_for_caller_saved: Vec<StackSlot> = Vec::new();
        ///
        for bb in self.blocks.iter() {
            let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
            let mut live_now: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                live_now.insert(*reg);
            });
            for inst in bb.insts.iter().rev() {
                if inst.get_type() == InstrsType::Call {
                    ///找出 caller saved
                    let mut to_saved: Vec<Reg> = Vec::new();
                    for reg in live_now.iter() {
                        if reg.is_caller_save() {
                            to_saved.push(*reg);
                        }
                    }

                    //TODO to_check, 根据指令判断是否使用
                    let func_name = inst.get_label().get_func_name();
                    let callers_used = callers_used.get(&func_name).unwrap();
                    to_saved = to_saved
                        .iter()
                        .cloned()
                        .filter(|reg| callers_used.contains(reg))
                        .collect();

                    //根据调用的函数的情况,判断这个函数使用了哪些caller save寄存器
                    // 准备栈空间
                    while slots_for_caller_saved.len() < to_saved.len() {
                        let last_slot = self.stack_addr.back().unwrap();
                        let new_pos = last_slot.get_pos() + last_slot.get_size();
                        let new_slot = StackSlot::new(new_pos, ADDR_SIZE);
                        self.stack_addr.push_back(new_slot);
                        slots_for_caller_saved.push(new_slot);
                    }
                    //产生一条指令
                    let build_ls = |reg: Reg, offset: i32, kind: InstrsType| -> LIRInst {
                        debug_assert!(
                            (kind == InstrsType::LoadFromStack || kind == InstrsType::StoreToStack)
                        );
                        let mut ins = LIRInst::new(
                            kind,
                            vec![Operand::Reg(reg), Operand::IImm(IImm::new(offset))],
                        );
                        ins.set_double();
                        ins
                    };
                    // 插入恢复指令
                    for (index, reg) in to_saved.iter().enumerate() {
                        let pos = slots_for_caller_saved.get(index).unwrap().get_pos();
                        let load_inst = build_ls(*reg, pos, InstrsType::LoadFromStack);
                        let load_inst = pool.put_inst(load_inst);
                        new_insts.push(load_inst);
                    }
                    new_insts.push(*inst);
                    //插入保存指令
                    for (index, reg) in to_saved.iter().enumerate() {
                        let pos = slots_for_caller_saved.get(index).unwrap().get_pos();
                        let store_inst = build_ls(*reg, pos, InstrsType::StoreToStack);
                        let store_inst = pool.put_inst(store_inst);
                        new_insts.push(store_inst);
                    }
                    continue;
                }
                for reg in inst.get_reg_def() {
                    debug_assert!(live_now.contains(&reg));
                    live_now.remove(&reg);
                }
                for reg in inst.get_reg_use() {
                    live_now.insert(reg);
                }
                new_insts.push(*inst);
            }
            new_insts.reverse();
            bb.as_mut().insts = new_insts;
        }

        slots_for_caller_saved.iter().for_each(|slot| {
            let imm = IImm::new(slot.get_pos());
            self.slot_to_rearrange.insert(imm, *slot);
        });
    }
}

// rearrange slot实现 ,for module-build v3
impl Func {
    pub fn rearrange_stack_slot(&mut self) {}
}
