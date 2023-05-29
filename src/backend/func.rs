use std::collections::LinkedList;
pub use std::collections::{HashSet, VecDeque};
use std::vec::Vec;
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
use std::io::Write;

use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::backend::operand::Reg;
use crate::backend::instrs::LIRInst;
use crate::backend::asm_builder::AsmBuilder;
use crate::backend::module::AsmModule;
use crate::backend::block::*;
use crate::backend::regalloc::{regalloc::Regalloc, easy_ls_alloc::Allocator};
use super::structs::*;

// #[derive(Clone)]
pub struct Func {
    label: String,
    blocks: Vec<ObjPtr<BB>>,
    pub stack_addr: LinkedList<StackSlot>,
    pub callee_stack_addr: LinkedList<StackSlot>,
    pub params: Vec<ObjPtr<Inst>>,
    pub param_cnt: (i32, i32),  // (int, float)

    pub entry: Option<ObjPtr<BB>>,

    reg_def: Vec<HashSet<CurInstrInfo>>,
    reg_use: Vec<HashSet<CurInstrInfo>>,
    reg_num: i32,
    fregs: HashSet<Reg>,

    pub context: Context,

    blocks_mpool: ObjPool<BB>,
}


/// reg_num, stack_addr, caller_stack_addr考虑借助回填实现
/// 是否需要caller_stack_addr？caller函数sp保存在s0中
impl Func {
    pub fn new(name: &str) -> Self {
        Self {
            label: name.to_string(),
            blocks: Vec::new(),
            stack_addr: LinkedList::new(),
            callee_stack_addr: LinkedList::new(),
            params: Vec::new(),
            param_cnt: (0, 0),
            entry: None,
            reg_def: Vec::new(),
            reg_use: Vec::new(),
            reg_num: 0,
            fregs: HashSet::new(),

            context: Context::new(),

            blocks_mpool: ObjPool::new(),
        }
    }

    // 或许不会删除函数？
    pub fn del(&mut self) {
        self.blocks_mpool.free_all()
    }

