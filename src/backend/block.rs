pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::{Result, Write};

use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{Inst, InstKind, BinOp, UnOp};
use crate::ir::ir_type::IrType;
use crate::backend::operand::{Reg, IImm, FImm};
use crate::backend::instrs::{LIRInst, InstrsType, SingleOp, BinaryOp};
use crate::backend::instrs::Operand;

use crate::backend::func::Func;
use crate::backend::operand;
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
    pub fn construct(&mut self, func: &Func, block: ObjPtr<BasicBlock>, next_blocks: Option<ObjPtr<BB>>, map_info: &mut Mapping) {
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
                    let mut dst_reg : Operand = self.resolve_operand(ir_block_inst, true);
                    match op {
                        //TODO: Float Binary
                        BinOp::Add => {
                            let mut inst_kind = InstrsType::Binary(BinaryOp::Add);
                            match lhs.as_ref().get_ir_type() {
                                IrType::ConstInt => {
                                    // 负数范围比正数大，使用subi代替addi
                                    let imm = lhs.as_ref().get_int_bond();
                                    lhs_reg = self.resolve_operand(rhs, true);
                                    if operand::is_imm_12bs(-imm) {
                                        inst_kind = InstrsType::Binary(BinaryOp::Sub);
                                        rhs_reg = self.resolve_iimm(-imm);
                                    } else {
                                        rhs_reg = self.resolve_operand(lhs, false);
                                    }
                                    self.insts.push(
                                        self.insts_mpool.put(
                                            LIRInst::new(inst_kind, vec![dst_reg, lhs_reg, rhs_reg])
                                        )
                                    );
                                },
                                _ => {
                                    lhs_reg = self.resolve_operand(lhs, true);
                                    match rhs.as_ref().get_ir_type() {
                                        IrType::ConstInt => {
                                            let imm = rhs.as_ref().get_int_bond();
                                            if operand::is_imm_12bs(-imm) {
                                                inst_kind = InstrsType::Binary(BinaryOp::Sub);
                                                rhs_reg = self.resolve_iimm(-imm);
                                            }
                                            rhs_reg = self.resolve_iimm(imm);
                                        },
                                        _ => {
                                            rhs_reg = self.resolve_operand(rhs, false);
                                        }
                                    }
                                    self.insts.push(
                                        self.insts_mpool.put(
                                            LIRInst::new(inst_kind, vec![dst_reg, lhs_reg, rhs_reg])
                                        )
                                    );
                                },
                            }
                        },
                        BinOp::Sub => {
                            let mut inst_kind = InstrsType::Binary(BinaryOp::Sub);
                            match lhs.as_ref().get_ir_type() {
                                // 左操作数为常数时，插入mov语句把左操作数存到寄存器中
                                IrType::ConstInt => {
                                    lhs_reg = self.resolve_operand(lhs, true);
                                    let imm = rhs.as_ref().get_int_bond();
                                    let iimm = IImm::new(-imm);
                                    if operand::is_imm_12bs(-imm) {
                                        inst_kind = InstrsType::Binary(BinaryOp::Add);
                                        rhs_reg = self.resolve_iimm(-imm);
                                    } else {
                                        rhs_reg = self.resolve_operand(rhs, false);
                                    }
                                    self.insts.push(
                                        self.insts_mpool.put(
                                            LIRInst::new(inst_kind, vec![dst_reg, lhs_reg, rhs_reg])
                                        )
                                    )
                                },
                                _ => {
                                    lhs_reg = self.resolve_operand(lhs, true);
                                    match rhs.as_ref().get_ir_type() {
                                        IrType::ConstInt => {
                                            let imm = rhs.as_ref().get_int_bond();
                                            let iimm = IImm::new(-imm);
                                            if operand::is_imm_12bs(-imm) {
                                                inst_kind = InstrsType::Binary(BinaryOp::Add);
                                                rhs_reg = self.resolve_iimm(-imm);
                                            } else {
                                                rhs_reg = self.resolve_operand(rhs, false);
                                            }
                                        }
                                        _ => {
                                            rhs_reg = self.resolve_operand(rhs, false);
                                        }
                                    }
                                    self.insts.push(
                                        self.insts_mpool.put(
                                            LIRInst::new(inst_kind, vec![dst_reg, lhs_reg, rhs_reg])
                                        )
                                    );
                                }
                            }
                        },
                        BinOp::Mul => {
                            let mut op_flag = false;
                            let mut src : Operand;
                            let mut imm = 0;
                            if lhs.as_ref().get_ir_type() == IrType::ConstInt || rhs.as_ref().get_ir_type() == IrType::ConstInt {
                                if lhs.as_ref().get_ir_type() == IrType::ConstInt && is_opt_mul(lhs.as_ref().get_int_bond()) {
                                    op_flag = true;
                                    src = self.resolve_operand(rhs, true);
                                    imm = lhs.as_ref().get_int_bond();
                                }
                                if rhs.as_ref().get_ir_type() == IrType::ConstInt && is_opt_mul(rhs.as_ref().get_int_bond()) {
                                    op_flag = true;
                                    src = self.resolve_operand(lhs, true);
                                    imm = rhs.as_ref().get_int_bond();
                                }
                                if op_flag {
                                    self.resolve_opt_mul(dst_reg, src, imm);
                                    break;
                                }
                            }
                            lhs_reg = self.resolve_operand(lhs, true);
                            rhs_reg = self.resolve_operand(rhs, true);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::Binary(BinaryOp::Mul), vec![dst_reg, lhs_reg, rhs_reg])
                                )
                            );
                        },
                        BinOp::Div => {
                            lhs_reg = self.resolve_operand(lhs, true);
                            if rhs.as_ref().get_ir_type() == IrType::ConstInt {
                                self.resolve_opt_div(dst_reg, lhs_reg, rhs.as_ref().get_int_bond());
                            } else {
                                rhs_reg = self.resolve_operand(rhs, true);
                                self.insts.push(
                                    self.insts_mpool.put(
                                        LIRInst::new(InstrsType::Binary(BinaryOp::Div), vec![dst_reg, lhs_reg, rhs_reg])
                                    )
                                );
                                
                            }
                        },
                        BinOp::Rem => {
                            // x % y == x - (x / y) *y
                            // % 0 % 1 % 2^n 特殊判断
                            if rhs.as_ref().get_ir_type() == IrType::ConstInt {
                                let imm = rhs.as_ref().get_int_bond();
                                match imm {
                                    0 => {
                                        lhs_reg = self.resolve_operand(lhs, true);
                                        self.insts.push(
                                            self.insts_mpool.put(
                                                LIRInst::new(InstrsType::OpReg(SingleOp::IMv), vec![dst_reg, lhs_reg])
                                            )
                                        );
                                    },
                                    1 | -1 => {
                                        self.insts.push(
                                            self.insts_mpool.put(
                                                LIRInst::new(InstrsType::OpReg(SingleOp::IMv), vec![dst_reg, Operand::IImm(IImm::new(0))])
                                            )
                                        );
                                    },
                                    _ => {
                                        if is_opt_num(imm) || is_opt_num(-imm) {
                                            //TODO: 
                                            self.resolve_opt_rem(dst_reg, lhs, rhs);
                                        } else {
                                            lhs_reg = self.resolve_operand(lhs, true);
                                            rhs_reg = self.resolve_operand(rhs, false);
                                            self.insts.push(
                                                self.insts_mpool.put(
                                                    LIRInst::new(InstrsType::Binary(BinaryOp::Rem), vec![dst_reg, lhs_reg, rhs_reg])
                                                )
                                            );
                                        }
                                    }
                                }
                            } else {
                                lhs_reg = self.resolve_operand(lhs, true);
                                rhs_reg = self.resolve_operand(rhs, false);
                                self.insts.push(
                                    self.insts_mpool.put(
                                        LIRInst::new(InstrsType::Binary(BinaryOp::Rem), vec![dst_reg, lhs_reg, rhs_reg])
                                    )
                                );
                            }
                        },
                        _ => {
                            //TODO:
                            panic!("more binary op to resolve");
                        }
                    }
                },
                InstKind::Unary(op) => {
                    let dst_reg = self.resolve_operand(ir_block_inst, true);
                    let src = ir_block_inst.as_ref().get_unary_operand();
                    let src_reg = self.resolve_operand(src, false);
                    match op {
                        UnOp::Neg => {
                            match src.as_ref().get_ir_type() {
                                IrType::ConstInt => {
                                    let imm = src.as_ref().get_int_bond();
                                    let iimm = self.resolve_iimm(-imm);
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::Li), vec![dst_reg, iimm])
                                    ))
                                },
                                IrType::Int => {
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::INeg), vec![dst_reg, src_reg])
                                    ))
                                }
                                IrType::Float => {
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::FNeg), vec![dst_reg, src_reg])
                                    ))
                                }
                                _ => { panic!("invalid unary type for neg"); }
                            }
                        },
                        UnOp::Not => {
                            match src.as_ref().get_ir_type() {
                                IrType::ConstInt => {
                                    let imm = src.as_ref().get_int_bond();
                                    let iimm = self.resolve_iimm(!imm);
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::Li), vec![dst_reg, iimm])
                                    ));
                                },
                                IrType::Int => {
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::INot), vec![dst_reg, src_reg])
                                    ));
                                },
                                _ => { panic!("invalid unary type for not"); }
                            }
                        }
                        UnOp::Pos => {
                            match src.as_ref().get_ir_type() {
                                IrType::ConstInt => {
                                    let imm = src.as_ref().get_int_bond();
                                    let iimm = self.resolve_iimm(imm);
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::Li), vec![dst_reg, iimm])
                                    ));
                                },
                                IrType::Int => {
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::IMv), vec![dst_reg, src_reg])
                                    ));
                                },
                                IrType::Float => {
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::FMv), vec![dst_reg, src_reg])
                                    ));
                                },
                                _ => { panic!("invalid unary type for pos"); }
                            }
                        }
                    }
                }
                
                //TODO: load/store float 
                InstKind::Load => {
                    let addr = inst_ref.get_ptr();
                    //TODO: if global var
                    let dst_reg = self.resolve_operand(ir_block_inst, true);
                    let src_reg = self.resolve_operand(addr, false);
                    self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Load, 
                            vec![dst_reg, src_reg, Operand::IImm(IImm::new(0))])));
                },
                InstKind::Store => {
                    let addr = inst_ref.get_dest();
                    let value = inst_ref.get_value();
                    let addr_reg = self.resolve_operand(addr, false);
                    let value_reg = self.resolve_operand(value, true);
                    self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Store, 
                        vec![value_reg, addr_reg])));
                },
                InstKind::Alloca => {
                    //TODO: 数组的优化使用
                    let dst = self.resolve_operand(ir_block_inst, false);
                    let slot = func.stack_addr[func.stack_addr.len()-1];
                    let pos = slot.get_pos() + slot.get_size();
                    let size = inst_ref.get_array_length().as_ref().get_int_bond();
                    func.stack_addr.push(&StackSlot::new(pos, size));
                    self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::StoreToStack, 
                        vec![dst, Operand::IImm(IImm::new(pos))])));
                }
                InstKind::Gep => {
                    // type(8B) * index 
                    let offset = inst_ref.get_offset().as_ref().get_int_bond() * 8;
                    let dst_reg = self.resolve_operand(ir_block_inst, true);
                    let src_reg = self.resolve_operand(inst_ref.get_ptr(), true);
                    //TODO:判断地址合法
                    self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Load, 
                            vec![dst_reg, src_reg, Operand::IImm(IImm::new(offset))])));
                },
                InstKind::Return => {
                    match inst_ref.get_ir_type() {
                        IrType::Void => self.insts.push(
                            self.insts_mpool.put(
                                LIRInst::new(InstrsType::Ret(ScalarType::Void), vec![])
                            )
                        ),
                        IrType::Int => {
                            let src = inst_ref.get_return_value();
                            let src_operand = self.resolve_operand(src, false);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::OpReg(SingleOp::IMv), vec![src_operand])
                                )
                            );
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::Ret(ScalarType::Int), vec![])
                                )
                            );
                        },
                        IrType::Float => {
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

    fn resolve_operand(&self, src: ObjPtr<Inst>, is_left: bool) -> Operand {
        //TODO: ObjPtr match
        if !is_left {
            match src.as_ref().get_kind() {
                //TODO: resolve different kind of operand
                InstKind::ConstInt(iimm) => self.resolve_iimm(iimm),
                InstKind::ConstFloat(fimm) => self.resolve_fimm(fimm),
                // InstKind::GlobalConstInt(_) => resolveGlobalVar(src),
                // InstKind::GlobalConstFloat(_) => resolveGlobalVar(src),
                _ => {
                    //TODO:
                    panic!("more operand-ir_inst to resolve");
                }
            }
        } else {
            match src.as_ref().get_kind() {
                InstKind::ConstInt(iimm) => self.load_iimm_to_ireg(iimm),
                InstKind::ConstFloat(fimm) => self.load_fimm_to_freg(fimm),
                // InstKind::GlobalConstInt(_) => resolveGlobalVar(src),
                // InstKind::GlobalConstFloat(_) => resolveGlobalVar(src),
                _ => {
                    panic!("more operand-ir_inst to resolve");
                }
            }
        }
        
    }

    fn resolve_iimm(&self, imm: i32) -> Operand {
        //TODO: if type > i32
        let res = IImm::new(imm);
        if operand::is_imm_12bs(imm) {
            Operand::IImm(res)
        } else {
            self.load_iimm_to_ireg(imm)
        }
    }

    fn resolve_fimm(&self, imm: f32) -> Operand {
        Operand::FImm(FImm::new(imm))
    }

    fn load_iimm_to_ireg(&self, imm: i32) -> Operand {
        let reg = Operand::Reg(Reg::init(ScalarType::Int));
        let iimm = Operand::IImm(IImm::new(imm));
        if operand::is_imm_12bs(imm) {
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::OpReg(SingleOp::Li), vec![reg, iimm])));
        } else {
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::OpReg(SingleOp::Lui), vec![reg, iimm])));
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Binary(BinaryOp::Add), vec![reg, reg, iimm])));
        }
        reg
    }

    fn load_fimm_to_freg(&self, imm: f32) -> Operand {
        //TODO:
        Operand::FImm(FImm::new(0.0))
    }

    fn resolve_opt_mul(&mut self, dst: Operand, src: Operand, imm: i32) {
        //TODO:

    }

    fn resolve_opt_div(&mut self, dst: Operand, src: Operand, imm: i32) {
        //TODO:
    }

    fn resolve_opt_rem(&mut self, dst: Operand, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) {
        //TODO:
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

fn is_opt_mul(imm: i32) -> bool {
    //TODO:
    false
}

//FIXME: ConstInt instance of i32 not i64?
fn is_opt_num(imm: i32) -> bool {
    (imm & (imm - 1)) == 0
}