pub use std::cmp::max;
pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
pub use std::vec;


pub use crate::backend::func::Func;
pub use crate::backend::instrs::Operand;
pub use crate::backend::instrs::{BinaryOp, CmpOp, InstrsType, LIRInst, SingleOp};
pub use crate::backend::operand::{IImm, Reg, F_REG_ID, I_REG_ID};
pub use crate::ir::basicblock::BasicBlock;
pub use crate::ir::instruction::{BinOp, Inst, InstKind, UnOp};
pub use crate::ir::ir_type::IrType;
pub use crate::utility::{ObjPtr, ScalarType};

pub use super::instrs::AsmBuilder;
pub use super::operand::ARG_REG_COUNT;
pub use super::{structs::*, BackendPool};
pub use crate::backend::operand;

pub static mut ARRAY_NUM: i32 = 0;
// pub static mut GLOBAL_SEQ: i32 = 0;
pub static mut TMP_BB: i32 = 0;

pub const ADDR_SIZE: i32 = 8;
pub const NUM_SIZE: i32 = 4;

#[derive(Clone)]
pub struct BB {
    pub label: String,
    pub showed: bool,

    pub insts: Vec<ObjPtr<LIRInst>>,

    pub in_edge: Vec<ObjPtr<BB>>,
    pub out_edge: Vec<ObjPtr<BB>>,

    pub live_use: HashSet<Reg>,
    pub live_def: HashSet<Reg>,
    pub live_in: HashSet<Reg>,
    pub live_out: HashSet<Reg>,

    pub phis: Vec<ObjPtr<LIRInst>>,

    pub global_map: HashMap<ObjPtr<Inst>, Operand>,
}

