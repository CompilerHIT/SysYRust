pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
use std::io::prelude::*;
use std::cmp::max;

use crate::utility::{ScalarType, ObjPool, ObjPtr};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{Inst, InstKind, BinOp, UnOp};
use crate::ir::ir_type::IrType;
use crate::backend::operand::{Reg, IImm, FImm};
use crate::backend::instrs::{LIRInst, InstrsType, SingleOp, BinaryOp, CmpOp};
use crate::backend::instrs::Operand;

use crate::backend::func::Func;
use crate::backend::operand;
use super::operand::{REG_COUNT, ARG_REG_COUNT, ToString};
use super::{structs::*, FILE_PATH};

pub static mut ARRAY_NUM: i32 = 0;

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
    pub fn del(&mut self) {
        self.insts_mpool.free_all()
    }

    /// 寄存器分配时决定开栈大小、栈对象属性(size(4/8 bytes), pos)，回填func的stack_addr
    /// 尽量保证程序顺序执行，并满足首次遇分支向后跳转的原则？
    //FIXME: b型指令长跳转(目标地址偏移量为+-4KiB)，若立即数非法是否需要增添一个jal块实现间接跳转？
    pub fn construct(&mut self, block: ObjPtr<BasicBlock>, next_blocks: Option<ObjPtr<BB>>, map_info: &mut Mapping) {
        let mut ir_block_inst = block.as_ref().get_head_inst();
        loop {
            let inst_ref = ir_block_inst.as_ref();
            println!("inst_ref: {:?}", inst_ref.get_kind());
            // translate ir to lir, use match
            match inst_ref.get_kind() {
                InstKind::Binary(op) => {
                    let lhs = inst_ref.get_lhs();
                    let rhs = inst_ref.get_rhs();
                    let mut lhs_reg : Operand = Operand::IImm(IImm::new(0));
                    let mut rhs_reg : Operand = Operand::IImm(IImm::new(0));
                    let mut dst_reg : Operand = self.resolve_operand(ir_block_inst, true, map_info);
                    match op {
                        //TODO: Float Binary
                        //TODO: 判断右操作数是否为常数，若是则使用i型指令
                        BinOp::Add => {
                            let mut inst_kind = InstrsType::Binary(BinaryOp::Add);
                            match lhs.as_ref().get_ir_type() {
                                IrType::ConstInt => {
                                    // 负数范围比正数大，使用subi代替addi
                                    let imm = lhs.as_ref().get_int_bond();
                                    lhs_reg = self.resolve_operand(rhs, true, map_info);
                                    if operand::is_imm_12bs(-imm) {
                                        inst_kind = InstrsType::Binary(BinaryOp::Sub);
                                        rhs_reg = self.resolve_iimm(-imm);
                                    } else {
                                        rhs_reg = self.resolve_operand(lhs, false, map_info);
                                    }
                                    self.insts.push(
                                        self.insts_mpool.put(
                                            LIRInst::new(inst_kind, vec![dst_reg, lhs_reg, rhs_reg])
                                        )
                                    );
                                },
                                _ => {
                                    lhs_reg = self.resolve_operand(lhs, true, map_info);
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
                                            rhs_reg = self.resolve_operand(rhs, false, map_info);
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
                                    lhs_reg = self.resolve_operand(lhs, true, map_info);
                                    let imm = rhs.as_ref().get_int_bond();
                                    let iimm = IImm::new(-imm);
                                    if operand::is_imm_12bs(-imm) {
                                        inst_kind = InstrsType::Binary(BinaryOp::Add);
                                        rhs_reg = self.resolve_iimm(-imm);
                                    } else {
                                        rhs_reg = self.resolve_operand(rhs, false, map_info);
                                    }
                                    self.insts.push(
                                        self.insts_mpool.put(
                                            LIRInst::new(inst_kind, vec![dst_reg, lhs_reg, rhs_reg])
                                        )
                                    )
                                },
                                _ => {
                                    lhs_reg = self.resolve_operand(lhs, true, map_info);
                                    match rhs.as_ref().get_ir_type() {
                                        IrType::ConstInt => {
                                            let imm = rhs.as_ref().get_int_bond();
                                            let iimm = IImm::new(-imm);
                                            if operand::is_imm_12bs(-imm) {
                                                inst_kind = InstrsType::Binary(BinaryOp::Add);
                                                rhs_reg = self.resolve_iimm(-imm);
                                            } else {
                                                rhs_reg = self.resolve_operand(rhs, false, map_info);
                                            }
                                        }
                                        _ => {
                                            rhs_reg = self.resolve_operand(rhs, false, map_info);
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
                            let mut src : Operand = Operand::IImm(IImm::new(0));
                            let mut imm = 0;
                            if lhs.as_ref().get_ir_type() == IrType::ConstInt || rhs.as_ref().get_ir_type() == IrType::ConstInt {
                                if lhs.as_ref().get_ir_type() == IrType::ConstInt && is_opt_mul(lhs.as_ref().get_int_bond()) {
                                    op_flag = true;
                                    src = self.resolve_operand(rhs, true, map_info);
                                    imm = lhs.as_ref().get_int_bond();
                                }
                                if rhs.as_ref().get_ir_type() == IrType::ConstInt && is_opt_mul(rhs.as_ref().get_int_bond()) {
                                    op_flag = true;
                                    src = self.resolve_operand(lhs, true, map_info);
                                    imm = rhs.as_ref().get_int_bond();
                                }
                                if op_flag {
                                    self.resolve_opt_mul(dst_reg, src, imm);
                                    break;
                                }
                            }
                            lhs_reg = self.resolve_operand(lhs, true, map_info);
                            rhs_reg = self.resolve_operand(rhs, true, map_info);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::Binary(BinaryOp::Mul), vec![dst_reg, lhs_reg, rhs_reg])
                                )
                            );
                        },
                        BinOp::Div => {
                            lhs_reg = self.resolve_operand(lhs, true, map_info);
                            if rhs.as_ref().get_ir_type() == IrType::ConstInt {
                                self.resolve_opt_div(dst_reg, lhs_reg, rhs.as_ref().get_int_bond());
                            } else {
                                rhs_reg = self.resolve_operand(rhs, true, map_info);
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
                                        lhs_reg = self.resolve_operand(lhs, true, map_info);
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
                                            lhs_reg = self.resolve_operand(lhs, true, map_info);
                                            rhs_reg = self.resolve_operand(rhs, false, map_info);
                                            self.insts.push(
                                                self.insts_mpool.put(
                                                    LIRInst::new(InstrsType::Binary(BinaryOp::Rem), vec![dst_reg, lhs_reg, rhs_reg])
                                                )
                                            );
                                        }
                                    }
                                }
                            } else {
                                lhs_reg = self.resolve_operand(lhs, true, map_info);
                                rhs_reg = self.resolve_operand(rhs, false, map_info);
                                self.insts.push(
                                    self.insts_mpool.put(
                                        LIRInst::new(InstrsType::Binary(BinaryOp::Rem), vec![dst_reg, lhs_reg, rhs_reg])
                                    )
                                );
                            }
                        },
                        BinOp::And => {
                            lhs_reg = self.resolve_operand(lhs, true, map_info);
                            rhs_reg = self.resolve_operand(rhs, false, map_info);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::Binary(BinaryOp::And), vec![dst_reg, lhs_reg, rhs_reg])
                                )
                            );
                        },
                        BinOp::Or => {
                            lhs_reg = self.resolve_operand(lhs, true, map_info);
                            rhs_reg = self.resolve_operand(rhs, false, map_info);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::Binary(BinaryOp::Or), vec![dst_reg, lhs_reg, rhs_reg])
                                )
                            );
                        },
                        _ => {
                            //TODO:
                            unreachable!("more binary op to resolve");
                        }
                    }
                },
                InstKind::Unary(op) => {
                    let dst_reg = self.resolve_operand(ir_block_inst, true, map_info);
                    let src = ir_block_inst.as_ref().get_unary_operand();
                    let src_reg = self.resolve_operand(src, false, map_info);
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
                    let dst_reg = self.resolve_operand(ir_block_inst, true, map_info);
                    let src_reg = self.resolve_operand(addr, false, map_info);
                    self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Load, 
                            vec![dst_reg, src_reg, Operand::IImm(IImm::new(0))])));
                },
                InstKind::Store => {
                    let addr = inst_ref.get_dest();
                    let value = inst_ref.get_value();
                    let addr_reg = self.resolve_operand(addr, false, map_info);
                    let value_reg = self.resolve_operand(value, true, map_info);
                    self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Store, 
                        vec![value_reg, addr_reg])));
                },
                //FIXME:获取数组名
                InstKind::Alloca => {
                    unsafe {
                        let label = format!(".LC{ARRAY_NUM}");
                        ARRAY_NUM += 1;
                        //将发生分配的数组装入map_info中：记录数组结构、占用栈空间
                        //TODO:la dst label    sd dst (offset)sp
                        //TODO: 大数组而装填因子过低的压缩问题
                        //FIXME: 未考虑数组全零数组，仅考虑int数组
                        let size = inst_ref.get_array_length().as_ref().get_int_bond();
                        let alloca = IntArray::new(label.clone(), size, true,
                                                             inst_ref.get_int_init().clone());
                        let last = map_info.stack_slot_set.front().unwrap();
                        let pos = last.get_pos() + last.get_size();
                        map_info.stack_slot_set.push_front(StackSlot::new(pos, (size * 4 / 8 + 1) * 8));

                        let dst_reg = self.resolve_operand(ir_block_inst, true, map_info);
                        let offset = pos;
                        self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::OpReg(SingleOp::LoadAddr), 
                            vec![dst_reg.clone(), Operand::Addr(label.clone())])));
                        
                        let mut store = LIRInst::new(InstrsType::StoreParamToStack, 
                            vec![dst_reg.clone(), Operand::IImm(IImm::new(offset))]);
                        store.set_double();
                        self.insts.push(self.insts_mpool.put(store));
                        
                        // array: offset~offset+size(8字节对齐)
                        // map_key: array_name
                        map_info.int_array_map.insert(alloca);
                        map_info.array_slot_map.insert(ir_block_inst, offset);
                    }
                }
                InstKind::Gep => {
                    //TODO: 数组的优化使用
                    // type(4B) * index 
                    // 数组成员若是int型则不超过32位，使用word
                    // ld dst array_offset(sp)
                    // lw dst 4 * gep_offset(dst)
                    let offset = inst_ref.get_gep_offset().as_ref().get_int_bond() * 4;
                    let dst_reg = self.resolve_operand(ir_block_inst, true, map_info);
                    let index = inst_ref.get_ptr();
                    if let Some(head) = map_info.array_slot_map.get(&index){
                        //TODO:判断地址合法
                        let mut load = LIRInst::new(InstrsType::LoadParamFromStack, 
                            vec![dst_reg.clone(), Operand::IImm(IImm::new(*head))]);
                        load.set_double();
                        self.insts.push(self.insts_mpool.put(load));
                        self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Load, 
                            vec![dst_reg.clone(), dst_reg.clone(), Operand::IImm(IImm::new(offset))])));
                    } else {
                     panic!("array not found");
                    }
                },
                InstKind::Branch => {
                    // if jump
                    if inst_ref.is_jmp() {
                        let next_bb = block.as_ref().get_next_bb()[0];
                        let jump_block = match map_info.ir_block_map.get(&next_bb) {
                            Some(block) => block,
                            None => panic!("jump block not found"),
                        };
                        if *jump_block != next_blocks.unwrap() {
                            self.insts.push(self.insts_mpool.put(
                                LIRInst::new(InstrsType::Jump, 
                                    vec![Operand::Addr(next_bb.as_ref().get_name().to_string())])
                            ));
                        }
                        jump_block.as_mut().in_edge.push(ObjPtr::new(self));
                        self.out_edge.push(*jump_block);
                        break;
                    }

                    // if branch
                    let cond_ref = inst_ref.get_br_cond().as_ref();

                    let true_bb = block.as_ref().get_next_bb()[0];
                    let false_bb = block.as_ref().get_next_bb()[1];
                    let block_map = map_info.ir_block_map.clone();
                    let true_block = match block_map.get(&true_bb) {
                        Some(block) => block,
                        None => unreachable!("true block not found"),
                    };
                    let false_block = match block_map.get(&false_bb) {
                        Some(block) => block,
                        None => unreachable!("false block not found"),
                    };
                    
                    match cond_ref.get_kind() {
                        InstKind::Binary(cond) => {
                            let lhs_reg = self.resolve_operand(cond_ref.get_lhs(), true, map_info);
                            let rhs_reg = self.resolve_operand(cond_ref.get_rhs(), true, map_info);
                            let inst_kind = match cond {
                                BinOp::Eq => InstrsType::Branch(CmpOp::Eq),
                                BinOp::Ne => InstrsType::Branch(CmpOp::Ne),
                                BinOp::Ge => InstrsType::Branch(CmpOp::Ge),
                                BinOp::Le => InstrsType::Branch(CmpOp::Le),
                                BinOp::Gt => InstrsType::Branch(CmpOp::Gt),
                                BinOp::Lt => InstrsType::Branch(CmpOp::Lt),
                                _ => { unreachable!("no condition match") }
                            };
                            self.insts.push(self.insts_mpool.put(
                                LIRInst::new(inst_kind, 
                                    vec![Operand::Addr(false_bb.as_ref().get_name().to_string()), lhs_reg, rhs_reg])
                            ));
                            self.insts.push(self.insts_mpool.put(
                                LIRInst::new(InstrsType::Jump, 
                                    vec![Operand::Addr(false_bb.as_ref().get_name().to_string())])
                            ));
                            true_block.as_mut().in_edge.push(ObjPtr::new(self));
                            false_block.as_mut().in_edge.push(ObjPtr::new(self));
                            self.out_edge.append(vec![*true_block, *false_block].as_mut());
                        }
                        _ => { unreachable!("cond is not binary condition judgement, to improve") }
                    }
                },
                InstKind::Call(func_label) => {
                    let arg_list = inst_ref.get_args();
                    let mut icnt = 0;
                    let mut fcnt = 0;
                    for arg in arg_list {
                        assert!(arg.as_ref().get_ir_type() == IrType::Parameter);
                        if arg.as_ref().get_param_type() == IrType::Int {
                            icnt += 1
                        } else if arg.as_ref().get_param_type() == IrType::Float {
                            fcnt += 1
                        } else {
                            unreachable!("call arg type not match, either be int or float")
                        }
                    }

                    let mut lir_inst = LIRInst::new(InstrsType::Call,
                        vec![Operand::Addr(func_label.to_string())]);
                    lir_inst.set_param_cnts(icnt, fcnt);
                    self.insts.push(self.insts_mpool.put(lir_inst));

                    // set stack slot
                    let mut pos = 0;
                    let mut size = 0;
                    if let Some(last_slot) = map_info.stack_slot_set.back() {
                        pos = last_slot.get_pos() + last_slot.get_size();
                        size = max(0, icnt - ARG_REG_COUNT) + max(0, fcnt - ARG_REG_COUNT);
                        //FIXME: 是否需要对齐
                        if size % 2 == 1 {
                            size += 1;
                        }
                        size *= 4;
                        map_info.stack_slot_set.push_back(StackSlot::new(pos, size));
                    } else {
                        unreachable!("stack slot set is empty");
                    }
                    
                    for arg in arg_list.iter().rev() {
                        match arg.as_ref().get_param_type() {
                            IrType::Int => {
                                icnt -= 1;
                                if icnt >= ARG_REG_COUNT {
                                    let src_reg = self.resolve_operand(*arg, true, map_info);
                                    // 这里用于存到栈上的参数是从后往前的
                                    let offset = Operand::IImm(IImm::new(pos + size - icnt * 4));
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::StoreParamToStack,
                                            vec![src_reg, offset])
                                    ));
                                } else {
                                    // 保存在寄存器中的参数，从前往后
                                    // FIXME:保存寄存器需要额外的开栈开销
                                    let src_reg = self.resolve_operand(*arg, true, map_info);
                                    let dst_reg = Operand::Reg(Reg::new(icnt, ScalarType::Int));
                                    let pos = map_info.stack_slot_set.back().unwrap().get_pos() + map_info.stack_slot_set.back().unwrap().get_size();
                                    let size = 8;
                                    map_info.stack_slot_set.push_back(StackSlot::new(pos, size));
                                    let mut inst = LIRInst::new(InstrsType::StoreToStack,
                                        vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))]);
                                    inst.set_double();
                                    self.insts.push(self.insts_mpool.put(inst));
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::IMv), 
                                            vec![dst_reg, src_reg])
                                    ));
                                }
                            },
                            IrType::Float => {
                                fcnt -= 1;
                                if fcnt >= ARG_REG_COUNT {
                                    let src_reg = self.resolve_operand(*arg, true, map_info);
                                    // 这里用于存到栈上的参数是从后往前的
                                    let offset = Operand::IImm(IImm::new(pos + size - fcnt * 4));
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::StoreParamToStack,
                                            vec![src_reg, offset])
                                    ));
                                } else {
                                    let src_reg = self.resolve_operand(*arg, true, map_info);
                                    let dst_reg = Operand::Reg(Reg::new(fcnt, ScalarType::Int));
                                    let pos = map_info.stack_slot_set.back().unwrap().get_pos() + map_info.stack_slot_set.back().unwrap().get_size();
                                    let size = 8;
                                    map_info.stack_slot_set.push_back(StackSlot::new(pos, size));
                                    let mut inst = LIRInst::new(InstrsType::StoreToStack,
                                        vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))]);
                                    inst.set_double();
                                    self.insts.push(self.insts_mpool.put(inst));
                                    self.insts.push(self.insts_mpool.put(
                                        LIRInst::new(InstrsType::OpReg(SingleOp::FMv), 
                                            vec![dst_reg, src_reg])
                                    ));
                                }
                            },
                            _ => unreachable!("call arg type not match, either be int or float")
                        }
                    }
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
                            let src_operand = self.resolve_operand(src, false, map_info);
                            self.insts.push(
                                self.insts_mpool.put(
                                    LIRInst::new(InstrsType::OpReg(SingleOp::Li), 
                                        vec![Operand::Reg(Reg::new(10, ScalarType::Int)) ,src_operand])
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
                },
                InstKind::ConstFloat(..) | InstKind::ConstInt(..) | InstKind::GlobalConstFloat(..) | InstKind::GlobalConstInt(..) | InstKind::GlobalFloat(..) |
                InstKind::GlobalInt(..) | InstKind::Head | InstKind::Parameter | InstKind::Phi => {
                    // do nothing
                },
                // _ => {
                // // TODO: ir translation.
                //     unreachable!("cannot reach, ir translation false")
                // }
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

    fn resolve_operand(&mut self, src: ObjPtr<Inst>, is_left: bool, map: &mut Mapping) -> Operand {
        //TODO: ObjPtr match
        if is_left {
            match src.as_ref().get_kind() {
                InstKind::ConstInt(iimm) => return self.load_iimm_to_ireg(iimm),
                _ => {}
            }
        }

        match src.as_ref().get_kind() {
            InstKind::ConstInt(iimm) => self.resolve_iimm(iimm),
            InstKind::ConstFloat(fimm) => self.resolve_fimm(fimm),
            InstKind::Parameter => self.resolve_param(src, map),
            // InstKind::GlobalConstInt(_) => resolveGlobalVar(src),
            // InstKind::GlobalConstFloat(_) => resolveGlobalVar(src),
            _ => {
                panic!("more operand-ir_inst to resolve");
            }
        }
    }

    fn resolve_iimm(&mut self, imm: i32) -> Operand {
        //TODO: if type > i32
        let res = IImm::new(imm);
        if operand::is_imm_12bs(imm) {
            Operand::IImm(res)
        } else {
            self.load_iimm_to_ireg(imm)
        }
    }

    //FIXME: fimm 使用16进制表示转换为int，使用浮点数加法
    fn resolve_fimm(&mut self, imm: f32) -> Operand {
        let fimm = Operand::FImm(FImm::new(imm));
        let reg = Operand::Reg(Reg::init(ScalarType::Float));
        //FIXME:
        if operand::is_imm_12bs(imm as i32) {
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::OpReg(SingleOp::Li), vec![reg.clone(), fimm.clone()])));
        } else {
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::OpReg(SingleOp::Lui), vec![reg.clone(), fimm.clone()])));
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Binary(BinaryOp::Add), vec![reg.clone(), reg.clone(), fimm.clone()])));
        }
        reg
    }

    fn load_iimm_to_ireg(&mut self, imm: i32) -> Operand {
        let reg = Operand::Reg(Reg::init(ScalarType::Int));
        let iimm = Operand::IImm(IImm::new(imm));
        if operand::is_imm_12bs(imm) {
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::OpReg(SingleOp::Li), vec![reg.clone(), iimm.clone()])));
        } else {
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::OpReg(SingleOp::Lui), vec![reg.clone(), iimm.clone()])));
            self.insts.push(self.insts_mpool.put(LIRInst::new(InstrsType::Binary(BinaryOp::Add), vec![reg.clone(), reg.clone(), iimm.clone()])));
        }
        reg
    }

    fn resolve_param(&self, src: ObjPtr<Inst>, map: &mut Mapping) -> Operand {
        if !map.val_map.contains_key(&src) {
            let reg = match src.as_ref().get_param_type() {
                IrType::Int => Operand::Reg(Reg::init(ScalarType::Int)),
                IrType::Float => Operand::Reg(Reg::init(ScalarType::Float)),
                _ => unreachable!("cannot reach, param either int or float")
            };
            map.val_map.insert(src, reg.clone());
            reg
        } else {
            map.val_map.get(&src).unwrap().clone()
        }
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
    fn generate(&mut self, context: ObjPtr<Context>, f: FILE_PATH) -> Result<()> {
        if self.called {
            print!("{}:\n", self.label);
        }
        println!("generate bb\n");
        for inst in self.insts.iter() {
            print!("inst here\n");
            inst.as_mut().generate(context.clone(), f.clone())?;
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

fn is_opt_num(imm: i32) -> bool {
    (imm & (imm - 1)) == 0
}