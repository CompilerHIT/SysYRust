use std::collections::LinkedList;
pub use std::collections::{HashSet, VecDeque};
use std::vec::Vec;
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
use std::io::Write;
use std::cmp::max;

use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::ir_type::IrType;
use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::backend::operand::{Reg, ARG_REG_COUNT};
use crate::backend::instrs::{LIRInst, Operand};
use crate::backend::asm_builder::AsmBuilder;
use crate::backend::module::AsmModule;
use crate::backend::block::*;
use crate::backend::regalloc::{regalloc::Regalloc, easy_ls_alloc::Allocator, structs::FuncAllocStat};
use super::structs::*;

// #[derive(Clone)]
pub struct Func {
    pub label: String,
    blocks: Vec<ObjPtr<BB>>,
    pub stack_addr: LinkedList<StackSlot>,
    pub params: Vec<ObjPtr<Inst>>,
    pub param_cnt: (i32, i32),  // (int, float)

    pub entry: Option<ObjPtr<BB>>,

    reg_def: Vec<HashSet<CurInstrInfo>>,
    reg_use: Vec<HashSet<CurInstrInfo>>,
    reg_num: i32,
    // fregs: HashSet<Reg>,

    pub context: Context,

    blocks_mpool: ObjPool<BB>,
    pub reg_alloc_info: FuncAllocStat,
    pub spill_stack_map: HashMap<i32, StackSlot>,

    pub const_array: HashSet<IntArray>
}


/// reg_num, stack_addr, caller_stack_addr考虑借助回填实现
/// 是否需要caller_stack_addr？caller函数sp保存在s0中
impl Func {
    pub fn new(name: &str) -> Self {
        Self {
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

            context: Context::new(),

            blocks_mpool: ObjPool::new(),

            reg_alloc_info: FuncAllocStat::new(),
            spill_stack_map: HashMap::new(),
            const_array: HashSet::new(),
        }
    }

    // 或许不会删除函数？
    pub fn del(&mut self) {
        self.blocks_mpool.free_all()
    }

    pub fn construct(&mut self, module: &AsmModule, ir_func: &Function, func_seq: i32) {
        //FIXME: temporary
        // more infos to add
        let mut info = Mapping::new();

        // 处理全局变量
        let globl = &module.upper_module.global_variable;
        globl.iter().for_each(|(name, val)| {
            info.val_map.insert(val.clone(), Operand::Addr(name.to_string()));
        });
        
        // entry shouldn't generate for asm, called label for entry should always be false
        let label = &self.label;
        let entry = self.blocks_mpool.put(BB::new(&format!(".entry_{label}")));
        self.entry = Some(entry);
        self.blocks.push(self.entry.unwrap());

        // 第一遍pass
        let fblock = ir_func.get_head();
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
        self.handle_parameters(ir_func);
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
        let ra = Reg::new(1, ScalarType::Int);
        
        self.calc_live();
        println!("cal live end");
        let mut allocator = Allocator::new();
        println!("start alloc");
        let alloc_stat = allocator.alloc(self);
        println!("alloc end");

        self.reg_alloc_info = alloc_stat;
        self.context.set_reg_map(&self.reg_alloc_info.dstr);
        println!("stack_size: {}", self.reg_alloc_info.stack_size);
        println!("alloc result: {:?}", self.reg_alloc_info.dstr);

        let mut stack_size = self.reg_alloc_info.stack_size as i32;
        if let Some(addition_stack_info) = self.stack_addr.front() {
            stack_size += addition_stack_info.get_pos() + addition_stack_info.get_size();
        }
        
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
            offset -= 8;
            builder.s(&ra.to_string(), "sp", offset, false, true);
        });

        let mut offset = stack_size;
        self.context.set_epilogue_event(move||{
            let mut builder = AsmBuilder::new(&mut f2);
            offset -= 8;
            builder.l(&ra.to_string(), "sp", offset, false, true);
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
        //TODO:
        let mut iparam : Vec<_> = ir_func.get_params().iter()
            .filter(|(_, param)| param.as_ref().get_param_type() == IrType::Int)
            .map(|(_, param)| param.clone()).collect();
        let mut fparam : Vec<_> = ir_func.get_params().iter()
            .filter(|(_, param)| param.as_ref().get_param_type() == IrType::Float)
            .map(|(_, param)| param.clone()).collect();
        self.param_cnt = (iparam.len() as i32, fparam.len() as i32);
        self.params.append(&mut iparam);
        self.params.append(&mut fparam);

        let mut offset = 0;
        let overflow_param = max(0, self.param_cnt.0 - ARG_REG_COUNT) + max(0, self.param_cnt.1 - ARG_REG_COUNT);
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

    pub fn handle_spill(&mut self) {
        for block in self.blocks.iter() {
            let pos = match self.reg_alloc_info.bb_stack_sizes.get(&block) {
                Some(pos) => {
                    *pos as i32
                },
                None => continue,
            };
            block.as_mut().handle_spill(ObjPtr::new(&self), &self.reg_alloc_info.spillings, pos);
        }
    }
}

impl GenerateAsm for Func {
    fn generate(&mut self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        let context = ObjPtr::new(&self.context);
        for mut a in self.const_array.clone() {
            a.generate(context, f)?;
        }
        AsmBuilder::new(f).show_func(&self.label)?;
        self.context.call_prologue_event();
        for block in self.blocks.iter() {
            block.as_mut().generate(context, f)?;
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