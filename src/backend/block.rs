use std::cmp::max;
pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;

use crate::backend::func::Func;
use crate::backend::instrs::Operand;
use crate::backend::instrs::{BinaryOp, CmpOp, InstrsType, LIRInst, SingleOp};
use crate::backend::operand::{IImm, Reg, F_REG_ID, I_REG_ID};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{BinOp, Inst, InstKind, UnOp};
use crate::ir::ir_type::IrType;
use crate::utility::{ObjPtr, ScalarType};

use super::operand::ARG_REG_COUNT;
use super::{structs::*, BackendPool};
use crate::backend::operand;

pub static mut ARRAY_NUM: i32 = 0;
// pub static mut GLOBAL_SEQ: i32 = 0;
pub static mut TMP_BB: i32 = 0;

pub const ADDR_SIZE: i32 = 8;
pub const NUM_SIZE: i32 = 4;

#[derive(Clone)]
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

    global_map: HashMap<ObjPtr<Inst>, Operand>,
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
            global_map: HashMap::new(),
        }
    }

    /// 寄存器分配时决定开栈大小、栈对象属性(size(4/8 bytes), pos)，回填func的stack_addr
    /// 尽量保证程序顺序执行，并满足首次遇分支向后跳转的原则？
    //FIXME: b型指令长跳转(目标地址偏移量为+-4KiB)，若立即数非法是否需要增添一个jal块实现间接跳转？
    pub fn construct(
        &mut self,
        func: ObjPtr<Func>,
        block: ObjPtr<BasicBlock>,
        next_blocks: Option<ObjPtr<BB>>,
        map_info: &mut Mapping,
        pool: &mut BackendPool,
    ) {
        let mut ir_block_inst = block.as_ref().get_head_inst();
        loop {
            let inst_ref = ir_block_inst.as_ref();
            println!("inst_ref: {:?}", inst_ref.get_kind());
            // translate ir to lir, use match
            match inst_ref.get_kind() {
                InstKind::Binary(op) => {
                    let lhs = inst_ref.get_lhs();
                    let rhs = inst_ref.get_rhs();
                    let mut lhs_reg: Operand = Operand::IImm(IImm::new(0));
                    let mut rhs_reg: Operand = Operand::IImm(IImm::new(0));
                    let mut dst_reg: Operand =
                        self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    match op {
                        //TODO: Float Binary
                        BinOp::Add => {
                            let inst_kind = InstrsType::Binary(BinaryOp::Add);
                            match lhs.as_ref().get_kind() {
                                //立即数
                                InstKind::ConstInt(..) => {
                                    lhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                                    rhs_reg =
                                        self.resolve_operand(func, lhs, false, map_info, pool);
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        inst_kind,
                                        vec![dst_reg, lhs_reg, rhs_reg],
                                    )));
                                }
                                _ => {
                                    //不是立即数
                                    assert!(
                                        lhs.as_ref().get_ir_type() == IrType::Int
                                            && rhs.as_ref().get_ir_type() == IrType::Int
                                    );
                                    lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                                    rhs_reg =
                                        self.resolve_operand(func, rhs, false, map_info, pool);
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        inst_kind,
                                        vec![dst_reg, lhs_reg, rhs_reg],
                                    )));
                                }
                            }
                        }
                        BinOp::Sub => {
                            let mut inst_kind = InstrsType::Binary(BinaryOp::Sub);
                            lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);

                            match rhs.as_ref().get_kind() {
                                InstKind::ConstInt(imm) => {
                                    inst_kind = InstrsType::Binary(BinaryOp::Add);
                                    rhs_reg = self.resolve_iimm(-imm, pool);
                                }
                                _ => {
                                    //不是立即数
                                    assert!(
                                        lhs.as_ref().get_ir_type() == IrType::Int
                                            && rhs.as_ref().get_ir_type() == IrType::Int
                                    );
                                    rhs_reg =
                                        self.resolve_operand(func, rhs, false, map_info, pool);
                                }
                            }
                            println!("lhs_reg: {:?}", lhs_reg);
                            println!("rhs_reg: {:?}", rhs_reg);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![dst_reg, lhs_reg, rhs_reg],
                            )));
                        }
                        BinOp::Mul => {
                            let mut op_flag = false;
                            let mut src: Operand = Operand::IImm(IImm::new(0));
                            let mut imm = 0;
                            let limm = match lhs.as_ref().get_kind() {
                                InstKind::ConstInt(..) => true,
                                _ => false,
                            };
                            let rimm = match rhs.as_ref().get_kind() {
                                InstKind::ConstInt(..) => true,
                                _ => false,
                            };
                            if limm && rimm {
                                if limm && is_opt_mul(lhs.as_ref().get_int_bond()) {
                                    op_flag = true;
                                    src = self.resolve_operand(func, rhs, true, map_info, pool);
                                    imm = lhs.as_ref().get_int_bond();
                                }
                                if rimm && is_opt_mul(rhs.as_ref().get_int_bond()) {
                                    op_flag = true;
                                    src = self.resolve_operand(func, lhs, true, map_info, pool);
                                    imm = rhs.as_ref().get_int_bond();
                                }
                                //FIXME: 暂时不使用乘法优化
                                // if op_flag {
                                //     self.resolve_opt_mul(dst_reg, src, imm);
                                //     break;
                                // }
                            }
                            lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                            rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Binary(BinaryOp::Mul),
                                vec![dst_reg, lhs_reg, rhs_reg],
                            )));
                        }
                        BinOp::Div => {
                            lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                            assert!(rhs.as_ref().get_ir_type() == IrType::Int);
                            // match rhs.as_ref().get_kind() {
                            // InstKind::ConstInt(imm) => {
                            // self.resolve_opt_div(dst_reg, lhs_reg, imm, pool)
                            // }
                            // _ => {
                            rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Binary(BinaryOp::Div),
                                vec![dst_reg, lhs_reg, rhs_reg],
                            )));
                            // }
                            // }
                        }
                        BinOp::Rem => {
                            // x % y == x - (x / y) *y
                            // % 0 % 1 % 2^n 特殊判断
                            match rhs.as_ref().get_kind() {
                                InstKind::ConstInt(imm) => {
                                    match imm {
                                        0 => {
                                            lhs_reg = self
                                                .resolve_operand(func, lhs, true, map_info, pool);
                                            self.insts.push(pool.put_inst(LIRInst::new(
                                                InstrsType::OpReg(SingleOp::IMv),
                                                vec![dst_reg, lhs_reg],
                                            )));
                                        }
                                        1 | -1 => {
                                            self.insts.push(pool.put_inst(LIRInst::new(
                                                InstrsType::OpReg(SingleOp::IMv),
                                                vec![dst_reg, Operand::IImm(IImm::new(0))],
                                            )));
                                        }
                                        _ => {
                                            if is_opt_num(imm) || is_opt_num(-imm) {
                                                //TODO: 暂时不使用优化
                                                self.resolve_opt_rem(
                                                    func, map_info, dst_reg, lhs, rhs, pool,
                                                );
                                            } else {
                                                lhs_reg = self.resolve_operand(
                                                    func, lhs, true, map_info, pool,
                                                );
                                                rhs_reg = self.resolve_operand(
                                                    func, rhs, true, map_info, pool,
                                                );
                                                self.insts.push(pool.put_inst(LIRInst::new(
                                                    InstrsType::Binary(BinaryOp::Rem),
                                                    vec![dst_reg, lhs_reg, rhs_reg],
                                                )));
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    assert!(
                                        lhs.as_ref().get_ir_type() == IrType::Int
                                            && rhs.as_ref().get_ir_type() == IrType::Int
                                    );
                                    lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                                    rhs_reg =
                                        self.resolve_operand(func, rhs, false, map_info, pool);
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Rem),
                                        vec![dst_reg, lhs_reg, rhs_reg],
                                    )));
                                }
                            };
                        }
                        BinOp::And => {
                            lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                            rhs_reg = self.resolve_operand(func, rhs, false, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Binary(BinaryOp::And),
                                vec![dst_reg, lhs_reg, rhs_reg],
                            )));
                        }
                        BinOp::Or => {
                            lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                            rhs_reg = self.resolve_operand(func, rhs, false, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Binary(BinaryOp::Or),
                                vec![dst_reg, lhs_reg, rhs_reg],
                            )));
                        }
                        _ => {
                            unreachable!("more binary op to resolve");
                        }
                    }
                }
                InstKind::Unary(op) => {
                    let dst_reg = self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    let src = ir_block_inst.as_ref().get_unary_operand();
                    let src_reg = self.resolve_operand(func, src, true, map_info, pool);
                    match op {
                        UnOp::Neg => match src.as_ref().get_kind() {
                            InstKind::ConstInt(imm) => {
                                let iimm = self.resolve_iimm(-imm, pool);
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Li),
                                    vec![dst_reg, iimm],
                                )))
                            }
                            InstKind::ConstFloat(fimm) => {
                                todo!("neg float");
                            }
                            _ => match src.as_ref().get_ir_type() {
                                IrType::Int => self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::INeg),
                                    vec![dst_reg, src_reg],
                                ))),
                                IrType::Float => self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::FNeg),
                                    vec![dst_reg, src_reg],
                                ))),
                                _ => {
                                    unreachable!("invalid unary type for neg");
                                }
                            },
                        },
                        UnOp::Not => match src.as_ref().get_kind() {
                            InstKind::ConstInt(imm) => {
                                let imm = src.as_ref().get_int_bond();
                                let iimm = self.resolve_iimm(!imm, pool);
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Li),
                                    vec![dst_reg, iimm],
                                )));
                            }
                            InstKind::ConstFloat(..) => {
                                todo!("not float");
                            }
                            _ => match src.as_ref().get_ir_type() {
                                IrType::Int => {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::INot),
                                        vec![dst_reg, src_reg],
                                    )));
                                }
                                _ => {
                                    panic!("invalid unary type for not");
                                }
                            },
                        },
                        UnOp::Pos => match src.as_ref().get_kind() {
                            InstKind::ConstInt(imm) => {
                                let imm = src.as_ref().get_int_bond();
                                let iimm = self.resolve_iimm(imm, pool);
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Li),
                                    vec![dst_reg, iimm],
                                )));
                            }
                            InstKind::ConstFloat(..) => {
                                todo!("pos float");
                            }
                            _ => match src.as_ref().get_ir_type() {
                                IrType::Int => {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::IMv),
                                        vec![dst_reg, src_reg],
                                    )));
                                }
                                IrType::Float => {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::FMv),
                                        vec![dst_reg, src_reg],
                                    )));
                                }
                                _ => {
                                    panic!("invalid unary type for pos");
                                }
                            },
                        },
                    }
                }

                //TODO: load/store float
                InstKind::Load => {
                    let addr = inst_ref.get_ptr();
                    //TODO: if global var
                    let dst_reg = self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    match addr.as_ref().get_kind() {
                        InstKind::GlobalConstFloat(..)
                        | InstKind::GlobalFloat(..)
                        | InstKind::GlobalInt(..)
                        | InstKind::GlobalConstInt(..) => {
                            assert!(map_info.val_map.contains_key(&addr));
                            let src_reg = self.resolve_global(addr, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Load,
                                vec![dst_reg, src_reg, Operand::IImm(IImm::new(0))],
                            )));
                        }
                        InstKind::Gep => {
                            //TODO: 数组的优化使用
                            //TODO: lable从栈上重复加载问题
                            // type(4B) * index
                            // 数组成员若是int型则不超过32位，使用word
                            // ld dst array_offset(sp) # get label(base addr)
                            // lw dst 4 * gep_offset(dst)
                            println!("occur");
                            let offset = addr.as_ref().get_gep_offset().as_ref().get_int_bond() * 4;
                            println!("gep offset: {}", offset);
                            let dst_reg =
                                self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                            let index = addr.as_ref().get_gep_ptr();
                            if let Some(head) = map_info.array_slot_map.get(&index) {
                                //TODO:判断地址合法
                                let mut load = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![dst_reg.clone(), Operand::IImm(IImm::new(*head))],
                                );
                                load.set_double();
                                self.insts.push(pool.put_inst(load));
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::Load,
                                    vec![
                                        dst_reg.clone(),
                                        dst_reg.clone(),
                                        Operand::IImm(IImm::new(offset)),
                                    ],
                                )));
                            } else {
                                // 找不到，认为是全局数组，全局数组的访问是load -> gep -> load -> alloca
                                let src_reg = self.resolve_operand(func, index, true, map_info, pool);
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::Load,
                                    vec![dst_reg, src_reg, Operand::IImm(IImm::new(offset))],
                                )));
                            }
                        }
                        InstKind::Alloca(..) => {
                            assert!(
                                addr.as_ref().get_ir_type() == IrType::IntPtr
                                    || addr.as_ref().get_ir_type() == IrType::FloatPtr
                            );
                            if let Some(ga) = map_info.val_map.get(&addr) {
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::LoadAddr),
                                    vec![dst_reg.clone(), ga.clone()],
                                )));
                            } else {
                                unreachable!("invalid gep");
                            }
                        }
                        _ => {
                            unreachable!("invalid load addr");
                            // self.resolve_operand(func, addr, false, map_info, pool);
                        }
                    };
                }
                InstKind::Store => {
                    let addr = inst_ref.get_dest();
                    let value = inst_ref.get_value();
                    println!("{:?}", addr.as_ref().get_kind());
                    println!("{:?}", value.as_ref().get_kind());
                    let value_reg = self.resolve_operand(func, value, true, map_info, pool);
                    match addr.as_ref().get_kind() {
                        InstKind::Gep => {
                            let mut load_new = true;
                            let addr_reg = match map_info.val_map.get(&addr.as_ref().get_gep_ptr())
                            {
                                Some(reg) => {
                                    load_new = false;
                                    reg.clone()
                                }
                                None => Operand::Reg(Reg::init(ScalarType::Int)),
                            };
                            if let Some(base) =
                                map_info.array_slot_map.get(&addr.as_ref().get_gep_ptr())
                            {
                                let offset =
                                    addr.as_ref().get_gep_offset().as_ref().get_int_bond() * 4;
                                if load_new {
                                    let mut load = LIRInst::new(
                                        InstrsType::LoadParamFromStack,
                                        vec![addr_reg.clone(), Operand::IImm(IImm::new(*base))],
                                    );
                                    load.set_double();
                                    self.insts.push(pool.put_inst(load));
                                }
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::Store,
                                    vec![addr_reg, value_reg, Operand::IImm(IImm::new(offset))],
                                )));
                            } else {
                                panic!("array not found");
                            }
                        }
                        _ => {
                            let addr_reg = self.resolve_operand(func, addr, false, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Store,
                                vec![addr_reg, value_reg, Operand::IImm(IImm::new(0))],
                            )));
                        }
                    }
                }
                InstKind::Alloca(size) => {
                    let array_num = get_current_array_num();
                    let label = format!(".LC{array_num}");
                    inc_array_num();
                    //将发生分配的数组装入map_info中：记录数组结构、占用栈空间
                    //TODO:la dst label    sd dst (offset)sp
                    //TODO: 大数组而装填因子过低的压缩问题
                    //FIXME: 认为未初始化数组也被初始化为全0
                    let alloca =
                        IntArray::new(label.clone(), size, true, inst_ref.get_int_init().clone());
                    let last = func.as_ref().stack_addr.front().unwrap();
                    let pos = last.get_pos() + ADDR_SIZE;
                    func.as_mut()
                        .stack_addr
                        .push_front(StackSlot::new(pos, ADDR_SIZE));

                    let dst_reg = self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    let offset = pos;
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::LoadAddr),
                        vec![dst_reg.clone(), Operand::Addr(label.clone())],
                    )));

                    let mut store = LIRInst::new(
                        InstrsType::StoreParamToStack,
                        vec![dst_reg.clone(), Operand::IImm(IImm::new(offset))],
                    );
                    store.set_double();
                    self.insts.push(pool.put_inst(store));

                    // array: offset~offset+size(8字节对齐)
                    // map_key: array_name
                    func.as_mut().const_array.insert(alloca);
                    map_info.array_slot_map.insert(ir_block_inst, offset);
                }
                InstKind::Branch => {
                    // if jump
                    let mut inst = LIRInst::new(InstrsType::Jump, vec![]);
                    if inst_ref.is_jmp() {
                        let next_bb = block.as_ref().get_next_bb()[0];
                        let jump_block = match map_info.ir_block_map.get(&next_bb) {
                            Some(block) => block,
                            None => panic!("jump block not found"),
                        };
                        if *jump_block != next_blocks.unwrap() {
                            inst.replace_op(vec![Operand::Addr(
                                next_bb.as_ref().get_name().to_string(),
                            )]);
                            let obj_inst = pool.put_inst(inst);
                            self.insts.push(obj_inst);
                            map_info
                                .block_branch
                                .insert(pool.put_block(self.clone()), obj_inst);
                        }
                        let this = self.clone();
                        jump_block.as_mut().in_edge.push(pool.put_block(this));
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
                            let lhs_reg = self.resolve_operand(
                                func,
                                cond_ref.get_lhs(),
                                true,
                                map_info,
                                pool,
                            );
                            let rhs_reg = self.resolve_operand(
                                func,
                                cond_ref.get_rhs(),
                                true,
                                map_info,
                                pool,
                            );
                            let inst_kind = match cond {
                                BinOp::Eq => InstrsType::Branch(CmpOp::Eq),
                                BinOp::Ne => InstrsType::Branch(CmpOp::Ne),
                                BinOp::Ge => InstrsType::Branch(CmpOp::Ge),
                                BinOp::Le => InstrsType::Branch(CmpOp::Le),
                                BinOp::Gt => InstrsType::Branch(CmpOp::Gt),
                                BinOp::Lt => InstrsType::Branch(CmpOp::Lt),
                                _ => {
                                    unreachable!("no condition match")
                                }
                            };
                            self.insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![
                                    Operand::Addr(false_bb.as_ref().get_name().to_string()),
                                    lhs_reg,
                                    rhs_reg,
                                ],
                            )));
                            self.push_back(pool.put_inst(LIRInst::new(
                                InstrsType::Jump,
                                vec![Operand::Addr(true_bb.as_ref().get_name().to_string())],
                            )));

                            inst.replace_op(vec![Operand::Addr(
                                false_bb.as_ref().get_name().to_string(),
                            )]);
                            let obj_inst = pool.put_inst(inst);
                            map_info
                                .block_branch
                                .insert(pool.put_block(self.clone()), obj_inst);
                            let this = self.clone();
                            true_block
                                .as_mut()
                                .in_edge
                                .push(pool.put_block(this.clone()));
                            false_block.as_mut().in_edge.push(pool.put_block(this));
                            self.out_edge
                                .append(vec![*true_block, *false_block].as_mut());
                        }
                        _ => {
                            unreachable!("cond is not binary condition judgement, to improve")
                        }
                    }
                }
                InstKind::Call(func_label) => {
                    let arg_list = inst_ref.get_args();
                    let mut icnt = 0;
                    let mut fcnt = 0;
                    for arg in arg_list {
                        if arg.as_ref().get_param_type() == IrType::Int {
                            icnt += 1
                        } else if arg.as_ref().get_param_type() == IrType::Float {
                            fcnt += 1
                        } else {
                            unreachable!("call arg type not match, either be int or float")
                        }
                    }

                    for arg in arg_list.iter().rev() {
                        match arg.as_ref().get_param_type() {
                            IrType::Int | IrType::IntPtr => {
                                icnt -= 1;
                                if icnt >= ARG_REG_COUNT {
                                    let src_reg =
                                        self.resolve_operand(func, *arg, true, map_info, pool);
                                    // 最后一个溢出参数在最下方（最远离sp位置）
                                    let offset = Operand::IImm(IImm::new(
                                        -(max(0, icnt - ARG_REG_COUNT)
                                            + max(0, fcnt - ARG_REG_COUNT))
                                            * 4,
                                    ));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::StoreToStack,
                                        vec![src_reg, offset],
                                    )));
                                } else {
                                    // 保存在寄存器中的参数，从前往后
                                    let src_reg =
                                        self.resolve_operand(func, *arg, true, map_info, pool);
                                    let dst_reg =
                                        Operand::Reg(Reg::new(icnt + 10, ScalarType::Int));
                                    let stack_addr = &func.as_ref().stack_addr;
                                    let pos = stack_addr.front().unwrap().get_pos()
                                        + ADDR_SIZE;
                                    let size = 8;
                                    let slot = StackSlot::new(pos, size);
                                    func.as_mut().stack_addr.push_front(slot);
                                    func.as_mut().spill_stack_map.insert(icnt, slot);
                                    let mut inst = LIRInst::new(
                                        InstrsType::StoreParamToStack,
                                        vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))],
                                    );
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::IMv),
                                        vec![dst_reg, src_reg],
                                    )));
                                }
                            }
                            IrType::Float | IrType::FloatPtr => {
                                fcnt -= 1;
                                if fcnt >= ARG_REG_COUNT {
                                    let src_reg =
                                        self.resolve_operand(func, *arg, true, map_info, pool);
                                    // 第后一个溢出参数在最下方（最远离sp位置）
                                    let offset = Operand::IImm(IImm::new(
                                        -(max(0, icnt - ARG_REG_COUNT)
                                            + max(0, fcnt - ARG_REG_COUNT))
                                            * 4,
                                    ));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::StoreToStack,
                                        vec![src_reg, offset],
                                    )));
                                } else {
                                    //FIXME:暂时不考虑浮点数参数
                                    let src_reg =
                                        self.resolve_operand(func, *arg, true, map_info, pool);
                                    let dst_reg =
                                        Operand::Reg(Reg::new(fcnt + 10, ScalarType::Float));
                                    let stack_addr = &func.as_ref().stack_addr;
                                    let pos = stack_addr.back().unwrap().get_pos()
                                        + ADDR_SIZE;
                                    let size = 8;
                                    func.as_mut()
                                        .stack_addr
                                        .push_back(StackSlot::new(pos, size));
                                    let mut inst = LIRInst::new(
                                        InstrsType::StoreParamToStack,
                                        vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))],
                                    );
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::FMv),
                                        vec![dst_reg, src_reg],
                                    )));
                                }
                            }
                            _ => unreachable!("call arg type not match, either be int or float"),
                        }
                    }
                    let mut lir_inst = LIRInst::new(
                        InstrsType::Call,
                        vec![Operand::Addr(func_label.to_string())],
                    );
                    lir_inst.set_param_cnts(icnt, fcnt);
                    self.insts.push(pool.put_inst(lir_inst));

                    match inst_ref.get_ir_type() {
                        IrType::Int => {
                            let dst_reg =
                                self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::OpReg(SingleOp::IMv),
                                vec![dst_reg, Operand::Reg(Reg::new(10, ScalarType::Int))],
                            )));
                        }
                        IrType::Float => {
                            let dst_reg =
                                self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::OpReg(SingleOp::FMv),
                                vec![dst_reg, Operand::Reg(Reg::new(10, ScalarType::Float))],
                            )));
                        }
                        IrType::Void => {}
                        _ => unreachable!("call return type not match, must be int, float or void"),
                    }

                    // restore stack slot
                    let mut i = 0;
                    while i < ARG_REG_COUNT {
                        if let Some(slot) = func.as_ref().spill_stack_map.get(&i) {
                            let mut inst = LIRInst::new(
                                InstrsType::LoadFromStack,
                                vec![
                                    Operand::Reg(Reg::new(i + 10, ScalarType::Int)),
                                    Operand::IImm(IImm::new(slot.get_pos())),
                                ],
                            );
                            inst.set_double();
                            self.insts.push(pool.put_inst(inst));
                        }
                        i += 1;
                    }
                }
                InstKind::Return => match inst_ref.get_ir_type() {
                    IrType::Void => self.insts.push(
                        pool.put_inst(LIRInst::new(InstrsType::Ret(ScalarType::Void), vec![])),
                    ),
                    IrType::Int => {
                        let src = inst_ref.get_return_value();
                        let src_operand = self.resolve_operand(func, src, true, map_info, pool);
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::IMv),
                            vec![Operand::Reg(Reg::new(10, ScalarType::Int)), src_operand],
                        )));

                        self.insts.push(
                            pool.put_inst(LIRInst::new(InstrsType::Ret(ScalarType::Int), vec![])),
                        );
                    }
                    IrType::Float => {
                        todo!("return float");
                    }
                    _ => panic!("cannot reach, Return false"),
                },
                InstKind::ItoF => {
                    todo!("ItoF")
                }
                InstKind::FtoI => {
                    todo!("FtoI")
                }

                InstKind::Phi => {
                    let phi_reg = self.resolve_operand(func, ir_block_inst, false, map_info, pool);
                    let mut kind = ScalarType::Void;
                    let temp = match phi_reg {
                        Operand::Reg(reg) => {
                            assert!(reg.get_type() != ScalarType::Void);
                            kind = reg.get_type();
                            Operand::Reg(Reg::init(reg.get_type()))
                        }
                        _ => unreachable!("phi reg must be reg"),
                    };
                    assert!(kind != ScalarType::Void);
                    let inst_kind = match kind {
                        ScalarType::Int => InstrsType::OpReg(SingleOp::IMv),
                        ScalarType::Float => InstrsType::OpReg(SingleOp::FMv),
                        _ => unreachable!("mv must be int or float"),
                    };
                    self.insts.insert(
                        0,
                        pool.put_inst(LIRInst::new(inst_kind, vec![temp.clone(), phi_reg])),
                    );
                    inst_ref.get_operands().iter().for_each(|op| {
                        let src_reg = self.resolve_operand(func, *op, true, map_info, pool);
                        let inst = LIRInst::new(inst_kind, vec![temp.clone(), src_reg]);
                        let obj_inst = pool.put_inst(inst);
                        let incoming_block = map_info
                            .ir_block_map
                            .get(&op.as_ref().get_parent_bb())
                            .unwrap();
                        if map_info.block_branch.contains_key(incoming_block) {
                            let b_inst = map_info.block_branch.get(incoming_block).unwrap();
                            for i in 0..self.insts.len() {
                                if self.insts[i] == *b_inst {
                                    self.insts.insert(i, obj_inst);
                                    break;
                                }
                            }
                        } else {
                            self.push_back(obj_inst);
                        }
                    });
                }
                _ => {
                    // do nothing
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

    pub fn handle_spill(
        &mut self,
        func: ObjPtr<Func>,
        spill: &HashSet<i32>,
        pos: i32,
        pool: &mut BackendPool,
    ) {
        let mut index = 0;
        loop {
            if index >= self.insts.len() {
                break;
            }
            let inst = self.insts[index].as_mut();
            let id = inst.is_spill(spill);
            let mut offset = 0;
            let mut size = 0;
            if id != -1 {
                //FIXME:暂时使用double进行栈操作，且未处理浮点数
                inst.replace(id, 5);

                let mut store = LIRInst::new(
                    InstrsType::StoreToStack,
                    vec![
                        Operand::Reg(Reg::new(5, ScalarType::Int)),
                        Operand::IImm(IImm::new(pos)),
                    ],
                );
                store.set_double();
                self.insts.insert(index, pool.put_inst(store));
                index += 1;

                //FIXME:直接恢复，故不需存栈信息
                // func.as_mut().stack_addr.push_back(StackSlot::new(pos, 8));

                match func.as_ref().spill_stack_map.get(&id) {
                    Some(slot) => {
                        offset = slot.get_pos();
                        size = slot.get_size();
                        let mut load = LIRInst::new(
                            InstrsType::LoadFromStack,
                            vec![
                                Operand::Reg(Reg::new(5, ScalarType::Int)),
                                Operand::IImm(IImm::new(offset)),
                            ],
                        );
                        if size == 8 {
                            load.set_double();
                        } else if size == 4 {
                        } else {
                            unreachable!("local variable must be 4 or 8 bytes");
                        }
                        self.insts.insert(index, pool.put_inst(load));
                        index += 1;
                    }
                    None => {}
                }

                index += 1;
                store = LIRInst::new(
                    InstrsType::StoreToStack,
                    vec![
                        Operand::Reg(Reg::new(5, ScalarType::Int)),
                        Operand::IImm(IImm::new(pos)),
                    ],
                );
                store.set_double();
                self.insts.insert(index, pool.put_inst(store));
                let slot = StackSlot::new(pos, 8);
                func.as_mut().stack_addr.push_back(slot);
                func.as_mut().spill_stack_map.insert(id, slot);

                index += 1;
                let mut load = LIRInst::new(
                    InstrsType::LoadFromStack,
                    vec![
                        Operand::Reg(Reg::new(5, ScalarType::Int)),
                        Operand::IImm(IImm::new(pos)),
                    ],
                );
                load.set_double();
                self.insts.insert(index, pool.put_inst(load));
            } else {
                index += 1;
            }
        }
    }

    pub fn handle_overflow(&mut self, func: ObjPtr<Func>, pool: &mut BackendPool) {
        let mut pos = 0;
        loop {
            if pos >= self.insts.len() {
                break;
            }
            let inst_ref = self.insts[pos].as_ref();
            match inst_ref.get_type() {
                InstrsType::Load | InstrsType::Store => {
                    let temp = Operand::Reg(Reg::init(ScalarType::Int));
                    let offset = inst_ref.get_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        break;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    self.insts.insert(
                        pos,
                        pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Add),
                            vec![temp.clone(), temp.clone(), inst_ref.get_lhs().clone()],
                        )),
                    );
                    pos += 1;
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }
                InstrsType::LoadFromStack | InstrsType::StoreToStack => {
                    let temp = Operand::Reg(Reg::init(ScalarType::Int));
                    let offset = inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        break;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    self.insts.insert(
                        pos,
                        pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Add),
                            vec![
                                temp.clone(),
                                temp.clone(),
                                Operand::Reg(Reg::new(2, ScalarType::Int)),
                            ],
                        )),
                    );
                    pos += 1;
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }
                InstrsType::LoadParamFromStack | InstrsType::StoreParamToStack => {
                    let temp = Operand::Reg(Reg::init(ScalarType::Int));
                    let offset = func.as_ref().reg_alloc_info.stack_size as i32
                        - inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        break;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    self.insts.insert(
                        pos,
                        pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Add),
                            vec![
                                temp.clone(),
                                temp.clone(),
                                Operand::Reg(Reg::new(2, ScalarType::Int)),
                            ],
                        )),
                    );
                    pos += 1;
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }
                InstrsType::Branch(..) | InstrsType::Jump => {
                    // deal with false branch
                    let mut distance = 0;
                    let is_j = match inst_ref.get_type() {
                        InstrsType::Branch(..) => false,
                        InstrsType::Jump => true,
                        _ => unreachable!(),
                    };
                    let target = match inst_ref.get_label() {
                        Operand::Addr(label) => label,
                        _ => unreachable!("branch must have a label"),
                    };
                    let mut i = 0;
                    let (mut flag, mut first_j) = (false, true);
                    loop {
                        let block_ref = func.as_ref().blocks[i];
                        if &self.label == &block_ref.as_ref().label {
                            flag = true;
                        }
                        if &block_ref.as_ref().label == target {
                            break;
                        }
                        if flag {
                            distance += block_ref.as_ref().insts.len() * 4;
                        }
                        i += 1;
                        if (!is_j && !operand::is_imm_12bs(distance as i32))
                            || (is_j && !operand::is_imm_20bs(distance as i32))
                        {
                            let name = format!("overflow_{}", get_tmp_bb());
                            let tmp = pool.put_block(BB::new(&name));
                            func.as_mut().blocks.insert(i, tmp);
                            if first_j {
                                self.insts[pos].as_mut().replace_label(name);
                                if is_j {
                                    distance -= operand::IMM_20_Bs as usize;
                                } else {
                                    distance -= operand::IMM_12_Bs as usize;
                                }
                            } else {
                                self.insts.insert(
                                    pos,
                                    pool.put_inst(LIRInst::new(
                                        InstrsType::Jump,
                                        vec![Operand::Addr(name)],
                                    )),
                                );
                                distance -= operand::IMM_20_Bs as usize;
                            }
                            pos += 1;
                            first_j = false;
                        }
                    }
                }
                InstrsType::Call => {
                    // call 指令不会发生偏移量的溢出
                    // let mut flag = false;
                    // let mut distance = 0;
                    // let target = match inst_ref.get_label() {
                    //     Operand::Addr(label) => label,
                    //     _ => unreachable!("call must have a label"),
                    // };
                    // for (_, f) in func_map.iter() {
                    //     if &f.as_ref().label == target || f.as_ref().label == func.as_ref().label {
                    //         if !flag {
                    //             flag = true;
                    //         } else {
                    //             break;
                    //         }
                    //     }
                    //     if flag {
                    //         distance += f.as_ref().cal_func_size();
                    //     }
                    //     if !operand::is_imm_20bs(distance as i32) {
                    //         let name = format!("overflow_{}", get_tmp_bb());
                    //         let tmp = pool.put_block(BB::new(&name));
                    //         func.as_mut().blocks.insert(0, tmp);
                    //         self.insts.insert(
                    //             pos,
                    //             pool.put_inst(LIRInst::new(
                    //                 InstrsType::Jump,
                    //                 vec![Operand::Addr(name)],
                    //             )),
                    //         );
                    //         pos += 1;
                    //         distance -= operand::IMM_20_Bs as usize;
                    //     }
                    // }
                }
                _ => {}
            }
            pos += 1;
        }
    }

    fn resolve_overflow_sl(
        &mut self,
        temp: Operand,
        pos: &mut usize,
        offset: i32,
        pool: &mut BackendPool,
    ) {
        let op1 = Operand::IImm(IImm::new(offset >> 12));
        let op2 = Operand::IImm(IImm::new(offset & 0xfff));
        self.insts.insert(
            *pos,
            pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Lui),
                vec![temp.clone(), op1],
            )),
        );
        *pos += 1;
        self.insts.insert(
            *pos,
            pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Add),
                vec![temp.clone(), temp.clone(), op2],
            )),
        );
        *pos += 1;
    }

    fn resolve_operand(
        &mut self,
        func: ObjPtr<Func>,
        src: ObjPtr<Inst>,
        is_left: bool,
        map: &mut Mapping,
        pool: &mut BackendPool,
    ) -> Operand {
        if is_left {
            match src.as_ref().get_kind() {
                InstKind::ConstInt(iimm) => return self.load_iimm_to_ireg(iimm, pool),
                _ => {}
            }
        }

        match src.as_ref().get_kind() {
            InstKind::ConstInt(iimm) => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                self.resolve_iimm(iimm, pool)
            }
            InstKind::ConstFloat(fimm) => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                self.resolve_fimm(fimm, pool, func)
            }
            InstKind::Parameter => self.resolve_param(src, func, map, pool),
            InstKind::GlobalConstInt(_)
            | InstKind::GlobalInt(..)
            | InstKind::GlobalConstFloat(_)
            | InstKind::GlobalFloat(..) => self.resolve_global(src, map, pool),
            _ => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                let op: Operand = match src.as_ref().get_ir_type() {
                    IrType::Int | IrType::IntPtr => Operand::Reg(Reg::init(ScalarType::Int)),
                    IrType::Float | IrType::FloatPtr => Operand::Reg(Reg::init(ScalarType::Float)),
                    _ => unreachable!("cannot reach, resolve_operand func, false, pool"),
                };
                map.val_map.insert(src, op.clone());
                op
            }
        }
    }

    fn resolve_iimm(&mut self, imm: i32, pool: &mut BackendPool) -> Operand {
        let res = IImm::new(imm);
        if operand::is_imm_12bs(imm) {
            Operand::IImm(res)
        } else {
            self.load_iimm_to_ireg(imm, pool)
        }
    }

    fn resolve_fimm(&mut self, imm: f32, pool: &mut BackendPool, func: ObjPtr<Func>) -> Operand {
        let var_name = format!(
            "{label}_float{index}",
            label = func.as_ref().label,
            index = func.as_ref().floats.len()
        );
        func.as_mut().floats.push((var_name.clone(), imm));
        let reg = Operand::Reg(Reg::init(ScalarType::Float));
        let tmp = Operand::Reg(Reg::init(ScalarType::Int));
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::OpReg(SingleOp::LoadAddr),
            vec![tmp.clone(), Operand::Addr(var_name)],
        )));
        let mut inst = LIRInst::new(
            InstrsType::Load,
            vec![reg.clone(), tmp, Operand::IImm(IImm::new(0))],
        );
        inst.set_float();
        self.insts.push(pool.put_inst(inst));
        reg
    }

    fn load_iimm_to_ireg(&mut self, imm: i32, pool: &mut BackendPool) -> Operand {
        let reg = Operand::Reg(Reg::init(ScalarType::Int));
        let iimm = Operand::IImm(IImm::new(imm));
        if operand::is_imm_12bs(imm) {
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Li),
                vec![reg.clone(), iimm],
            )));
        } else {
            let op1 = Operand::IImm(IImm::new(imm >> 12));
            let op2 = Operand::IImm(IImm::new(imm & 0xfff));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Lui),
                vec![reg.clone(), op1],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Add),
                vec![reg.clone(), reg.clone(), op2],
            )));
        }
        reg
    }

    fn resolve_param(
        &mut self,
        src: ObjPtr<Inst>,
        func: ObjPtr<Func>,
        map: &mut Mapping,
        pool: &mut BackendPool,
    ) -> Operand {
        if !map.val_map.contains_key(&src) {
            let params = &func.as_ref().params;
            let reg = match src.as_ref().get_param_type() {
                IrType::Int => Operand::Reg(Reg::init(ScalarType::Int)),
                IrType::Float => Operand::Reg(Reg::init(ScalarType::Float)),
                _ => unreachable!("cannot reach, param either int or float"),
            };
            map.val_map.insert(src, reg.clone());
            let (mut inum, mut fnum) = (0, 0);
            for p in params {
                match p.as_ref().get_param_type() {
                    IrType::Int => {
                        if src == *p {
                            if inum < ARG_REG_COUNT {
                                let inst = LIRInst::new(
                                    InstrsType::OpReg(SingleOp::IMv),
                                    vec![
                                        reg.clone(),
                                        Operand::Reg(Reg::new(inum + 10, ScalarType::Int)),
                                    ],
                                );
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));
                            } else {
                                let inst = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![
                                        reg.clone(),
                                        Operand::IImm(IImm::new(
                                            inum - ARG_REG_COUNT + max(fnum - ARG_REG_COUNT, 0) * 4,
                                        )),
                                    ],
                                );
                                self.insts.push(pool.put_inst(inst));
                            }
                        }
                        inum += 1;
                    }
                    IrType::Float => {
                        if src == *p {
                            if fnum < ARG_REG_COUNT {
                                let inst = LIRInst::new(
                                    InstrsType::OpReg(SingleOp::FMv),
                                    vec![
                                        reg.clone(),
                                        Operand::Reg(Reg::new(fnum + 10, ScalarType::Float)),
                                    ],
                                );
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));
                            } else {
                                let inst = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![
                                        reg.clone(),
                                        Operand::IImm(IImm::new(
                                            fnum - ARG_REG_COUNT + max(inum - ARG_REG_COUNT, 0) * 4,
                                        )),
                                    ],
                                );
                                self.insts.push(pool.put_inst(inst));
                            }
                        }
                        fnum += 1;
                    }
                    _ => unreachable!("cannot reach, param either int or float"),
                }
            }
            reg
        } else {
            map.val_map.get(&src).unwrap().clone()
        }
    }

    fn resolve_global(
        &mut self,
        src: ObjPtr<Inst>,
        map: &mut Mapping,
        pool: &mut BackendPool,
    ) -> Operand {
        if !self.global_map.contains_key(&src) {
            let reg = match src.as_ref().get_ir_type() {
                IrType::Int => Operand::Reg(Reg::init(ScalarType::Int)),
                IrType::Float => Operand::Reg(Reg::init(ScalarType::Float)),
                _ => unreachable!("cannot reach, global var is either int or float"),
            };
            self.global_map.insert(src, reg.clone());
            // let global_num = get_current_global_seq();
            // self.label = String::from(format!(".Lpcrel_hi{global_num}"));
            // inc_global_seq();
            assert!(map.val_map.contains_key(&src));
            let global_name = match map.val_map.get(&src) {
                Some(Operand::Addr(addr)) => addr,
                _ => unreachable!("cannot reach, global var must be addr"),
            };
            let inst = LIRInst::new(
                InstrsType::OpReg(SingleOp::LoadAddr),
                vec![reg.clone(), Operand::Addr(global_name.clone())],
            );
            self.insts.push(pool.put_inst(inst));
            reg
        } else {
            println!("find!");
            return self.global_map.get(&src).unwrap().clone();
        }
    }

    fn resolve_opt_mul(&mut self, dst: Operand, src: Operand, imm: i32) {
        //TODO: 暂时不使用优化
    }

    fn resolve_opt_div(&mut self, dst: Operand, src: Operand, imm: i32, pool: &mut BackendPool) {
        //TODO: 暂时不使用优化
        let reg = self.resolve_iimm(imm, pool);
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::Binary(BinaryOp::Div),
            vec![dst, src, reg],
        )));
    }

    fn resolve_opt_rem(
        &mut self,
        func: ObjPtr<Func>,
        map: &mut Mapping,
        dst: Operand,
        lhs: ObjPtr<Inst>,
        rhs: ObjPtr<Inst>,
        pool: &mut BackendPool,
    ) {
        //TODO:
        let lhs_reg = self.resolve_operand(func, lhs, true, map, pool);
        let rhs_reg = self.resolve_operand(func, rhs, true, map, pool);
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::Binary(BinaryOp::Rem),
            vec![dst, lhs_reg, rhs_reg],
        )));
    }

    // fn clear_reg_info(&mut self) {
    //     self.live_def.clear();
    //     self.live_use.clear();
    //     self.live_in.clear();
    //     self.live_out.clear();
    // }
}
impl GenerateAsm for BB {
    fn generate(&mut self, context: ObjPtr<Context>, f: &mut File) -> Result<()> {
        if self.called {
            print!("{}:\n", self.label);
        }
        for inst in self.insts.iter() {
            inst.as_mut().generate(context.clone(), f)?;
        }
        Ok(())
    }
}

fn is_opt_mul(imm: i32) -> bool {
    //FIXME:暂时不使用优化
    false
}

fn is_opt_num(imm: i32) -> bool {
    //FIXME:暂时不使用优化
    // (imm & (imm - 1)) == 0
    false
}

// fn get_current_global_seq() -> i32 {
//     unsafe { GLOBAL_SEQ }
// }

// fn inc_global_seq() {
//     unsafe {
//         GLOBAL_SEQ += 1;
//     }
// }

fn get_current_array_num() -> i32 {
    unsafe { ARRAY_NUM }
}

fn inc_array_num() {
    unsafe {
        ARRAY_NUM += 1;
    }
}

fn get_tmp_bb() -> i32 {
    unsafe {
        TMP_BB += 1;
        TMP_BB
    }
}
