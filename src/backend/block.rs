pub use crate::log;
use std::cmp::max;
use std::cmp::min;
pub use std::collections::{HashSet, VecDeque};
pub use std::fs::File;
pub use std::hash::{Hash, Hasher};
pub use std::io::Result;
use std::vec;

use crate::backend::func::Func;
use crate::backend::instrs::Operand;
use crate::backend::instrs::{BinaryOp, CmpOp, InstrsType, LIRInst, SingleOp};
use crate::backend::operand::{IImm, Reg};
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::{BinOp, Inst, InstKind, UnOp};
use crate::ir::ir_type::IrType;
use crate::utility::{ObjPtr, ScalarType};

use super::instrs::AsmBuilder;
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
    pub showed: bool,

    pub insts: Vec<ObjPtr<LIRInst>>,

    pub in_edge: Vec<ObjPtr<BB>>,
    pub out_edge: Vec<ObjPtr<BB>>,

    pub live_use: HashSet<Reg>,
    pub live_def: HashSet<Reg>,
    pub live_in: HashSet<Reg>,
    pub live_out: HashSet<Reg>,

    pub phis: Vec<ObjPtr<LIRInst>>,

    global_map: HashMap<ObjPtr<Inst>, Operand>,
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
        log!(">>>>{}", block.get_name());
        if block.is_empty() {
            self.showed = false;
            return;
        }
        let mut ir_block_inst = block.as_ref().get_head_inst();
        loop {
            let inst_ref = ir_block_inst.as_ref();
            log!("inst_ref: {:?}", inst_ref.get_kind());
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
                            // log!("lhs_reg: {:?}", lhs_reg);
                            // log!("rhs_reg: {:?}", rhs_reg);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![dst_reg, lhs_reg, rhs_reg],
                            )));
                        }
                        BinOp::Mul => {
                            lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                            rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Binary(BinaryOp::Mul),
                                vec![dst_reg, lhs_reg, rhs_reg],
                            )));
                            // let mut imm = 0;
                            // let limm = match lhs.as_ref().get_kind() {
                            //     InstKind::ConstInt(limm) => {
                            //         imm = limm;
                            //         true
                            //     }
                            //     _ => false,
                            // };
                            // let rimm = match rhs.as_ref().get_kind() {
                            //     InstKind::ConstInt(rimm) => {
                            //         imm = rimm;
                            //         true
                            //     }
                            //     _ => false,
                            // };
                            // if rimm {
                            //     let src = self.resolve_operand(func, lhs, true, map_info, pool);
                            //     self.resolve_opt_mul(dst_reg, src, imm, pool);
                            // } else if limm {
                            //     let src = self.resolve_operand(func, rhs, true, map_info, pool);
                            //     self.resolve_opt_mul(dst_reg, src, imm, pool);
                            // } else {
                            //     lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                            //     rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                            //     self.insts.push(pool.put_inst(LIRInst::new(
                            //         InstrsType::Binary(BinaryOp::Mul),
                            //         vec![dst_reg, lhs_reg, rhs_reg],
                            //     )));
                            // }
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
                                let iimm = match imm {
                                    0 => self.resolve_iimm(1, pool),
                                    _ => self.resolve_iimm(0, pool),
                                };
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
                                        InstrsType::OpReg(SingleOp::Seqz),
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
                                // 找不到，认为是全局数组或者参数，全局数组的访问是load -> gep -> load -> alloca
                                src_reg = self.resolve_operand(func, index, true, map_info, pool);
                            }
                            match addr.get_gep_offset().get_kind() {
                                InstKind::ConstInt(imm) | InstKind::GlobalConstInt(imm) => {
                                    let offset = imm * 4;
                                    let dst_reg = self.resolve_operand(
                                        func,
                                        ir_block_inst,
                                        true,
                                        map_info,
                                        pool,
                                    );
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Load,
                                        vec![dst_reg, src_reg, Operand::IImm(IImm::new(offset))],
                                    )));
                                }
                                _ => {
                                    let offset = self.resolve_operand(
                                        func,
                                        addr.get_gep_offset(),
                                        true,
                                        map_info,
                                        pool,
                                    );
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
                                        vec![dst_reg.clone(), src_reg.clone(), offset],
                                    );
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Load,
                                        vec![dst_reg.clone(), dst_reg, Operand::IImm(IImm::new(0))],
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
                    let value_reg = self.resolve_operand(func, value, true, map_info, pool);
                    match addr.as_ref().get_kind() {
                        InstKind::Gep => {
                            let mut load_new = true;
                            let mut addr_reg = match map_info.val_map.get(&addr.get_gep_ptr()) {
                                Some(reg) => {
                                    load_new = false;
                                    reg.clone()
                                }
                                None => Operand::Reg(Reg::init(ScalarType::Int)),
                            };
                            if let Some(base) = map_info.array_slot_map.get(&addr.get_gep_ptr()) {
                                if load_new {
                                    let mut load = LIRInst::new(
                                        InstrsType::LoadParamFromStack,
                                        vec![addr_reg.clone(), Operand::IImm(IImm::new(*base))],
                                    );
                                    load.set_double();
                                    self.insts.push(pool.put_inst(load));
                                }
                            } else {
                                // 找不到，认为是全局数组或者参数
                                addr_reg = self.resolve_operand(
                                    func,
                                    addr.get_gep_ptr(),
                                    true,
                                    map_info,
                                    pool,
                                );
                            }
                            match addr_reg {
                                // 使用全局数组，addr_reg获得的是地址，而非寄存器，因此需要加载
                                Operand::Addr(..) => {
                                    let addr = addr_reg.clone();
                                    addr_reg = Operand::Reg(Reg::init(ScalarType::Int));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::LoadAddr),
                                        vec![addr_reg.clone(), addr],
                                    )));
                                }
                                _ => {}
                            }
                            match addr.get_gep_offset().get_kind() {
                                InstKind::ConstInt(offset) | InstKind::GlobalConstInt(offset) => {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Store,
                                        vec![
                                            addr_reg.clone(),
                                            value_reg,
                                            Operand::IImm(IImm::new(offset * 4)),
                                        ],
                                    )));
                                }
                                _ => {
                                    let temp = self.resolve_operand(
                                        func,
                                        addr.get_gep_offset(),
                                        true,
                                        map_info,
                                        pool,
                                    );

                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Shl),
                                        vec![
                                            temp.clone(),
                                            temp.clone(),
                                            Operand::IImm(IImm::new(2)),
                                        ],
                                    )));
                                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                                    let mut inst = LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Add),
                                        vec![tmp.clone(), addr_reg.clone(), temp],
                                    );
                                    // log!("addr_reg: {:?}, value_reg: {:?}", addr_reg, value_reg);
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Store,
                                        vec![tmp, value_reg, Operand::IImm(IImm::new(0))],
                                    )));
                                }
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
                        if next_bb.is_empty() {
                            break;
                        }
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
                            let mut lhs_reg = Operand::IImm(IImm::new(0));
                            let mut rhs_reg = Operand::IImm(IImm::new(0));
                            if let Some(lhs_cond) = is_cond_op(cond_ref.get_lhs()) {
                                lhs_reg = self.resolve_bool(
                                    func,
                                    lhs_cond,
                                    cond_ref.get_lhs(),
                                    map_info,
                                    pool,
                                );
                            } else {
                                lhs_reg = self.resolve_operand(
                                    func,
                                    cond_ref.get_lhs(),
                                    true,
                                    map_info,
                                    pool,
                                );
                            }

                            if let Some(rhs_cond) = is_cond_op(cond_ref.get_rhs()) {
                                rhs_reg = self.resolve_bool(
                                    func,
                                    rhs_cond,
                                    cond_ref.get_rhs(),
                                    map_info,
                                    pool,
                                );
                            } else {
                                rhs_reg = self.resolve_operand(
                                    func,
                                    cond_ref.get_rhs(),
                                    true,
                                    map_info,
                                    pool,
                                );
                            }

                            let mut bz = false;
                            let inst_kind = match cond {
                                BinOp::Eq => InstrsType::Branch(CmpOp::Eq),
                                BinOp::Ne => InstrsType::Branch(CmpOp::Ne),
                                BinOp::Ge => InstrsType::Branch(CmpOp::Ge),
                                BinOp::Le => InstrsType::Branch(CmpOp::Le),
                                BinOp::Gt => InstrsType::Branch(CmpOp::Gt),
                                BinOp::Lt => InstrsType::Branch(CmpOp::Lt),
                                // BinOp::And =>
                                _ => {
                                    // 无法匹配，认为是if(a)情况，与0进行比较
                                    bz = true;
                                    InstrsType::Branch(CmpOp::Nez)
                                }
                            };
                            if bz {
                                let src_reg =
                                    self.resolve_operand(func, cond_ref, true, map_info, pool);
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    inst_kind,
                                    vec![
                                        Operand::Addr(false_succ_block.label.to_string()),
                                        src_reg,
                                    ],
                                )))
                            } else {
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    inst_kind,
                                    vec![
                                        Operand::Addr(false_succ_block.label.to_string()),
                                        lhs_reg,
                                        rhs_reg,
                                    ],
                                )));
                            }
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
                            // log!("{:?}", cond_ref.get_kind());
                            let lhs_reg =
                                self.resolve_operand(func, cond_ref, true, map_info, pool);
                            let inst_kind = InstrsType::Branch(CmpOp::Nez);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![Operand::Addr(false_succ_block.label.to_string()), lhs_reg],
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
                        let param_type = arg.get_param_type();
                        if param_type == IrType::Int
                            || param_type == IrType::IntPtr
                            || param_type == IrType::FloatPtr
                        {
                            icnt += 1
                        } else if param_type == IrType::Float {
                            fcnt += 1
                        } else {
                            unreachable!("call arg type not match, either be int or float")
                        }
                    }
                    let reg_cnt = min(icnt, ARG_REG_COUNT);
                    func.as_mut().max_params = max(reg_cnt, func.max_params);
                    for arg in arg_list.iter() {
                        log!("call arg: {:?}", arg.get_param_type());
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
                                    let dst_reg =
                                        Operand::Reg(Reg::new(10 + icnt, ScalarType::Int));
                                    let src_reg = match arg.get_kind() {
                                        InstKind::Gep => self.resolve_operand(
                                            func,
                                            arg.get_gep_ptr(),
                                            true,
                                            map_info,
                                            pool,
                                        ),
                                        _ => self.resolve_operand(func, *arg, true, map_info, pool),
                                    };
                                    let stack_addr = &func.as_ref().stack_addr;
                                    let mut pos = 0;
                                    if let Some(slot) = stack_addr.front() {
                                        pos = slot.get_pos() + ADDR_SIZE;
                                    } else {
                                        pos = ADDR_SIZE;
                                    }
                                    let slot = StackSlot::new(pos, ADDR_SIZE);
                                    func.as_mut().stack_addr.push_front(slot);
                                    func.as_mut().spill_stack_map.insert(icnt, slot);
                                    let mut inst = LIRInst::new(
                                        InstrsType::StoreParamToStack,
                                        vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))],
                                    );
                                    inst.set_double();
                                    //避免覆盖
                                    self.insts.push(pool.put_inst(inst));
                                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::IMv),
                                        vec![tmp.clone(), src_reg],
                                    )));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::IMv),
                                        vec![dst_reg.clone(), tmp],
                                    )));
                                }
                            }
                            IrType::Float | IrType::FloatPtr => {
                                unreachable!("param is float");
                                // fcnt -= 1;
                                // if fcnt >= ARG_REG_COUNT {
                                //     let src_reg =
                                //         self.resolve_operand(func, *arg, true, map_info, pool);
                                //     // 第后一个溢出参数在最下方（最远离sp位置）
                                //     let offset = Operand::IImm(IImm::new(
                                //         -(max(0, icnt - ARG_REG_COUNT)
                                //             + max(0, fcnt - ARG_REG_COUNT))
                                //             * 4,
                                //     ));
                                //     self.insts.push(pool.put_inst(LIRInst::new(
                                //         InstrsType::StoreToStack,
                                //         vec![src_reg, offset],
                                //     )));
                                // } else {
                                //     //FIXME:暂时不考虑浮点数参数
                                //     let src_reg =
                                //         self.resolve_operand(func, *arg, true, map_info, pool);
                                //     let dst_reg =
                                //         Operand::Reg(Reg::new(fcnt + 10, ScalarType::Float));
                                //     let stack_addr = &func.as_ref().stack_addr;
                                //     let pos = stack_addr.back().unwrap().get_pos() + ADDR_SIZE;
                                //     let size = 8;
                                //     func.as_mut()
                                //         .stack_addr
                                //         .push_back(StackSlot::new(pos, size));
                                //     let mut inst = LIRInst::new(
                                //         InstrsType::StoreParamToStack,
                                //         vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))],
                                //     );
                                //     inst.set_double();
                                //     self.insts.push(pool.put_inst(inst));
                                //     self.insts.push(pool.put_inst(LIRInst::new(
                                //         InstrsType::OpReg(SingleOp::FMv),
                                //         vec![dst_reg, src_reg],
                                //     )));
                                // }
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

                    // restore stack slot
                    let mut i = 1;
                    while i < ARG_REG_COUNT {
                        if let Some(slot) = func.as_ref().spill_stack_map.get(&i) {
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

                    // log!("phi reg: {:?}, tmp: {:?}", phi_reg.clone(), temp.clone());
                    self.phis
                        .push(pool.put_inst(LIRInst::new(inst_kind, vec![phi_reg, temp.clone()])));

                    let mut op_list: HashSet<ObjPtr<Inst>> = HashSet::new();
                    for (index, op) in ir_block_inst.get_operands().iter().enumerate() {
                        if !op_list.insert(*op) {
                            continue;
                        }
                        // log!("op: {:?}", op.get_kind());
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
                        // log!("save to insert phi inst: {:?}", inst);
                        let obj_inst = pool.put_inst(inst);
                        let incoming_block = map_info
                            .ir_block_map
                            .get(&ir_block_inst.get_phi_predecessor(index))
                            .unwrap()
                            .label
                            .clone();

                        if let Some(insts) = map_info.phis_to_block.get_mut(&incoming_block) {
                            // log!("insert phi inst: {:?}", obj_inst);
                            insts.insert(obj_inst);
                        } else {
                            // log!("insert phi inst: {:?}", obj_inst);
                            let mut set = HashSet::new();
                            set.insert(obj_inst);
                            map_info.phis_to_block.insert(incoming_block, set);
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

    pub fn save_reg(&mut self, func: ObjPtr<Func>, pool: &mut BackendPool) {
        let mut index = 0; 
        loop {
            if index >= self.insts.len() {
                break;
            }
            let inst = self.insts[index];
            for op in inst.operands.iter() {
                match op {
                    Operand::Reg(reg) => {
                        //FIXME: solve float regs
                        if let Some(phy_id) = func.reg_alloc_info.dstr.get(&reg.get_id()) {
                            let save_reg = Reg::new(*phy_id, reg.get_type());
                            if save_reg.is_caller_save() {
                                func.as_mut().caller_saved.insert(*phy_id, reg.get_id());
                            }
                            if save_reg.is_callee_save() {
                                func.as_mut().callee_saved.insert(save_reg);
                            }
                        } else {
                        }
                    }
                    _ => {}
                }
            }
            let mut caller_regs: HashSet<i32> = HashSet::new();
            let (icnt, fcnt) = inst.get_param_cnts();
            //FIXME: solve float regs
            match inst.get_type() {
                InstrsType::Call => {
                    for op in inst.get_reg_def().iter() {
                        if let Some(reg) = func.as_mut().caller_saved.get(&op.get_id()) {
                            if op.get_id() - 10 < icnt && op.get_id() >= 10 {
                                continue;
                            }
                            caller_regs.insert(*reg);
                        }
                    }
                    let mut pos = func.stack_addr.back().unwrap().get_pos();
                    pos += func.stack_addr.back().unwrap().get_size();
                    for (i, id) in caller_regs.iter().enumerate() {
                        let offset = pos + i as i32 * ADDR_SIZE;
                        let mut ins = LIRInst::new(
                            InstrsType::StoreToStack,
                            vec![
                                Operand::Reg(Reg::new(*id, ScalarType::Int)),
                                Operand::IImm(IImm::new(offset)),
                            ],
                        );
                        ins.set_double();
                        self.insts.insert(index, pool.put_inst(ins));
                        index += 1;
                    }
                    index += 1;
                    for (i, id) in caller_regs.iter().enumerate() {
                        let offset = pos + i as i32 * ADDR_SIZE;
                        let mut ins = LIRInst::new(
                            InstrsType::LoadFromStack,
                            vec![
                                Operand::Reg(Reg::new(*id, ScalarType::Int)),
                                Operand::IImm(IImm::new(offset)),
                            ],
                        );
                        ins.set_double();
                        self.insts.insert(index, pool.put_inst(ins));
                        index += 1;
                    }
                }
                _ => {
                    index += 1;
                }
            }
        }
    }

    pub fn handle_spill(
        &mut self,
        func: ObjPtr<Func>,
        spill: &HashSet<i32>,
        pool: &mut BackendPool,
    ) {
        self.save_reg(func, pool);
        let mut index = 0;
        loop {
            if index >= self.insts.len() {
                break;
            }
            let inst = self.insts[index];
            let spills = inst.is_spill(spill);
            log!("spills: {:?}", spills);
            if spills.is_empty() {
                index += 1;
                continue;
            } else {
                for (i, id) in spills.iter().enumerate() {
                    let reg = Operand::Reg(Reg::new(5 + (i as i32), ScalarType::Int));
                    if let Some(stack_slot) = func.spill_stack_map.get(&id) {
                        let mut ins = LIRInst::new(
                            InstrsType::LoadFromStack,
                            vec![reg, Operand::IImm(IImm::new(stack_slot.get_pos()))],
                        );
                        ins.set_double();
                        self.insts.insert(index, pool.put_inst(ins));
                        index += 1;
                    } else {
                        let last_slot = func.stack_addr.back().unwrap();
                        let mut pos = last_slot.get_pos() + last_slot.get_size();
                        let stack_slot = StackSlot::new(pos, ADDR_SIZE);
                        func.as_mut().stack_addr.push_back(stack_slot);
                        func.as_mut().spill_stack_map.insert(*id, stack_slot);
                    }
                }
                for (i, id) in spills.iter().enumerate() {
                    inst.as_mut().replace(*id, 5 + (i as i32))
                }
                index += 1;
                match inst.get_dst() {
                    Operand::Reg(_) => match inst.get_type() {
                        InstrsType::Store
                        | InstrsType::StoreParamToStack
                        | InstrsType::StoreToStack => {
                            continue;
                        }
                        _ => {}
                    },
                    _ => {
                        continue;
                    }
                }

                for (i, id) in spills.iter().enumerate() {
                    let reg = Operand::Reg(Reg::new(5 + (i as i32), ScalarType::Int));
                    let stack_slot = func.spill_stack_map.get(id).unwrap();
                    match self.insts[index - 1].get_dst() {
                        Operand::Reg(ireg) => {
                            if ireg.get_id() != 5 + (i as i32) {
                                continue;
                            }
                        }
                        _ => {}
                    }
                    let mut ins = LIRInst::new(
                        InstrsType::StoreToStack,
                        vec![reg, Operand::IImm(IImm::new(stack_slot.get_pos()))],
                    );
                    ins.set_double();
                    self.insts.insert(index, pool.put_inst(ins));
                    index += 1;
                }
            }
        }
        // log!("---------------------------");
        // log!("{:?}", func.spill_stack_map);
        // log!("---------------------------");
    }

    pub fn handle_overflow(&mut self, func: ObjPtr<Func>, pool: &mut BackendPool) {
        let mut pos = 0;
        log!("{}, len: {}", self.label, self.insts.len());
        loop {
            if pos >= self.insts.len() {
                break;
            }
            log!("inst: {:?}", self.insts[pos]);
            let inst_ref = self.insts[pos].as_ref();
            match inst_ref.get_type() {
                InstrsType::Load | InstrsType::Store => {
                    let temp = Operand::Reg(Reg::new(3, ScalarType::Int));
                    let offset = inst_ref.get_offset().get_data();
                    log!("offset {}", offset);
                    if operand::is_imm_12bs(offset) {
                        pos += 1;
                        continue;
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
                    let temp = Operand::Reg(Reg::new(3, ScalarType::Int));
                    let offset = inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        pos += 1;
                        continue;
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
                    let temp = Operand::Reg(Reg::new(3, ScalarType::Int));
                    let offset = func.as_ref().reg_alloc_info.stack_size as i32
                        - inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        pos += 1;
                        continue;
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
                        if i >= func.as_ref().blocks.len() {
                            break;
                        }
                        let block_ref = func.as_ref().blocks[i];
                        if &self.label == &block_ref.as_ref().label {
                            flag = true;
                        }
                        if &block_ref.as_ref().label == target {
                            //FIXME: this is a bug
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
        let lower = (offset << 20) >> 20;
        let upper = (offset - lower) >> 12;
        //取得高20位
        let op1 = Operand::IImm(IImm::new(upper));
        //取得低12位
        let op2 = Operand::IImm(IImm::new(lower));
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
                    _ => {
                        unreachable!("cannot reach, resolve_operand func, false, pool")
                    }
                };
                // log!("inst kind: {:?}", src.get_kind());
                // log!("new reg: {:?}", op);
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
            let lower = (imm << 20) >> 20;
            let upper = (imm - lower) >> 12;
            //取得高20位
            let op1 = Operand::IImm(IImm::new(upper));
            //取得低12位
            let op2 = Operand::IImm(IImm::new(lower));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Lui),
                vec![reg.clone(), op1],
            )));
            log!("op2: {:?}", op2);
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
                IrType::Int | IrType::IntPtr | IrType::FloatPtr => {
                    Operand::Reg(Reg::init(ScalarType::Int))
                }
                IrType::Float => Operand::Reg(Reg::init(ScalarType::Float)),
                _ => unreachable!("cannot reach, param either int or float"),
            };
            map.val_map.insert(src, reg.clone());
            let (mut inum, mut fnum) = (0, 0);
            // 由于寄存器分配策略，读取参数时需要先在函数开头把所有参数保存，再从寄存器中读取
            for p in params {
                match p.as_ref().get_param_type() {
                    IrType::Int | IrType::IntPtr | IrType::FloatPtr => {
                        if src == *p {
                            let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                            if inum < ARG_REG_COUNT {
                                let inst = LIRInst::new(
                                    InstrsType::OpReg(SingleOp::IMv),
                                    vec![
                                        tmp.clone(),
                                        Operand::Reg(Reg::new(inum + 10, ScalarType::Int)),
                                    ],
                                );
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));

                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::IMv),
                                    vec![reg.clone(), tmp],
                                )));
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
                                let tmp = Operand::Reg(Reg::init(ScalarType::Float));
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
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::FMv),
                                    vec![reg.clone(), tmp],
                                )));
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
                    _ => {
                        // log!("{:?}", p.get_param_type());
                        unreachable!("cannot reach, param must be int, float or ptr")
                    }
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
            // log!("find!");
            return self.global_map.get(&src).unwrap().clone();
        }
    }

    fn resolve_bool(
        &mut self,
        func: ObjPtr<Func>,
        cond: BinOp,
        src: ObjPtr<Inst>,
        map: &mut Mapping,
        pool: &mut BackendPool,
    ) -> Operand {
        let dst_reg = self.resolve_operand(func, src, true, map, pool);
        let mut lhs_reg = Operand::IImm(IImm::new(0));
        let mut rhs_reg = Operand::IImm(IImm::new(0));
        let lhs = src.get_lhs();
        let rhs = src.get_rhs();
        if let Some(lhs_cond) = is_cond_op(lhs) {
            lhs_reg = self.resolve_bool(func, lhs_cond, lhs, map, pool);
        } else {
            lhs_reg = self.resolve_operand(func, lhs, false, map, pool);
        };
        if let Some(rhs_cond) = is_cond_op(rhs) {
            rhs_reg = self.resolve_bool(func, rhs_cond, rhs, map, pool);
        } else {
            rhs_reg = self.resolve_operand(func, rhs, false, map, pool);
        }
        let is_limm = match lhs_reg {
            Operand::IImm(..) | Operand::FImm(..) => true,
            Operand::Reg(..) => false,
            Operand::Addr(..) => unreachable!("reg cannot be addr"),
        };
        let is_rimm = match rhs_reg {
            Operand::IImm(..) | Operand::FImm(..) => true,
            Operand::Reg(..) => false,
            Operand::Addr(..) => unreachable!("reg cannot be addr"),
        };
        match cond {
            BinOp::Eq | BinOp::Ne => {
                // 允许交换
                if is_limm {
                    if !is_rimm {
                        let tmp = rhs_reg.clone();
                        rhs_reg = lhs_reg.clone();
                        lhs_reg = tmp;
                    } else {
                        lhs_reg = self.resolve_operand(func, lhs, true, map, pool)
                    }
                }
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Xor),
                    vec![dst_reg.clone(), lhs_reg, rhs_reg],
                )));
                match cond {
                    BinOp::Eq => self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Seqz),
                        vec![dst_reg.clone(), dst_reg.clone()],
                    ))),
                    BinOp::Ne => self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Snez),
                        vec![dst_reg.clone(), dst_reg.clone()],
                    ))),
                    _ => unreachable!("no more cond"),
                }
            }
            // 不允许交换
            BinOp::Lt => {
                if is_limm {
                    lhs_reg = self.resolve_operand(func, lhs, true, map, pool);
                }
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Slt),
                    vec![dst_reg.clone(), lhs_reg, rhs_reg],
                )))
            }
            BinOp::Gt => {
                // a > b 变为 b < a
                if is_rimm {
                    rhs_reg = self.resolve_operand(func, rhs, true, map, pool);
                }
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Slt),
                    vec![dst_reg.clone(), rhs_reg, lhs_reg],
                )))
            }
            BinOp::Le => {
                // a <= b 变为 !(b < a)
                if is_rimm {
                    rhs_reg = self.resolve_operand(func, rhs, true, map, pool);
                }
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Slt),
                    vec![dst_reg.clone(), rhs_reg, lhs_reg],
                )));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Xor),
                    vec![
                        dst_reg.clone(),
                        dst_reg.clone(),
                        Operand::IImm(IImm::new(1)),
                    ],
                )));
            }
            BinOp::Ge => {
                // a >= b 变为 !(a < b)
                if is_limm {
                    lhs_reg = self.resolve_operand(func, lhs, true, map, pool);
                }
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Slt),
                    vec![dst_reg.clone(), lhs_reg, rhs_reg],
                )));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Xor),
                    vec![
                        dst_reg.clone(),
                        dst_reg.clone(),
                        Operand::IImm(IImm::new(1)),
                    ],
                )));
            }
            _ => unreachable!("cond not illegal"),
        }
        dst_reg
    }

    fn resolve_opt_mul(&mut self, dst: Operand, src: Operand, imm: i32, pool: &mut BackendPool) {
        let abs = imm.abs();
        let is_neg = imm < 0;
        match abs {
            0 => {
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::OpReg(SingleOp::IMv),
                    vec![dst, Operand::IImm(IImm::new(0))],
                )));
            }
            1 => {
                if !is_neg {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::IMv),
                        vec![dst, src],
                    )));
                } else {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::INeg),
                        vec![dst, src],
                    )));
                }
            }
            _ => {
                if is_opt_num(abs) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![dst.clone(), src, Operand::IImm(IImm::new(log2(abs)))],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else if is_opt_num(abs - 1) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![
                            dst.clone(),
                            src.clone(),
                            Operand::IImm(IImm::new(log2(abs - 1))),
                        ],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![dst.clone(), dst.clone(), src],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else if is_opt_num(abs + 1) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![
                            dst.clone(),
                            src.clone(),
                            Operand::IImm(IImm::new(log2(abs + 1))),
                        ],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sub),
                        vec![dst.clone(), dst.clone(), src],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else {
                    let (mut power, mut opt_abs, mut do_add, mut can_opt) = (0, 0, false, false);
                    while (1 << power) <= abs {
                        if is_opt_num(abs + (1 << power)) {
                            do_add = true;
                            opt_abs = abs + (1 << power);
                            can_opt = true;
                            break;
                        }
                        if is_opt_num(abs - (1 << power)) {
                            opt_abs = abs - (1 << power);
                            can_opt = true;
                            break;
                        }
                        power += 1;
                    }
                    let temp = Operand::Reg(Reg::init(ScalarType::Int));
                    if !can_opt {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Li),
                            vec![temp.clone(), Operand::IImm(IImm::new(imm))],
                        )));
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::Binary(BinaryOp::Mul),
                            vec![dst, src, temp],
                        )));
                        return;
                    }
                    let bits = log2(opt_abs);
                    let combine_inst_kind = match do_add {
                        true => InstrsType::Binary(BinaryOp::Add),
                        false => InstrsType::Binary(BinaryOp::Sub),
                    };
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![temp.clone(), src.clone(), Operand::IImm(IImm::new(power))],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![dst.clone(), src.clone(), Operand::IImm(IImm::new(bits))],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        combine_inst_kind,
                        vec![dst.clone(), dst.clone(), temp],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::INeg),
                            vec![dst.clone(), dst],
                        )))
                    }
                }
            }
        }
    }

    fn resolve_opt_div(&mut self, dst: Operand, src: Operand, imm: i32, pool: &mut BackendPool) {
        let abs = imm.abs();
        let is_neg = imm < 0;
        match abs {
            0 => {
                unreachable!("div by zero");
            }
            1 => {
                if is_neg {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::INeg),
                        vec![dst, src],
                    )))
                } else {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::IMv),
                        vec![dst, src],
                    )))
                }
            }
            _ => {
                if is_opt_num(abs) {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sar),
                        vec![dst, src, Operand::IImm(IImm::new(log2(abs)))],
                    )))
                } else {
                    let (two31, uabs, mut p, mut delta) =
                        (1 << 31 as u32, abs as u32, 31, 0 as u32);
                    let t = two31 + (uabs >> 31);
                    let anc = t - 1 - t % uabs;
                    let (mut q1, mut q2) = (two31 / anc, two31 / uabs);
                    let (mut r1, mut r2) = (two31 - q1 * anc, two31 - q2 * uabs);

                    loop {
                        p += 1;
                        q1 *= 2;
                        r1 *= 2;

                        if r1 >= anc {
                            q1 += 1;
                            r1 -= anc;
                        }
                        q2 *= 2;
                        r2 *= 2;
                        if r2 >= uabs {
                            q2 += 1;
                            r2 -= uabs;
                        }
                        delta = uabs - r2;
                        if q1 < delta || (q1 == delta && r1 == 0) {
                            break;
                        }
                    }

                    let mut magic = (q2 + 1) as i32;
                    if is_neg {
                        magic = -magic;
                    }
                    let shift = p - 32;
                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Li),
                        vec![tmp.clone(), Operand::IImm(IImm::new(magic))],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Mulhs),
                        vec![tmp.clone(), tmp.clone(), src.clone()],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![tmp.clone(), src.clone(), tmp.clone()],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shr),
                        vec![tmp.clone(), tmp.clone(), Operand::IImm(IImm::new(shift))],
                    )));
                }
            }
        }
    }

    fn resolve_opt_rem(
        &mut self,
        func: ObjPtr<Func>,
        map: &mut Mapping,
        dst: Operand,
        lhs: ObjPtr<Inst>,
        imm: i32,
        pool: &mut BackendPool,
    ) {
        let lhs_reg = self.resolve_operand(func, lhs, true, map, pool);
        let abs = imm.abs();
        let is_neg = imm < 0;
        if is_opt_num(abs) {
            let k = log2(abs);
            // r = ((n + t) & (2^k - 1)) - t
            // t = (n >> k - 1) >> 32 - k
            let tmp = Operand::Reg(Reg::init(ScalarType::Int));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Sar),
                vec![
                    tmp.clone(),
                    lhs_reg.clone(),
                    Operand::IImm(IImm::new(k - 1)),
                ],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Shr),
                vec![tmp.clone(), tmp.clone(), Operand::IImm(IImm::new(32 - k))],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Add),
                vec![dst.clone(), dst.clone(), tmp.clone()],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::And),
                vec![dst.clone(), dst.clone(), Operand::IImm(IImm::new(abs - 1))],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Sub),
                vec![dst.clone(), dst.clone(), tmp.clone()],
            )));
        } else {
            let rhs_reg = self.load_iimm_to_ireg(imm, pool);
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Rem),
                vec![dst, lhs_reg, rhs_reg],
            )));
        }
    }

    // fn clear_reg_info(&mut self) {
    //     self.live_def.clear();
    //     self.live_use.clear();
    //     self.live_in.clear();
    //     self.live_out.clear();
    // }
    pub fn generate_row(&mut self, context: ObjPtr<Context>, f: &mut File) -> Result<()> {
        if self.showed {
            let mut builder = AsmBuilder::new(f);
            builder.show_block(&self.label)?;
        }
        context.as_mut().is_row = true;
        for inst in self.insts.iter() {
            inst.as_mut().generate(context.clone(), f)?;
        }
        Ok(())
    }
}
impl GenerateAsm for BB {
    fn generate(&mut self, context: ObjPtr<Context>, f: &mut File) -> Result<()> {
        if self.showed {
            let mut builder = AsmBuilder::new(f);
            builder.show_block(&self.label)?;
        }
        context.as_mut().is_row = false;
        for inst in self.insts.iter() {
            log!("{:?}, generate", self.label);
            inst.as_mut().v_to_phy(context.get_reg_map().clone());
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
    (imm & (imm - 1)) == 0
}

fn log2(imm: i32) -> i32 {
    assert!(is_opt_num(imm));
    let mut res = 0;
    let mut tmp = imm;
    while tmp != 1 {
        tmp >>= 1;
        res += 1;
    }
    res
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

fn is_cond_op(cond: ObjPtr<Inst>) -> Option<BinOp> {
    match cond.get_kind() {
        InstKind::Binary(cmp) => match cmp {
            BinOp::Eq | BinOp::Ne | BinOp::Ge | BinOp::Le | BinOp::Gt | BinOp::Lt => Some(cmp),
            _ => None,
        },
        _ => None,
    }
}
