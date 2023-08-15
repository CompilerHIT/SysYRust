use std::cmp::max;
use std::collections::LinkedList;
pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
use std::fs::OpenOptions;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
pub mod alloc;
pub mod handle_spill;
pub mod mem_rearrange;
pub mod realloc;
pub mod rm_inst;
use std::io::Write;
use std::vec::Vec;

use super::instrs::{InstrsType, SingleOp};
use super::operand::IImm;
use super::{structs::*, BackendPool};
use crate::backend::asm_builder::AsmBuilder;
use crate::backend::instrs::{BinaryOp, LIRInst, Operand};
use crate::backend::module::AsmModule;
use crate::backend::operand::{Reg, ARG_REG_COUNT};
use crate::backend::regalloc::regalloc;
use crate::backend::regalloc::structs::FuncAllocStat;
use crate::backend::regalloc::structs::RegUsedStat;
use crate::backend::{block::*, operand};
use crate::container::bitmap::Bitmap;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::ir_type::IrType;
use crate::utility::{ObjPtr, ScalarType};
use crate::{config, log_file};
pub mod dump;
pub mod handle_call;
pub mod p2v;
pub mod schedule;
pub mod utils;
#[derive(Clone)]
pub struct Func {
    pub is_extern: bool,
    pub label: String,
    pub blocks: Vec<ObjPtr<BB>>,
    pub stack_addr: LinkedList<StackSlot>,
    pub params: Vec<ObjPtr<Inst>>,
    pub param_cnt: (i32, i32), // (int, float)

    pub is_header: bool, //判断一个函数是否是一个模板族下的第一个
    pub entry: Option<ObjPtr<BB>>,

    // fregs: HashSet<Reg>,
    pub context: ObjPtr<Context>,

    pub reg_alloc_info: FuncAllocStat,
    pub spill_stack_map: HashMap<Reg, StackSlot>,

    pub const_array: HashSet<IntArray>,
    pub float_array: HashSet<FloatArray>,
    pub callee_saved: HashSet<Reg>,
    pub array_inst: Vec<ObjPtr<LIRInst>>,
    pub array_slot: Vec<i32>,

    pub tmp_vars: HashSet<Reg>,
}

