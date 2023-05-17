pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::{Result, Write};

use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::backend::operand::Reg;
use crate::backend::instrs::LIRInst;
use crate::backend::asm_builder::AsmBuilder;
use crate::backend::module::AsmModule;
use crate::backend::block::*;
use super::structs::*;

// #[derive(Clone)]
pub struct Func {
    label: String,
    blocks: Vec<ObjPtr<BB>>,
    stack_addr: Vec<ObjPtr<StackSlot>>,
    caller_stack_addr: Vec<ObjPtr<StackSlot>>,
    params: Vec<ObjPtr<Reg>>,
    entry: Option<BB>,

    reg_def: Vec<HashSet<CurInstrInfo>>,
    reg_use: Vec<HashSet<CurInstrInfo>>,
    fregs: HashSet<Reg>,

    context: Option<ObjPtr<Context>>,

    blocks_mpool: ObjPool<BB>,
}



impl Func {
    pub fn new(name: &str) -> Self {
        Self {
            label: name.to_string(),
            blocks: Vec::new(),
            stack_addr: Vec::new(),
            caller_stack_addr: Vec::new(),
            params: Vec::new(),
            entry: None,
            reg_def: Vec::new(),
            reg_use: Vec::new(),
            fregs: HashSet::new(),

            context: None,

            blocks_mpool: ObjPool::new(),
        }
    }

    // 或许不会删除函数？
    pub fn del(&self) {
        self.blocks_mpool.free_all()
    }

    pub fn construct(&mut self, module: &AsmModule) {
        //FIXME: temporary
        // more infos to add
        self.entry = Some(BB::new(&self.label));
        module.push_block(&self.entry.unwrap());
        // 需要遍历block的接口
        // self.borrow_mut().blocks.push(Pointer::new(self.entry));
            
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

    pub fn allocate_reg(&mut self, f: &'static mut File) {
        // 函数返回地址保存在ra中
        //FIXME: 暂时使用固定的寄存器ra、a0与s0，即r1, r8, r10
        //FIXME:暂时只考虑int型
        let reg_int = vec![Reg::new(1, ScalarType::Int), Reg::new(8, ScalarType::Int)];

        let mut stack_size = 0;
        for it in self.stack_addr.iter().rev() {
            it.as_mut().set_pos(stack_size);
            stack_size += it.as_ref().get_size();
        }

        let mut reg_int_res = Vec::from(reg_int);
        let mut reg_int_res_cl = reg_int_res.clone();
        let reg_int_size = reg_int_res.len();
        
        //TODO:栈对齐 - 调用func时sp需按16字节对齐

        let mut offset = stack_size;
        let mut f1 = f.try_clone().unwrap();
        if let Some(contxt) = &self.context {
            contxt.as_mut().set_prologue_event(move||{
                let mut builder = AsmBuilder::new(f);
                // addi sp -stack_size
                builder.addi("sp", "sp", -offset);
                for src in reg_int_res.iter() {
                    offset -= 8;
                    builder.sd(&src.to_string(), "sp", offset, false);
                }
            });
            let mut offset = stack_size;
            contxt.as_mut().set_epilogue_event(move||{
                let mut builder = AsmBuilder::new(&mut f1);
                for src in reg_int_res_cl.iter() {
                    offset -= 8;
                    builder.ld("sp", &src.to_string(), offset, false);
                }
                builder.addi("sp", "sp", offset);
            });
        }
        

        //TODO: for caller
        // let mut pos = stack_size + reg_int_size as i32 * 8;
        // for caller in self.caller_stack_addr.iter() {
        //     caller.borrow_mut().set_pos(pos);
        //     pos += caller.borrow().get_size();
        // }
        
    }
}

impl GenerateAsm for Func {
    fn generate(&self, _: ObjPtr<Context>, f: &mut File) -> Result<()> {
        AsmBuilder::new(f).show_func(&self.label);
        if let Some(contxt) = &self.context {
            contxt.as_mut().call_prologue_event();
            for block in self.blocks.iter() {
                block.as_ref().generate(contxt.clone(), f)?;
            }
        }
        Ok(())
    }
}