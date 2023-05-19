pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::{Result, Write};

use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::ir_type;
use crate::backend::operand::{Reg, IImm, FImm};
use crate::backend::instrs::{LIRInst, InstrsType, SingleOp};
use crate::backend::instrs::Operand;

use super::structs::*;


pub struct BB {
    pub label: String,
    pub called: bool,

    pub insts: Vec<ObjPtr<LIRInst>>,

    pub in_edge: Vec<ObjPtr<BB>>,
    pub out_edge: Vec<ObjPtr<BB>>,

    pub live_use: HashSet<Reg>,
    pub live_def: HashSet<Reg>,
    pub live_in: HashSet<Reg>,
    pub live_out: HashSet<Reg>,

    insts_mpool: ObjPool<LIRInst>,
}

impl BB {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            called: false,
            insts: Vec::new(),
            in_edge: Vec::new(),
            out_edge: Vec::new(),
            live_use: HashSet::new(),
            live_def: HashSet::new(),
            live_in: HashSet::new(),
            live_out: HashSet::new(),
            insts_mpool: ObjPool::new(),
        }
    }

    // 删除BB时清空当前块维护的内存池
    pub fn del(&self) {
        self.insts_mpool.free_all()
    }

    pub fn construct(&mut self, block: ObjPtr<BasicBlock>, next_block: ObjPtr<BB>) {
        let mut ir_block_inst = block.as_ref().get_head_inst();
        loop {
            let inst_ref = ir_block_inst.as_ref();
            // translate ir to lir, use match
            match inst_ref.get_kind() {
                InstKind::Return => {
                    match inst_ref.get_ir_type() {
                        ir_type::IrType::Void => self.insts.push(
                            self.insts_mpool.put(
                                LIRInst::new(InstrsType::Ret(ScalarType::Void), vec![])
                            )
                        ),
                        ir_type::IrType::Int => {
                            let src = inst_ref.get_return_value();
                            let src_operand = self.resolve_operand(src);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::OpReg(SingleOp::Mov), vec![src_operand])
                                )
                            );
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::Ret(ScalarType::Int), vec![])
                                )
                            );
                        },
                        ir_type::IrType::Float => {
                            //TODO:
                        },
                        _ => panic!("cannot reach, Return false")
                    }
                }
                _ => {
                // TODO: ir translation.
                }
            }
            if let Some(ir_block_inst) = ir_block_inst.as_ref().get_next() {} 
            else {
                break;
            }
        }
    }

    pub fn push_back(&mut self, inst: ObjPtr<LIRInst>) {
        self.insts.push(inst);
    }

    pub fn push_back_list(&mut self, inst: &mut Vec<ObjPtr<LIRInst>>) {
        self.insts.append(inst);
    }

    fn resolve_operand(&self, src: ObjPtr<Inst>) -> Operand {
        //TODO: ObjPtr match
        match src.as_ref().get_kind() {
            //TODO: resolve different kind of operand
            InstKind::ConstInt(iimm) => Operand::IImm(IImm::new(iimm)),
            InstKind::ConstFloat(fimm) => Operand::FImm(FImm::new(fimm)),
            // InstKind::GlobalConstInt(_) => resolveGlobalVar(src),
            // InstKind::GlobalConstFloat(_) => resolveGlobalVar(src),
            _ => {
                //TODO:
                panic!("more operand-ir_inst to resolve");
            }
        }
    }


    // fn clear_reg_info(&mut self) {
    //     self.live_def.clear();
    //     self.live_use.clear();
    //     self.live_in.clear();
    //     self.live_out.clear();
    // }
}
impl GenerateAsm for BB {
    fn generate(&self, context: ObjPtr<Context>,f: &mut File) -> Result<()> {
        if self.called {
            writeln!(f, "{}:", self.label)?;
        }

        for inst in self.insts.iter() {
            inst.as_ref().generate(context.clone(), f)?;
        }

        Ok(())
    }
}