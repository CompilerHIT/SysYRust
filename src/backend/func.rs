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
use crate::backend::block::*;
use crate::backend::instrs::{LIRInst, Operand};
use crate::backend::module::AsmModule;
use crate::backend::operand::{Reg, ARG_REG_COUNT};
use crate::backend::regalloc::{
    easy_ls_alloc::Allocator, regalloc::Regalloc, structs::FuncAllocStat,
};
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::ir_type::IrType;
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
    pub spill_stack_map: HashMap<i32, StackSlot>,

    pub const_array: HashSet<IntArray>,
    pub floats: Vec<(String, f32)>,
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
            floats: Vec::new(),
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
        let mut visited : HashSet<ObjPtr<BasicBlock>> = HashSet::new();
        while let Some(fblock) = tmp.pop_front() {
            let next_blocks = fblock.as_ref().get_next_bb();
            println!("next_blocks len: {:?}", next_blocks.len());
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
                println!("start build block");
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
            loop {
                match block.insts[index].get_type() {
                    InstrsType::Ret(..) | InstrsType::Branch(..) | InstrsType::Jump => {
                        if index == 0 {
                            break;
                        }
                        index -= 1;
                    }
                    _ => {
                        break;
                    }
                }
            }
            if index != 0 {
                index += 1;
            }
            if let Some(mut target) = info.phis_to_block.get_mut(&block.label) {
                for inst in target.iter() {
                    println!("label: {}", block.label);
                    println!("insert phi to last: {:?}", inst);
                    block.as_mut().insts.insert(index, *inst);
                }
            }
            let mut phis = block.phis.clone();
            while let Some(inst) = phis.pop() {
                block.as_mut().insts.insert(0, inst);
            }
            size += block.insts.len();
        }
        println!("phi insert size: {}", size);

        for block in self.blocks.iter() {
            println!("-----------------");
            println!("block: {:?}", block.label);
            for inst in block.insts.iter() {
                println!("row inst: {:?}", inst);
            }
        }

        // 第四遍pass，建立出入边关系
        for block in self.blocks.iter() {
            println!("-----------------");
            println!("{}", block.label);
            for out in block.out_edge.iter() {
                println!("out: {}", out.label);
                println!("reg info:\ndef: {:?}\nuse: {:?}\nin: {:?}\n out: {:?}\n", out.live_def, out.live_use, out.live_in, out.live_out);
            }
            for in_edge in block.in_edge.iter() {
                println!("in: {}", in_edge.label);
                println!("reg info:\ndef: {:?}\nuse: {:?}\nin: {:?}\n out: {:?}\n", in_edge.live_def, in_edge.live_use, in_edge.live_in, in_edge.live_out);
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
        let mut queue: VecDeque<(ObjPtr<BB>, Reg)> = VecDeque::new();
        for block in self.blocks.iter() {
            block.as_mut().live_use.clear();
            block.as_mut().live_def.clear();
            for it in block.as_ref().insts.iter().rev() {
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
            for reg in block.as_ref().live_use.iter() {
                queue.push_back((block.clone(), reg.clone()));
            }
            block.as_mut().live_in = block.as_ref().live_use.clone();
            block.as_mut().live_out.clear();
        }
        while let Some(value) = queue.pop_front() {
            let (block, reg) = value;
            for pred in block.as_ref().in_edge.iter() {
                if pred.as_mut().live_out.insert(reg) {
                    if pred.as_mut().live_def.take(&reg) == None
                        && pred.as_mut().live_in.insert(reg)
                    {
                        queue.push_back((pred.clone(), reg));
                    }
                }
            }
        }
    }

    pub fn allocate_reg(&mut self, f: &mut File) {
        // 函数返回地址保存在ra中
        let ra = Reg::new(1, ScalarType::Int);

        self.calc_live();
        let mut allocator = Allocator::new();
        let alloc_stat = allocator.alloc(self);

        self.reg_alloc_info = alloc_stat;
        self.context.as_mut().set_reg_map(&self.reg_alloc_info.dstr);
        println!("dstr map info{:?}", self.reg_alloc_info.dstr);

        let mut stack_size = self.reg_alloc_info.stack_size as i32;
        println!("stack_size: {}", stack_size);
        if let Some(addition_stack_info) = self.stack_addr.front() {
            stack_size += addition_stack_info.get_pos() + addition_stack_info.get_size();
        }

        //栈对齐 - 调用func时sp需按16字节对齐
        stack_size = stack_size / 16 * 16 + 16;

        self.context.as_mut().set_offset(stack_size - 8);
        let mut f1 = match f.try_clone() {
            Ok(f) => f,
            Err(e) => panic!("Error: {}", e),
        };
        let mut f2 = match f.try_clone() {
            Ok(f) => f,
            Err(e) => panic!("Error: {}", e),
        };
        let map = self.context.get_reg_map().clone();
        self.context.as_mut().set_prologue_event(move || {
            let mut builder = AsmBuilder::new(&mut f1);
            // addi sp -stack_size
            builder.addi("sp", "sp", -stack_size);
            builder.s(&ra.to_string(), "sp", stack_size - 8, false, true);
        });

        self.context.as_mut().set_epilogue_event(move || {
            let mut builder = AsmBuilder::new(&mut f2);
            builder.l(&ra.to_string(), "sp", stack_size - 8, false, true);
            builder.addi("sp", "sp", stack_size);
        });

        //TODO: for caller
        // let mut pos = stack_size + reg_int_size as i32 * 8;
        // for caller in self.caller_stack_addr.iter() {
        //     caller.borrow_mut().set_pos(pos);
        //     pos += caller.borrow().get_size();
        // }
    }

    fn handle_parameters(&mut self, ir_func: &Function) {
        let mut iparam: Vec<_> = ir_func
            .get_params()
            .iter()
            .filter(|(_, param)| param.as_ref().get_param_type() == IrType::Int)
            .map(|(_, param)| param.clone())
            .collect();
        let mut fparam: Vec<_> = ir_func
            .get_params()
            .iter()
            .filter(|(_, param)| param.as_ref().get_param_type() == IrType::Float)
            .map(|(_, param)| param.clone())
            .collect();
        self.param_cnt = (iparam.len() as i32, fparam.len() as i32);
        self.params.append(&mut iparam);
        self.params.append(&mut fparam);

        let mut offset = 0;
        let overflow_param =
            max(0, self.param_cnt.0 - ARG_REG_COUNT) + max(0, self.param_cnt.1 - ARG_REG_COUNT);
        if overflow_param % 2 == 1 {
            offset = (overflow_param + 1) * 4;
        } else {
            offset = overflow_param * 4;
        }
        let slot = StackSlot::new(0, offset);
        assert!(self.stack_addr.is_empty());
        self.stack_addr.push_front(slot);
    }

    pub fn get_first_block(&self) -> ObjPtr<BB> {
        self.blocks[1].clone()
    }

    pub fn handle_spill(&mut self, pool: &mut BackendPool) {
        let this = pool.put_func(self.clone());
        for block in self.blocks.iter() {
            let pos = match self.reg_alloc_info.bb_stack_sizes.get(&block) {
                Some(pos) => *pos as i32,
                None => continue,
            };
            block
                .as_mut()
                .handle_spill(this, &self.reg_alloc_info.spillings, pos, pool);
        }
        self.update(this);
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
    }
}

impl GenerateAsm for Func {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        if self.is_extern {
            return Ok(());
        }
        if self.const_array.len() > 0 {
            writeln!(f, "	.section	.data\n   .align  3")?;
        }
        for mut a in self.const_array.clone() {
            a.generate(self.context, f)?;
        }
        if self.floats.len() > 0 {
            println!("generate float");
            writeln!(f, "   .data")?;
        }
        for (name, data) in self.floats.clone() {
            writeln!(f, "{name}:  .float  {data}")?;
        }
        AsmBuilder::new(f).show_func(&self.label)?;
        self.context.as_mut().call_prologue_event();
        let mut size = 0;
        for block in self.blocks.iter() {
            size += block.insts.len();
        }
        println!("tatol {}", size);
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
