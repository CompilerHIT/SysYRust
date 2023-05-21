pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::{Result, Write};

use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{Inst, InstKind};
use crate::ir::ir_type;
use crate::backend::operand::{Reg, IImm, FImm};
use crate::backend::instrs::{LIRInst, InstrsType, SingleOp, BinaryOp};
use crate::backend::instrs::Operand;

use crate::backend::func::Func;
use crate::backend::operand::ImmBs;
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

    /// 寄存器分配时决定开栈大小、栈对象属性(size(4/8 bytes), pos)，回填func的stack_addr
    /// 尽量保证程序顺序执行，并满足首次遇分支向后跳转的原则？
    //FIXME: b型指令长跳转(目标地址偏移量为+-4KiB)，若立即数非法是否需要增添一个jal块实现间接跳转？
    pub fn construct(&mut self, func: &Func, block: ObjPtr<BasicBlock>, next_blocks: Option<ObjPtr<BB>>, map_info: &Mapping) {
        let mut ir_block_inst = block.as_ref().get_head_inst();
        loop {
            let inst_ref = ir_block_inst.as_ref();
            // translate ir to lir, use match
            match inst_ref.get_kind() {
                InstKind::Binary(op) => {
                    let lhs = inst_ref.get_lhs();
                    let rhs = inst_ref.get_rhs();
                    let mut lhs_reg : Operand;
                    let mut rhs_reg : Operand;
                    let mut dst_reg : Operand;
                    match op {
                        Add => {
                            let inst_kind = InstrsType::Binary(super::instrs::BinaryOp::Add);
                            match lhs.as_ref().get_ir_type() {
                                ir_type::IrType::ConstInt => {
                                    // 负数范围比正数大，使用subi代替addi
                                    let imm = lhs.as_ref().get_int_bond();
                                    let iimm = IImm::new(-imm);
                                    if iimm.is_imm_12bs() {
                                        rhs_reg = self.resolve_iimm(-imm);
                                    }
                                },
                                _ => rhs_reg = self.resolve_operand(lhs),
                            }
                        }
                    }
                }
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
            if ir_block_inst == block.as_ref().get_tail_inst() {
                break;
            }
            ir_block_inst = ir_block_inst.as_ref().get_next();
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

impl PartialEq for BB {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label
    }
}

impl Eq for BB {}

impl Hash for BB {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.label.hash(state);
    }
}
