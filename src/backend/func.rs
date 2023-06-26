use std::cmp::max;
use std::collections::LinkedList;
pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
use std::io::Write;
use std::vec::Vec;

use super::instrs::InstrsType;
use super::{structs::*, BackendPool};
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
use crate::log_file;
use crate::utility::{ObjPtr, ScalarType};

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

    pub const_array: HashSet<IntArray>,
    pub float_array: HashSet<FloatArray>,
    //FIXME: resolve float regs
    pub callee_saved: HashSet<Reg>,
    pub caller_saved: HashMap<Reg, Reg>,
    pub max_params: i32,
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
            const_array: HashSet::new(),
            float_array: HashSet::new(),
            callee_saved: HashSet::new(),
            caller_saved: HashMap::new(),
            max_params: 0,
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
                // log!("start build block");
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
            if let Some(mut target) = info.phis_to_block.get_mut(&block.label) {
                for inst in target.iter() {
                    // log!("label: {}", block.label);
                    // log!("insert phi to last: {:?}", inst);
                    block.as_mut().insts.insert(index, *inst);
                }
            }
            let mut phis = block.phis.clone();
            while let Some(inst) = phis.pop() {
                block.as_mut().insts.insert(0, inst);
            }
            size += block.insts.len();
        }
        // log!("phi insert size: {}", size);

        for block in self.blocks.iter() {
            // log!("-----------------");
            // log!("block: {:?}", block.label);
            for inst in block.insts.iter() {
                // log!("row inst: {:?}", inst);
            }
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

    pub fn calc_live(&mut self) {
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

        // log_file!(calc_live_file,"-----------------------------------before count live def,live use----------------------------");
        // printinterval();

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
                    if reg.is_virtual() || reg.is_allocable() {
                        block.as_mut().live_use.remove(&reg);
                        block.as_mut().live_def.insert(reg);
                    }
                }
                for reg in it.as_ref().get_reg_use().into_iter() {
                    if reg.is_virtual() || reg.is_allocable() {
                        block.as_mut().live_def.remove(&reg);
                        block.as_mut().live_use.insert(reg);
                    }
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

        // log_file!(calc_live_file,"-----------------------------------before count live in,live out----------------------------");
        // printinterval();

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
                // log_file!(
                //     "19_2.txt",
                //     "live in:{:?}\nlive out:{:?}\nlive def:{:?}\nlive use:{:?}",
                //     pred.live_in
                //         .iter()
                //         .map(|r| r.get_id())
                //         .collect::<HashSet<i32>>(),
                //     pred.live_out
                //         .iter()
                //         .map(|r| r.get_id())
                //         .collect::<HashSet<i32>>(),
                //     pred.live_def
                //         .iter()
                //         .map(|r| r.get_id())
                //         .collect::<HashSet<i32>>(),
                //     pred.live_use
                //         .iter()
                //         .map(|r| r.get_id())
                //         .collect::<HashSet<i32>>(),
                // );
            }
        }

        log_file!(calc_live_file,"-----------------------------------after count live in,live out----------------------------");
        printinterval();
    }

    pub fn allocate_reg(&mut self) {
        // 函数返回地址保存在ra中
        self.calc_live();
        for bb in self.blocks.iter() {
            if self.label != "float_eq" {
                continue;
            }
            for inst in bb.insts.iter() {
                inst.get_regs().iter().for_each(|r| {
                    log!("{:?}", inst);
                    log!("{}", inst.as_ref());
                    log!("{}", r);
                })
            }
        }
        // let mut allocator = crate::backend::regalloc::easy_ls_alloc::Allocator::new();
        // let mut allocator = crate::backend::regalloc::easy_gc_alloc::Allocator::new();
        // let mut allocator=crate::backend::regalloc::opt_gc_alloc::Allocator::new();
        let mut allocator = crate::backend::regalloc::base_alloc::Allocator::new();
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
            "{:?},\n{:?}",
            alloc_stat.dstr,
            alloc_stat.spillings
        );
        let check_alloc_path = "check_alloc.txt";
        log_file!(check_alloc_path, "{:?}", self.label);
        log_file!(
            check_alloc_path,
            "{:?}",
            regalloc::check_alloc(self, &alloc_stat.dstr, &alloc_stat.spillings)
        );
        // TODO
        // simulate_assign::Simulator::simulate(&self, &alloc_stat);

        self.reg_alloc_info = alloc_stat;
        self.context.as_mut().set_reg_map(&self.reg_alloc_info.dstr);
        log!("dstr map info{:?}", self.reg_alloc_info.dstr);
        log!("spills:{:?}", self.reg_alloc_info.spillings);

        let stack_size = self.max_params * ADDR_SIZE;
        log!("set stack size:{}", stack_size);
        self.context.as_mut().set_offset(stack_size);
    }

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
                if inst.get_type() == InstrsType::OpReg(super::instrs::SingleOp::IMv) && dst == src
                {
                    bb.as_mut().insts.remove(index);
                } else {
                    index += 1;
                }
            }
        }
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
        let slot = StackSlot::new(offset, offset);
        assert!(self.stack_addr.is_empty());
        self.stack_addr.push_front(StackSlot::new(0, 0));
        self.stack_addr.push_front(slot);
    }

    pub fn get_first_block(&self) -> ObjPtr<BB> {
        self.blocks[1].clone()
    }

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
        self.max_params = func_ref.max_params;
    }

    fn save_callee(&mut self, pool: &mut BackendPool, f: &mut File) {
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
        log!("stack size:{}", stack_size);

        if let Some(addition_stack_info) = self.stack_addr.front() {
            stack_size += addition_stack_info.get_pos();
        }
        if let Some(slot) = self.stack_addr.back() {
            stack_size += slot.get_pos() + slot.get_size();
        };
        stack_size += self.caller_saved.len() as i32 * ADDR_SIZE;

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
                builder.op1("li", "gp", &stack_size.to_string());
                builder.op2("sub", "sp", "sp", "gp", false, true);
                builder.op2("add", "gp", "gp", "sp", false, true);
                builder.s(&ra.to_string(false), "gp", -ADDR_SIZE, false, true);

                let mut first = true;
                let mut start = 0;
                if !is_main {
                    for (reg, slot) in map.iter() {
                        let is_float = reg.get_type() == ScalarType::Float;
                        if operand::is_imm_12bs(slot.get_pos()) {
                            builder.s(
                                &reg.to_string(false),
                                "gp",
                                -(slot.get_pos()),
                                is_float,
                                true,
                            );
                        } else if operand::is_imm_12bs(stack_size - slot.get_pos()) {
                            builder.s(&reg.to_string(false), "sp", slot.get_pos(), is_float, true);
                        } else {
                            if first {
                                let offset = stack_size - slot.get_pos();
                                builder.op1("li", "gp", &offset.to_string());
                                builder.op2("add", "gp", "gp", "sp", false, true);
                                first = false;
                            }
                            builder.s(&reg.to_string(false), "gp", -start, is_float, true);
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
                builder.op1("li", "gp", &stack_size.to_string());
                builder.op2("add", "sp", "sp", "gp", false, true);
                builder.l(&ra.to_string(false), "sp", -ADDR_SIZE, false, true);

                let mut first = true;
                let mut start = 0;
                if !is_main {
                    for (reg, slot) in map_clone.iter() {
                        let is_float = reg.get_type() == ScalarType::Float;
                        if operand::is_imm_12bs(slot.get_pos()) {
                            builder.l(
                                &reg.to_string(false),
                                "gp",
                                -(slot.get_pos()),
                                is_float,
                                true,
                            );
                        } else if operand::is_imm_12bs(stack_size - slot.get_pos()) {
                            builder.l(&reg.to_string(false), "sp", slot.get_pos(), is_float, true);
                        } else {
                            if first {
                                let offset = stack_size - slot.get_pos();
                                builder.op1("li", "gp", &offset.to_string());
                                builder.op2("add", "gp", "gp", "sp", false, true);
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

impl Func {
    pub fn print_func(&self) {
        for block in self.blocks.iter() {
            for inst in block.insts.iter() {
                log!("{:?}", inst.as_ref());
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
        log!(
            "func: {}, stack size:{}",
            self.label,
            self.context.get_offset()
        );
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

fn set_append(blocks: &Vec<ObjPtr<BasicBlock>>) -> HashSet<ObjPtr<BasicBlock>> {
    let mut set = HashSet::new();
    for block in blocks.iter() {
        set.insert(block.clone());
    }
    set
}

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
}