    pub fn construct(&mut self, module: &AsmModule, ir_func: &Function, func_seq: i32, ) {
        //FIXME: temporary
        // more infos to add
        let mut info = Mapping::new();
        
        // entry shouldn't generate for asm, called label for entry should always be false
        let label = &self.label;
        let entry = self.blocks_mpool.put(BB::new(&format!(".entry_{label}")));
        self.entry = Some(entry);
        self.blocks.push(self.entry.unwrap());

        // 第一遍pass
        let mut fblock = ir_func.get_head();
        let mut ir_block_set: HashSet<ObjPtr<BasicBlock>> = HashSet::new();
        let first_block = self.blocks_mpool.put(BB::new(&label));
        info.ir_block_map.insert(fblock,first_block);
        info.block_ir_map.insert(first_block, fblock);
        ir_block_set.insert(fblock);

        let mut tmp = VecDeque::new();
        tmp.push_back(fblock);
        
        let mut block_seq = 0;
        self.blocks.push(first_block);
        
        while let Some(fblock) = tmp.pop_front() {
            let next_blocks = fblock.as_ref().get_next_bb();
            next_blocks.iter().for_each(|block|tmp.push_back(block.clone()));
            if block_seq == 0 {
                block_seq += 1;
                continue;
            }
            if ir_block_set.insert(fblock) {
                let label = format!(".LBB{func_seq}_{block_seq}");
                let block = self.blocks_mpool.put(BB::new(&label));
                info.ir_block_map.insert(fblock, block);
                info.block_ir_map.insert(block, fblock);
                self.blocks.push(block);
                block_seq += 1;
            }
        }
        self.handle_parameters();
        // 第二遍pass
        let first_block = info.ir_block_map.get(&ir_func.get_head()).unwrap();
        self.entry.unwrap().as_mut().out_edge.push(*first_block);
        first_block.as_mut().in_edge.push(self.entry.unwrap());
        let mut i = 0;

        self.blocks.iter().for_each(|block| {
            if *block != self.entry.unwrap() {
                let basicblock = info.block_ir_map.get(block).unwrap();
                if i + 1 < self.blocks.len() {
                    let next_block = Some(self.blocks[i + 1]);
                    block.as_mut().construct(ObjPtr::new(self), *basicblock, next_block, &mut info);
                } else {
                    block.as_mut().construct(ObjPtr::new(self) , *basicblock, None, &mut info);
                }
                i += 1;
            }
        });

        self.stack_addr = info.stack_slot_set.clone();
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
        self.reg_def.clear();   self.reg_use.clear();
        self.reg_def.resize(self.reg_num as usize, HashSet::new());
        self.reg_use.resize(self.reg_num as usize, HashSet::new());
        let mut p : CurInstrInfo = CurInstrInfo::new(0);
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
        let mut queue : VecDeque<(ObjPtr<BB>, Reg)> = VecDeque::new();
        for block in self.blocks.clone().iter() {
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
                    if pred.as_mut().live_def.take(&reg) == None && pred.as_mut().live_in.insert(reg) {
                        queue.push_back((pred.clone(), reg));
                    }
                }
            }
        }
    }

    pub fn allocate_reg(&mut self, f: &mut File) {
        // 函数返回地址保存在ra中
        let reg_int = vec![Reg::new(1, ScalarType::Int)];

        self.calc_live();
        let mut allocator = Allocator::new();
        let alloc_stat = allocator.alloc(self);
        self.context.set_reg_map(&alloc_stat.dstr);

        let mut stack_size = alloc_stat.stack_size as i32;
        
        let mut reg_int_res = Vec::from(reg_int);
        let mut reg_int_res_cl = reg_int_res.clone();
        let reg_int_size = reg_int_res.len();
        
        //栈对齐 - 调用func时sp需按16字节对齐
        stack_size = stack_size / 16 * 16 + 16;

        let mut offset = stack_size;
        let mut f1 = match f.try_clone() {
            Ok(f) => f,
            Err(e) => panic!("Error: {}", e),
        };
        let mut f2 = match f.try_clone() {
            Ok(f) => f,
            Err(e) => panic!("Error: {}", e),
        };
        self.context.set_prologue_event(move||{
            let mut builder = AsmBuilder::new(&mut f1);
            // addi sp -stack_size
            builder.addi("sp", "sp", -offset);
            for src in reg_int_res.iter() {
                offset -= 8;
                builder.s(&src.to_string(), "sp", offset, false, true);
            }
        });

        let mut offset = stack_size;
        self.context.set_epilogue_event(move||{
            let mut builder = AsmBuilder::new(&mut f2);
            for src in reg_int_res_cl.iter() {
                offset -= 8;
                builder.l("sp", &src.to_string(), offset, false, true);
            }
            builder.addi("sp", "sp", stack_size);
        });
        

        //TODO: for caller
        // let mut pos = stack_size + reg_int_size as i32 * 8;
        // for caller in self.caller_stack_addr.iter() {
        //     caller.borrow_mut().set_pos(pos);
        //     pos += caller.borrow().get_size();
        // }
        
    }

    fn handle_parameters(&mut self) {
        //TODO:
    }

    pub fn get_first_block(&self) -> ObjPtr<BB> {
        self.blocks[1].clone()
    }
}

impl GenerateAsm for Func {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        AsmBuilder::new(f).show_func(&self.label)?;
        self.context.call_prologue_event();
        for block in self.blocks.iter() {
            block.as_mut().generate(ObjPtr::new(&self.context), f)?;
        }
        writeln!(f, "	.size	{}, .-{}:", self.label, self.label)?;
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

impl Hash for ObjPtr<BB> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().label.hash(state);
    }
}
impl Hash for ObjPtr<BasicBlock> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().get_name().hash(state);
    }
}