impl BB {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            showed: true,
            insts: Vec::new(),
            in_edge: Vec::new(),
            out_edge: Vec::new(),
            live_use: HashSet::new(),
            live_def: HashSet::new(),
            live_in: HashSet::new(),
            live_out: HashSet::new(),
            global_map: HashMap::new(),
            phis: Vec::new(),
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
                            let mut imm = 0;
                            let limm = match lhs.as_ref().get_kind() {
                                InstKind::ConstInt(limm) => {
                                    imm = limm;
                                    true
                                }
                                _ => false,
                            };
                            let rimm = match rhs.as_ref().get_kind() {
                                InstKind::ConstInt(rimm) => {
                                    imm = rimm;
                                    true
                                }
                                _ => false,
                            };
                            if rimm {
                                let src = self.resolve_operand(func, lhs, true, map_info, pool);
                                self.resolve_opt_mul(dst_reg, src, imm, pool);
                            } else if limm {
                                let src = self.resolve_operand(func, rhs, true, map_info, pool);
                                self.resolve_opt_mul(dst_reg, src, imm, pool);
                            } else {
                                lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                                rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::Binary(BinaryOp::Mul),
                                    vec![dst_reg, lhs_reg, rhs_reg],
                                )));
                            }
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
                                InstKind::ConstInt(imm) => match imm {
                                    0 => {
                                        lhs_reg =
                                            self.resolve_operand(func, lhs, true, map_info, pool);
                                        self.insts.push(pool.put_inst(LIRInst::new(
                                            InstrsType::OpReg(SingleOp::IMv),
                                            vec![dst_reg, lhs_reg],
                                        )));
                                    }
                                    1 | -1 => {
                                        self.insts.push(pool.put_inst(LIRInst::new(
                                            InstrsType::OpReg(SingleOp::Li),
                                            vec![dst_reg, Operand::IImm(IImm::new(0))],
                                        )));
                                    }
                                    _ => {
                                        // self.resolve_opt_rem(
                                        //     func, map_info, dst_reg, lhs, imm, pool,
                                        // );
                                        lhs_reg =
                                            self.resolve_operand(func, lhs, true, map_info, pool);
                                        rhs_reg =
                                            self.resolve_operand(func, rhs, true, map_info, pool);
                                        self.insts.push(pool.put_inst(LIRInst::new(
                                            InstrsType::Binary(BinaryOp::Rem),
                                            vec![dst_reg, lhs_reg, rhs_reg],
                                        )));
                                    }
                                },
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
                        _ => {}
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
                            let mut src_reg = Operand::IImm(IImm::new(0));
                            let index = addr.as_ref().get_gep_ptr();
                            if let Some(head) = map_info.array_slot_map.get(&index) {
                                //TODO:判断地址合法
                                let mut load = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![dst_reg.clone(), Operand::IImm(IImm::new(*head))],
                                );
                                load.set_double();
                                self.insts.push(pool.put_inst(load));
                                src_reg = dst_reg.clone();
                            } else {
                                // 找不到，认为是全局数组，全局数组的访问是load -> gep -> load -> alloca
                                src_reg =
                                    self.resolve_operand(func, index, true, map_info, pool);
                            }
                            match addr.get_gep_offset().get_kind() {
                                InstKind::ConstInt(imm) | InstKind::GlobalConstInt(imm) | InstKind::GlobalInt(imm) => {
                                    let offset = imm * 4;
                                    let dst_reg =
                                        self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                                        self.insts.push(pool.put_inst(LIRInst::new(
                                            InstrsType::Load,
                                            vec![
                                                dst_reg,
                                                src_reg,
                                                Operand::IImm(IImm::new(offset)),
                                            ],
                                        )));
                                }
                                _ => {
                                    let offset = self.resolve_operand(func, addr.get_gep_offset(), true, map_info, pool);
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Shl),
                                        vec![
                                            offset.clone(),
                                            offset.clone(),
                                            Operand::IImm(IImm::new(2)),
                                        ],
                                    )));
                                    let mut inst = LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Add),
                                        vec![
                                            dst_reg.clone(),
                                            src_reg.clone(),
                                            offset,
                                        ],
                                    );
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Load,
                                        vec![
                                            dst_reg.clone(),
                                            dst_reg,
                                            Operand::IImm(IImm::new(0)),
                                        ],
                                    )));
                                }
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
                            inst.replace_op(vec![Operand::Addr(jump_block.label.to_string())]);
                            let obj_inst = pool.put_inst(inst);
                            self.insts.push(obj_inst);
                            map_info.block_branch.insert(self.label.clone(), obj_inst);
                        }
                        let this = self.clone();
                        jump_block.as_mut().in_edge.push(pool.put_block(this));
                        self.out_edge.push(*jump_block);
                        break;
                    }

                    // if branch
                    let cond_ref = inst_ref.get_br_cond();

                    let false_cond_bb = block.as_ref().get_next_bb()[0];
                    let true_cond_bb = block.as_ref().get_next_bb()[1];
                    let block_map = map_info.ir_block_map.clone();
                    let true_succ_block = match block_map.get(&false_cond_bb) {
                        Some(block) => block,
                        None => unreachable!("true block not found"),
                    };
                    let false_succ_block = match block_map.get(&true_cond_bb) {
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
                                // BinOp::And => 
                                _ => {
                                    println!("{:?}", cond);
                                    println!("left{:?}", cond_ref.get_lhs().get_kind());
                                    println!("right{:?}", cond_ref.get_rhs().get_kind());
                                    unreachable!("no condition match")
                                }
                            };
                            self.insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![
                                    Operand::Addr(false_succ_block.label.to_string()),
                                    lhs_reg,
                                    rhs_reg,
                                ],
                            )));
                            self.push_back(pool.put_inst(LIRInst::new(
                                InstrsType::Jump,
                                vec![Operand::Addr(true_succ_block.label.to_string())],
                            )));

                            inst.replace_op(vec![Operand::Addr(
                                true_cond_bb.as_ref().get_name().to_string(),
                            )]);
                            let obj_inst = pool.put_inst(inst);
                            map_info.block_branch.insert(self.label.clone(), obj_inst);
                            let this = self.clone();
                            true_succ_block
                                .as_mut()
                                .in_edge
                                .push(pool.put_block(this.clone()));
                            false_succ_block.as_mut().in_edge.push(pool.put_block(this));
                            self.out_edge
                                .append(vec![*true_succ_block, *false_succ_block].as_mut());
                        }
                        _ => {
                            println!("{:?}", cond_ref.get_kind());
                            let lhs_reg = self.resolve_operand(
                                func,
                                cond_ref,
                                true,
                                map_info,
                                pool,
                            );
                            let inst_kind = InstrsType::Branch(CmpOp::Eqz);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![
                                    Operand::Addr(false_succ_block.label.to_string()),
                                    lhs_reg,
                                ],
                            )));
                            self.push_back(pool.put_inst(LIRInst::new(
                                InstrsType::Jump,
                                vec![Operand::Addr(true_succ_block.label.to_string())],
                            )));

                            inst.replace_op(vec![Operand::Addr(
                                true_cond_bb.as_ref().get_name().to_string(),
                            )]);
                            let obj_inst = pool.put_inst(inst);
                            map_info.block_branch.insert(self.label.clone(), obj_inst);
                            let this = self.clone();
                            true_succ_block
                                .as_mut()
                                .in_edge
                                .push(pool.put_block(this.clone()));
                            false_succ_block.as_mut().in_edge.push(pool.put_block(this));
                            self.out_edge
                                .append(vec![*true_succ_block, *false_succ_block].as_mut());
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
                                    let pos = stack_addr.front().unwrap().get_pos() + ADDR_SIZE;
                                    let size = ADDR_SIZE;
                                    let slot = StackSlot::new(pos, size);
                                    func.as_mut().stack_addr.push_front(slot);
                                    func.as_mut().spill_stack_map.insert(icnt, slot);
                                    println!("save spill stack map {:?}", func.spill_stack_map);
                                    println!("{icnt}: {}", pos);
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
                                    let pos = stack_addr.back().unwrap().get_pos() + ADDR_SIZE;
                                    let size = ADDR_SIZE;
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
                            println!("restore stack map {:?}", func.spill_stack_map);
                            println!("{i}: {}", slot.get_pos());
                            let mut inst = LIRInst::new(
                                InstrsType::LoadParamFromStack,
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
                    let mut inst_kind = match kind {
                        ScalarType::Int => InstrsType::OpReg(SingleOp::IMv),
                        ScalarType::Float => InstrsType::OpReg(SingleOp::FMv),
                        _ => unreachable!("mv must be int or float"),
                    };

                    println!("phi reg: {:?}, tmp: {:?}", phi_reg.clone(), temp.clone());
                    self.phis
                        .push(pool.put_inst(LIRInst::new(inst_kind, vec![phi_reg, temp.clone()])));

                    let mut op_list : HashSet<ObjPtr<Inst>> = HashSet::new();
                    for op in ir_block_inst.get_operands().iter() {
                        if !op_list.insert(*op) {
                            continue
                        }
                        println!("op: {:?}", op.get_kind());
                        let src_reg = self.resolve_operand(func, *op, false, map_info, pool);
                        inst_kind = match src_reg {
                            Operand::Reg(reg) => match reg.get_type() {
                                ScalarType::Int => InstrsType::OpReg(SingleOp::IMv),
                                ScalarType::Float => InstrsType::OpReg(SingleOp::FMv),
                                _ => unreachable!("mv must be int or float"),
                            },
                            Operand::IImm(_) => InstrsType::OpReg(SingleOp::Li),
                            _ => unreachable!("phi operand must be reg or iimm"),
                        };
                        let inst = LIRInst::new(inst_kind, vec![temp.clone(), src_reg]);
                        println!("save to insert phi inst: {:?}", inst);
                        let obj_inst = pool.put_inst(inst);
                        let incoming_block = map_info
                            .ir_block_map
                            //FIXME: 对接phi优化
                            .get(&op.as_ref().get_parent_bb())
                            .unwrap()
                            .label
                            .clone();

                        if let Some(insts) = map_info.phis_to_block.get_mut(&incoming_block) {
                            println!("insert phi inst: {:?}", obj_inst);
                            insts.insert(obj_inst);
                        } else {
                            println!("insert phi inst: {:?}", obj_inst);
                            let mut set = HashSet::new();
                            set.insert(obj_inst);
                            map_info
                                .phis_to_block
                                .insert(incoming_block, set);
                        }
                    }
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

    // fn clear_reg_info(&mut self) {
    //     self.live_def.clear();
    //     self.live_use.clear();
    //     self.live_in.clear();
    //     self.live_out.clear();
    // }
}
impl GenerateAsm for BB {
    fn generate(&mut self, context: ObjPtr<Context>, f: &mut File) -> Result<()> {
        if self.showed {
            let mut builder = AsmBuilder::new(f);
            builder.show_block(&self.label)?;
        }
        for inst in self.insts.iter() {
            // println!("generate inst: {:?}", inst);
            inst.as_mut().v_to_phy(context.get_reg_map().clone());
            inst.as_mut().generate(context.clone(), f)?;
        }
        Ok(())
    }
}


fn get_current_array_num() -> i32 {
    unsafe { ARRAY_NUM }
}

fn inc_array_num() {
    unsafe {
        ARRAY_NUM += 1;
    }
}