/// 函数的构造
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
            context,
            is_header: true,

            reg_alloc_info: FuncAllocStat::new(),
            spill_stack_map: HashMap::new(),

            const_array: HashSet::new(),
            float_array: HashSet::new(),
            callee_saved: HashSet::new(),
            array_inst: Vec::new(),
            array_slot: Vec::new(),

            tmp_vars: HashSet::new(),
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
        let mut entry = pool.put_block(BB::new(&format!(".entry_{label}"), &self.label));
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
        let first_block = pool.put_block(BB::new(&label, &self.label));
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
                let block = pool.put_block(BB::new(&label, &self.label));
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
        let mut _size = 0;
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
            _size += block.insts.len();
        }
        self.update(this);
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

        let overflow_param =
            max(0, self.param_cnt.0 - ARG_REG_COUNT) + max(0, self.param_cnt.1 - ARG_REG_COUNT);
        let offset = overflow_param * ADDR_SIZE;
        let slot = StackSlot::new(offset, offset);
        assert!(self.stack_addr.is_empty());
        self.stack_addr.push_front(StackSlot::new(0, 0));
        if offset != 0 {
            self.stack_addr.push_front(slot);
        }
    }

    pub fn get_first_block(&self) -> ObjPtr<BB> {
        self.blocks[1].clone()
    }

    pub fn update_array_offset(&mut self, pool: &mut BackendPool) {
        let slot = self.stack_addr.back().unwrap();
        let base_size = slot.get_pos() + slot.get_size();

        for (i, inst) in self.array_inst.iter().enumerate() {
            //如果类型已经被修改,不是array_inst了,则修改
            let mut offset = match inst.get_rhs() {
                Operand::IImm(imm) => imm.get_data() + base_size,
                _ => unreachable!("array offset must be imm"),
            };
            offset += self.array_slot.iter().take(i).sum::<i32>() - self.array_slot[i];

            if !operand::is_imm_12bs(offset) {
                for block in self.blocks.iter() {
                    let index = match block.insts.iter().position(|i| i == inst) {
                        Some(index) => index,
                        None => continue,
                    };
                    let tmp = Operand::Reg(Reg::new(8, ScalarType::Int));
                    let i = LIRInst::new(
                        InstrsType::OpReg(SingleOp::Li),
                        vec![tmp.clone(), Operand::IImm(IImm::new(offset))],
                    );
                    block.as_mut().insts.insert(index, pool.put_inst(i));
                    inst.as_mut().replace_op(vec![
                        inst.get_dst().clone(),
                        inst.get_lhs().clone(),
                        tmp,
                    ]);
                }
            } else {
                inst.as_mut().replace_op(vec![
                    inst.get_dst().clone(),
                    inst.get_lhs().clone(),
                    Operand::IImm(IImm::new(offset)),
                ]);
            }
        }
    }

    pub fn handle_overflow_br(&mut self, pool: &mut BackendPool) {
        let this = pool.put_func(self.clone());
        for block in self.blocks.iter() {
            block.as_mut().handle_overflow_br(this, pool);
        }
        self.update(this);
    }

    pub fn handle_overflow_sl(&mut self, pool: &mut BackendPool) {
        let this = pool.put_func(self.clone());
        for block in self.blocks.iter() {
            block.as_mut().handle_overflow_sl(this, pool);
        }
        self.update(this);
    }

    fn update(&mut self, func: ObjPtr<Func>) {
        let func_ref = func.as_ref();
        self.is_extern = func.is_extern;
        self.is_header = func.is_header;
        self.blocks = func_ref.blocks.clone();
        self.stack_addr = func_ref.stack_addr.clone();
        self.spill_stack_map = func_ref.spill_stack_map.clone();
        self.const_array = func_ref.const_array.clone();
        self.float_array = func_ref.float_array.clone();
        self.callee_saved = func_ref.callee_saved.clone();
        self.array_inst = func_ref.array_inst.clone();
        self.array_slot = func_ref.array_slot.clone();
    }
    /// 为要保存的callee save寄存器开栈,然后开栈以及处理 callee save的保存和恢复
    pub fn save_callee(&mut self, f: &mut File) {
        let mut callee_map: HashMap<Reg, StackSlot> = HashMap::new();
        if self.label == "main" {
            self.build_stack_info(f, callee_map, true);
            return;
        }
        for id in self.callee_saved.iter() {
            config::record_callee_save_sl(
                &self.label,
                &format!("restore: {}store{}", self.label, id),
            );
            config::record_callee_save_sl(
                &self.label,
                &format!("restore: {}loadback{}", self.label, id),
            );
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

        if let Some(addition_stack_info) = self.stack_addr.front() {
            stack_size += addition_stack_info.get_pos();
        }
        if let Some(slot) = self.stack_addr.back() {
            stack_size += slot.get_pos() + slot.get_size();
        };

        // 局部数组空间
        for array_size in self.array_slot.iter() {
            stack_size += array_size
        }

        //栈对齐 - 调用func时sp需按16字节对齐
        stack_size = stack_size / 16 * 16 + 16;
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
}

///函数的基础功能
impl Func {
    ///无额外约束的计算寄存器活跃区间
    pub fn calc_live_base(&self) {
        let mut queue: VecDeque<(ObjPtr<BB>, Reg)> = VecDeque::new();
        for block in self.blocks.iter() {
            block.as_mut().live_use.clear();
            block.as_mut().live_def.clear();
            for it in block.as_ref().insts.iter().rev() {
                for reg in it.as_ref().get_reg_def().into_iter() {
                    block.as_mut().live_use.remove(&reg);
                    block.as_mut().live_def.insert(reg);
                }
                for reg in it.as_ref().get_reg_use().into_iter() {
                    block.as_mut().live_def.remove(&reg);
                    block.as_mut().live_use.insert(reg);
                }
            }
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
    }
}

///函数汇编生成
impl GenerateAsm for Func {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) {
        if self.const_array.len() > 0 || self.float_array.len() > 0 {
            writeln!(f, "	.data\n   .align  3").unwrap();
        }
        if self.is_header {
            for mut a in self.const_array.clone() {
                a.generate(self.context, f);
            }
            for mut a in self.float_array.clone() {
                a.generate(self.context, f);
            }
        }
        AsmBuilder::new(f).show_func(&self.label);
        self.context.as_mut().call_prologue_event();
        let mut _size = 0;
        for block in self.blocks.iter() {
            _size += block.insts.len();
        }
        // log!("tatol {}", size);
        for block in self.blocks.iter() {
            block.as_mut().generate(self.context, f);
        }
        writeln!(f, "	.size	{}, .-{}", self.label, self.label).unwrap();
    }
}
