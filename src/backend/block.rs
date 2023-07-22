use crate::config;
pub use crate::log;
// use crate::log_file;
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
use super::operand::{FImm, ToString};
use super::{structs::*, BackendPool};
use crate::backend::operand;

pub static mut ARRAY_NUM: i32 = 0;
// pub static mut GLOBAL_SEQ: i32 = 0;
pub static mut TMP_BB: i32 = 0;

pub const ADDR_SIZE: i32 = 8;
pub const NUM_SIZE: i32 = 4;
pub const FLOAT_BASE: i32 = 32;

#[derive(Clone)]
pub struct BB {
    pub label: String,
    pub func_label: String,
    pub showed: bool,

    pub insts: Vec<ObjPtr<LIRInst>>,

    pub in_edge: Vec<ObjPtr<BB>>,
    pub out_edge: Vec<ObjPtr<BB>>,

    pub live_use: HashSet<Reg>,
    pub live_def: HashSet<Reg>,
    pub live_in: HashSet<Reg>,
    pub live_out: HashSet<Reg>,

    /// key->val : (reg,born)->(reg,die)
    /// * 如果reg 在live in中,则对应born为-1,
    /// * 如果reg 在下标 i1 处指令中def, 则对应 key (reg,born)=(reg, i1)
    /// * 如果(reg,born) 在这次生命周期中最后一次使用在 i2, 则对应val (reg,die)=(reg,i2)
    pub reg_intervals: HashMap<(Reg, i32), (Reg, i32)>,

    pub phis: Vec<ObjPtr<LIRInst>>,

    global_map: HashMap<ObjPtr<Inst>, Operand>,
}

impl BB {
    pub fn new(label: &str, func_label: &str) -> Self {
        Self {
            label: label.to_string(),
            func_label: func_label.to_string(),
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
            reg_intervals: HashMap::new(),
        }
    }

    /// 寄存器分配时决定开栈大小、栈对象属性(size(4/8 bytes), pos)，回填func的stack_addr <br>
    /// 尽量保证程序顺序执行，并满足首次遇分支向后跳转的原则？ <br>
    /// FIXME: b型指令长跳转(目标地址偏移量为+-4KiB)，若立即数非法是否需要增添一个jal块实现间接跳转？
    pub fn construct(
        &mut self,
        func: ObjPtr<Func>,
        block: ObjPtr<BasicBlock>,
        next_blocks: Option<ObjPtr<BB>>,
        map_info: &mut Mapping,
        pool: &mut BackendPool,
    ) {
        // log!(">>>>{}", block.get_name());
        if block.is_empty() {
            self.showed = false;
            return;
        }
        let mut ir_block_inst = block.as_ref().get_head_inst();
        loop {
            let inst_ref = ir_block_inst.as_ref();
            // log!("inst_ref: {:?}", inst_ref.get_kind());
            // translate ir to lir, use match
            match inst_ref.get_kind() {
                InstKind::Binary(op) => {
                    let lhs = inst_ref.get_lhs();
                    let rhs = inst_ref.get_rhs();
                    let lhs_reg;
                    let rhs_reg;
                    let dst_reg: Operand =
                        self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    match op {
                        //TODO: Float Binary
                        BinOp::Add => {
                            let inst_kind = InstrsType::Binary(BinaryOp::Add);
                            match lhs.as_ref().get_kind() {
                                //立即数
                                InstKind::ConstInt(..) | InstKind::ConstFloat(..) => {
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

                        // 除法不允许调换左右操作数次序
                        BinOp::Div => {
                            lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                            match rhs.as_ref().get_kind() {
                                InstKind::ConstInt(imm) | InstKind::GlobalConstInt(imm) => {
                                    self.resolve_opt_div(dst_reg, lhs_reg, imm, pool)
                                    // rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                                    // self.insts.push(pool.put_inst(LIRInst::new(
                                    //     InstrsType::Binary(BinaryOp::Div),
                                    //     vec![dst_reg, lhs_reg, rhs_reg],
                                    // )));
                                }
                                _ => {
                                    rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Div),
                                        vec![dst_reg, lhs_reg, rhs_reg],
                                    )));
                                }
                            }
                        }
                        BinOp::Rem => {
                            // x % y == x - (x / y) *y
                            // % 0 % 1 % 2^n 特殊判断
                            match rhs.as_ref().get_kind() {
                                InstKind::ConstInt(imm) | InstKind::GlobalConstInt(imm) => {
                                    match imm {
                                        0 => {
                                            lhs_reg = self
                                                .resolve_operand(func, lhs, true, map_info, pool);
                                            self.insts.push(pool.put_inst(LIRInst::new(
                                                InstrsType::OpReg(SingleOp::Mv),
                                                vec![dst_reg, lhs_reg],
                                            )));
                                        }
                                        1 | -1 => {
                                            self.insts.push(pool.put_inst(LIRInst::new(
                                                InstrsType::Binary(BinaryOp::Add),
                                                vec![
                                                    dst_reg.clone(),
                                                    dst_reg,
                                                    Operand::IImm(IImm::new(0)),
                                                ],
                                            )));
                                        }
                                        _ => {
                                            self.resolve_opt_rem(
                                                func, map_info, dst_reg, lhs, imm, pool,
                                            );
                                            // lhs_reg = self
                                            //     .resolve_operand(func, lhs, true, map_info, pool);
                                            // rhs_reg = self
                                            //     .resolve_operand(func, rhs, true, map_info, pool);
                                            // self.insts.push(pool.put_inst(LIRInst::new(
                                            //     InstrsType::Binary(BinaryOp::Rem),
                                            //     vec![dst_reg, lhs_reg, rhs_reg],
                                            // )));
                                        }
                                    }
                                }
                                _ => {
                                    assert!(
                                        lhs.as_ref().get_ir_type() == IrType::Int
                                            && rhs.as_ref().get_ir_type() == IrType::Int
                                    );
                                    lhs_reg = self.resolve_operand(func, lhs, true, map_info, pool);
                                    rhs_reg = self.resolve_operand(func, rhs, true, map_info, pool);
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
                                let fimm = self.resolve_fimm(-fimm, pool);
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Mv),
                                    vec![dst_reg, fimm],
                                )))
                            }
                            _ => match src.as_ref().get_ir_type() {
                                IrType::Int | IrType::Float => {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Neg),
                                        vec![dst_reg, src_reg],
                                    )))
                                }
                                _ => {
                                    unreachable!("invalid unary type for neg");
                                }
                            },
                        },
                        UnOp::Not => match src.as_ref().get_kind() {
                            InstKind::ConstInt(imm) => {
                                let iimm = match imm {
                                    0 => self.resolve_iimm(1, pool),
                                    _ => self.resolve_iimm(0, pool),
                                };
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Li),
                                    vec![dst_reg, iimm],
                                )));
                            }
                            InstKind::ConstFloat(fimm) => {
                                let fimm = if fimm == 0.0 {
                                    self.resolve_iimm(1, pool)
                                } else {
                                    self.resolve_iimm(0, pool)
                                };
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Li),
                                    vec![dst_reg, fimm],
                                )));
                            }
                            _ => match src.as_ref().get_ir_type() {
                                IrType::Int | IrType::Float => {
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
                        _ => unreachable!("invalid unary op"),
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
                            let index = addr.as_ref().get_gep_ptr();
                            let src_reg = self.resolve_operand(func, index, true, map_info, pool);
                            // if let Some(head) = map_info.array_slot_map.get(&index) {
                            //     src_reg = Operand::Reg(Reg::init(ScalarType::Int));
                            //     let mut load = LIRInst::new(
                            //         InstrsType::LoadParamFromStack,
                            //         vec![src_reg.clone(), Operand::IImm(IImm::new(*head))],
                            //     );
                            //     load.set_double();
                            //     self.insts.push(pool.put_inst(load));
                            // } else {
                            //     // 找不到，认为是全局数组或者参数，全局数组的访问是load -> gep -> load -> alloca
                            //     // src_reg = self.resolve_operand(func, index, true, map_info, pool);
                            // }
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
                                    let tmp1 = Operand::Reg(Reg::init(ScalarType::Int));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Shl),
                                        vec![
                                            tmp1.clone(),
                                            offset.clone(),
                                            Operand::IImm(IImm::new(2)),
                                        ],
                                    )));
                                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                                    let mut inst = LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Add),
                                        vec![tmp.clone(), src_reg.clone(), tmp1],
                                    );
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Load,
                                        vec![dst_reg.clone(), tmp, Operand::IImm(IImm::new(0))],
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
                                let src_reg = match *ga {
                                    Operand::Addr(..) => self.resolve_global(addr, map_info, pool),
                                    _ => ga.clone(),
                                };
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Mv),
                                    vec![dst_reg.clone(), src_reg],
                                )));
                            } else {
                                unreachable!("invalid alloca load");
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
                            let addr_reg = self.resolve_operand(
                                func,
                                addr.get_gep_ptr(),
                                true,
                                map_info,
                                pool,
                            );
                            match addr.get_gep_offset().get_kind() {
                                InstKind::ConstInt(offset) | InstKind::GlobalConstInt(offset) => {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Store,
                                        vec![
                                            value_reg,
                                            addr_reg.clone(),
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
                                    let tmp1 = Operand::Reg(Reg::init(ScalarType::Int));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Shl),
                                        vec![
                                            tmp1.clone(),
                                            temp.clone(),
                                            Operand::IImm(IImm::new(2)),
                                        ],
                                    )));
                                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                                    let mut inst = LIRInst::new(
                                        InstrsType::Binary(BinaryOp::Add),
                                        vec![tmp.clone(), addr_reg.clone(), tmp1],
                                    );
                                    // log!("addr_reg: {:?}, value_reg: {:?}", addr_reg, value_reg);
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Store,
                                        vec![value_reg, tmp, Operand::IImm(IImm::new(0))],
                                    )));
                                }
                            }
                        }
                        _ => {
                            let addr_reg = self.resolve_operand(func, addr, false, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Store,
                                vec![value_reg, addr_reg, Operand::IImm(IImm::new(0))],
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

                    //TODO: opt: 对于出现过的数组，从寄存器中取出
                    // 在开启ir优化后，所有的非main函数都是递归函数，需要在栈上分配空间
                    // 而对于main函数的所有数组都可以当做全局数组使用
                    if func.label == "main" {
                        let first;
                        match inst_ref.get_ir_type() {
                            IrType::IntPtr => {
                                let array: Vec<i32> =
                                    inst_ref.get_int_init().1.iter().map(|(_, x)| *x).collect();
                                let alloca =
                                    IntArray::new(label.clone(), size, true, array.clone());
                                first = func.as_mut().const_array.insert(alloca);
                            }
                            IrType::FloatPtr => {
                                let array: Vec<f32> = inst_ref
                                    .get_float_init()
                                    .1
                                    .iter()
                                    .map(|(_, x)| *x)
                                    .collect();
                                let alloca =
                                    FloatArray::new(label.clone(), size, true, array.clone());
                                first = func.as_mut().float_array.insert(alloca);
                            }
                            _ => unreachable!("invalid alloca type {:?}", inst_ref.get_ir_type()),
                        }

                        if first {
                            let dst_reg =
                                self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                            // let offset = pos;
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::OpReg(SingleOp::LoadAddr),
                                vec![dst_reg.clone(), Operand::Addr(label.clone())],
                            )));
                        }
                    } else {
                        // 遇到局部数组存到栈上，8字节对齐
                        let array_size = (size + 1) * NUM_SIZE / 8 * 8;
                        // 记录数组大小
                        func.as_mut().array_slot.push(array_size);
                        let dst_reg =
                            self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                        let mut add = LIRInst::new(
                            InstrsType::Binary(BinaryOp::Add),
                            vec![
                                dst_reg.clone(),
                                Operand::Reg(Reg::new(2, ScalarType::Int)),
                                Operand::IImm(IImm::new(array_size)),
                            ],
                        );
                        add.set_double();
                        let obj_add = pool.put_inst(add);
                        self.insts.push(obj_add);

                        func.as_mut().array_inst.push(obj_add);

                        let (is_init, array_info) = match inst_ref.get_ir_type() {
                            IrType::IntPtr => inst_ref.get_int_init().clone(),
                            IrType::FloatPtr => (
                                inst_ref.get_float_init().0,
                                inst_ref
                                    .get_float_init()
                                    .1
                                    .iter()
                                    .map(|(flag, x)| {
                                        (*flag, FImm::new(*x).to_string().parse::<i32>().unwrap())
                                    })
                                    .collect::<Vec<_>>()
                                    .clone(),
                            ),
                            _ => unreachable!("invalid alloca type {:?}", inst_ref.get_ir_type()),
                        };

                        // 若进行了初始化
                        if is_init {
                            // 若没有不确定变量，则memcopy
                            let do_memcopy = array_info.iter().any(|(flag, _)| !*flag)
                                && (array_info
                                    .iter()
                                    .map(|(_, value)| (*value).abs())
                                    .sum::<i32>()
                                    != 0);
                            // 若装填因子小于一半，或长度为0，则memset
                            // memset优先级高于memcopy
                            let do_memset = (array_info
                                .iter()
                                .filter(|(flag, value)| *flag || *value != 0)
                                .count()
                                <= (size as usize / 2))
                                || array_info.len() == 0;

                            if do_memcopy || do_memset {
                                let a0 = Reg::new(10, ScalarType::Int);
                                let a1 = Reg::new(11, ScalarType::Int);
                                let a2 = Reg::new(12, ScalarType::Int);

                                // let stack_addr = &func.as_ref().stack_addr;
                                // let last = stack_addr.front().unwrap();
                                // let pos = last.get_pos() + ADDR_SIZE * 3;

                                // let slot = StackSlot::new(pos, ADDR_SIZE);
                                // let mut set = Vec::new();
                                // func.as_mut().stack_addr.push_front(slot);
                                // // save a0
                                // let mut inst = LIRInst::new(
                                //     InstrsType::StoreParamToStack,
                                //     vec![
                                //         Operand::Reg(a0).clone(),
                                //         Operand::IImm(IImm::new(pos - 2 * ADDR_SIZE)),
                                //     ],
                                // );
                                // inst.set_double();
                                // set.push(pool.put_inst(inst));
                                // //save a1
                                // let mut inst = LIRInst::new(
                                //     InstrsType::StoreParamToStack,
                                //     vec![
                                //         Operand::Reg(a1).clone(),
                                //         Operand::IImm(IImm::new(pos - ADDR_SIZE)),
                                //     ],
                                // );
                                // inst.set_double();
                                // set.push(pool.put_inst(inst));
                                // // save a2
                                // let mut inst = LIRInst::new(
                                //     InstrsType::StoreParamToStack,
                                //     vec![Operand::Reg(a2).clone(), Operand::IImm(IImm::new(pos))],
                                // );
                                // inst.set_double();
                                // set.push(pool.put_inst(inst));

                                // self.push_back_list(&mut set);

                                // a0 = label in stack
                                self.insts.push(pool.put_inst(LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Mv),
                                    vec![Operand::Reg(a0), dst_reg.clone()],
                                )));
                                if do_memset {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Li),
                                        vec![Operand::Reg(a1), Operand::IImm(IImm::new(0))],
                                    )));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Li),
                                        vec![
                                            Operand::Reg(a2),
                                            Operand::IImm(IImm::new(size * NUM_SIZE)),
                                        ],
                                    )));
                                    let mut call_inst = LIRInst::new(
                                        InstrsType::Call,
                                        vec![Operand::Addr("memset@plt".to_string())],
                                    );
                                    call_inst.set_param_cnts(3, 0);
                                    self.insts.push(pool.put_inst(call_inst));
                                } else if do_memcopy {
                                    let alloca = IntArray::new(
                                        label.clone(),
                                        size,
                                        true,
                                        array_info
                                            .iter()
                                            .map(|(_, value)| *value)
                                            .collect::<Vec<_>>()
                                            .clone(),
                                    );
                                    // let offset = pos;
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::LoadAddr),
                                        vec![Operand::Reg(a1), Operand::Addr(label.clone())],
                                    )));
                                    func.as_mut().const_array.insert(alloca);
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Li),
                                        vec![
                                            Operand::Reg(a2),
                                            Operand::IImm(IImm::new(size * NUM_SIZE)),
                                        ],
                                    )));
                                    let mut call_inst = LIRInst::new(
                                        InstrsType::Call,
                                        vec![Operand::Addr("memcpy@plt".to_string())],
                                    );
                                    call_inst.set_param_cnts(3, 0);
                                    self.insts.push(pool.put_inst(call_inst));
                                }
                                // let mut set = Vec::new();
                                // let mut inst = LIRInst::new(
                                //     InstrsType::LoadParamFromStack,
                                //     vec![
                                //         Operand::Reg(a0).clone(),
                                //         Operand::IImm(IImm::new(pos - 2 * ADDR_SIZE)),
                                //     ],
                                // );
                                // inst.set_double();
                                // set.push(pool.put_inst(inst));

                                // let mut inst = LIRInst::new(
                                //     InstrsType::LoadParamFromStack,
                                //     vec![
                                //         Operand::Reg(a1).clone(),
                                //         Operand::IImm(IImm::new(pos - ADDR_SIZE)),
                                //     ],
                                // );
                                // inst.set_double();
                                // set.push(pool.put_inst(inst));

                                // let mut inst = LIRInst::new(
                                //     InstrsType::LoadParamFromStack,
                                //     vec![Operand::Reg(a2).clone(), Operand::IImm(IImm::new(pos))],
                                // );
                                // inst.set_double();
                                // set.push(pool.put_inst(inst));

                                // self.push_back_list(&mut set);
                            } else {
                                for i in (array_info.len() as i32)..size {
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Store,
                                        vec![
                                            Operand::Reg(Reg::new(0, ScalarType::Int)),
                                            dst_reg.clone(),
                                            Operand::IImm(IImm::new(i * NUM_SIZE)),
                                        ],
                                    )))
                                }
                            }
                        }
                    }

                    // let last = func.as_ref().stack_addr.front().unwrap();
                    // let pos = last.get_pos() + ADDR_SIZE;
                    // func.as_mut()
                    //     .stack_addr
                    //     .push_front(StackSlot::new(pos, ADDR_SIZE));
                    // if first {
                    //     let dst_reg =
                    //         self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    //     // let offset = pos;
                    //     self.insts.push(pool.put_inst(LIRInst::new(
                    //         InstrsType::OpReg(SingleOp::LoadAddr),
                    //         vec![dst_reg.clone(), Operand::Addr(label.clone())],
                    //     )));
                    // }

                    // let mut store = LIRInst::new(
                    //     InstrsType::StoreParamToStack,
                    //     vec![dst_reg.clone(), Operand::IImm(IImm::new(offset))],
                    // );
                    // store.set_double();
                    // self.insts.push(pool.put_inst(store));

                    // array: offset~offset+size(8字节对齐)
                    // map_key: array_name
                    // map_info.array_slot_map.insert(ir_block_inst, offset);
                }
                InstKind::Branch => {
                    // if jump
                    let mut inst = LIRInst::new(InstrsType::Jump, vec![]);
                    if inst_ref.is_br_jmp() {
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
                        }
                        jump_block.as_mut().in_edge.push(ObjPtr::new(self));
                        self.out_edge.push(*jump_block);
                        break;
                    }

                    // if branch
                    // 将前端的跳转视为不跳转，并优先执行，因此设为j的目标。故要对前端传来的条件取反
                    let cond_ref = inst_ref.get_br_cond();

                    let false_cond_bb = block.as_ref().get_next_bb()[0];
                    let true_cond_bb = block.as_ref().get_next_bb()[1];
                    let block_map = map_info.ir_block_map.clone();
                    let true_succ_block = match block_map.get(&true_cond_bb) {
                        Some(block) => block,
                        None => unreachable!("true block not found"),
                    };
                    let false_succ_block = match block_map.get(&false_cond_bb) {
                        Some(block) => block,
                        None => unreachable!("false block not found"),
                    };

                    match cond_ref.get_kind() {
                        InstKind::Binary(cond) => {
                            let lhs_reg;
                            let rhs_reg;
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
                                BinOp::Eq => InstrsType::Branch(CmpOp::Ne),
                                BinOp::Ne => InstrsType::Branch(CmpOp::Eq),
                                BinOp::Ge => InstrsType::Branch(CmpOp::Lt),
                                BinOp::Le => InstrsType::Branch(CmpOp::Gt),
                                BinOp::Gt => InstrsType::Branch(CmpOp::Le),
                                BinOp::Lt => InstrsType::Branch(CmpOp::Ge),
                                _ => {
                                    // 无法匹配，认为是if(a)情况，与0进行比较
                                    bz = true;
                                    InstrsType::Branch(CmpOp::Eqz)
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
                                let lhs_register = match lhs_reg {
                                    Operand::Reg(reg) => reg,
                                    _ => unreachable!(),
                                };
                                let rhs_register = match rhs_reg {
                                    Operand::Reg(reg) => reg,
                                    _ => unreachable!(),
                                };
                                assert!(lhs_register.get_type() == rhs_register.get_type());
                                if lhs_register.get_type() == ScalarType::Float {
                                    let dst_reg = Operand::Reg(Reg::init(ScalarType::Int));
                                    let kind = match cond {
                                        BinOp::Eq => InstrsType::Binary(BinaryOp::FCmp(CmpOp::Eq)),
                                        BinOp::Ne => InstrsType::Binary(BinaryOp::FCmp(CmpOp::Ne)),
                                        BinOp::Ge => InstrsType::Binary(BinaryOp::FCmp(CmpOp::Ge)),
                                        BinOp::Le => InstrsType::Binary(BinaryOp::FCmp(CmpOp::Le)),
                                        BinOp::Gt => InstrsType::Binary(BinaryOp::FCmp(CmpOp::Gt)),
                                        BinOp::Lt => InstrsType::Binary(BinaryOp::FCmp(CmpOp::Lt)),
                                        _ => unreachable!(),
                                    };
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        kind,
                                        vec![dst_reg.clone(), lhs_reg, rhs_reg],
                                    )));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::Branch(CmpOp::Eqz),
                                        vec![
                                            Operand::Addr(false_succ_block.label.to_string()),
                                            dst_reg,
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
                            }
                            self.push_back(pool.put_inst(LIRInst::new(
                                InstrsType::Jump,
                                vec![Operand::Addr(true_succ_block.label.to_string())],
                            )));

                            true_succ_block.as_mut().in_edge.push(ObjPtr::new(self));
                            false_succ_block.as_mut().in_edge.push(ObjPtr::new(self));
                            self.out_edge
                                .append(vec![*true_succ_block, *false_succ_block].as_mut());
                        }
                        _ => {
                            // log!("{:?}", cond_ref.get_kind());
                            let lhs_reg =
                                self.resolve_operand(func, cond_ref, true, map_info, pool);
                            let inst_kind = InstrsType::Branch(CmpOp::Eqz);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![Operand::Addr(false_succ_block.label.to_string()), lhs_reg],
                            )));
                            self.push_back(pool.put_inst(LIRInst::new(
                                InstrsType::Jump,
                                vec![Operand::Addr(true_succ_block.label.to_string())],
                            )));

                            true_succ_block.as_mut().in_edge.push(ObjPtr::new(self));
                            false_succ_block.as_mut().in_edge.push(ObjPtr::new(self));
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
                    let int_param_cnt = icnt;
                    let float_param_cnt = fcnt;
                    let mut final_args: Vec<_> = arg_list
                        .iter()
                        .filter(|arg| arg.get_ir_type() == IrType::Float)
                        .collect();
                    final_args.append(
                        &mut arg_list
                            .iter()
                            .filter(|arg| arg.get_ir_type() != IrType::Float)
                            .collect(),
                    );

                    for arg in final_args.iter().rev() {
                        match arg.as_ref().get_param_type() {
                            IrType::Int | IrType::IntPtr | IrType::FloatPtr => {
                                icnt -= 1;
                                if icnt >= ARG_REG_COUNT {
                                    let src_reg = match arg.get_param_type() {
                                        IrType::Int => {
                                            self.resolve_operand(func, **arg, true, map_info, pool)
                                        }
                                        IrType::IntPtr | IrType::FloatPtr => {
                                            let src_reg = self.resolve_operand(
                                                func,
                                                arg.get_gep_ptr(),
                                                true,
                                                map_info,
                                                pool,
                                            );
                                            let tmp = self.resolve_operand(
                                                func,
                                                arg.get_gep_offset(),
                                                true,
                                                map_info,
                                                pool,
                                            );
                                            let dst_reg = self
                                                .resolve_operand(func, **arg, true, map_info, pool);
                                            let tmp1 = Operand::Reg(Reg::init(ScalarType::Int));
                                            self.insts.push(pool.put_inst(LIRInst::new(
                                                InstrsType::Binary(BinaryOp::Shl),
                                                vec![
                                                    tmp1.clone(),
                                                    tmp.clone(),
                                                    Operand::IImm(IImm::new(2)),
                                                ],
                                            )));
                                            let mut add = LIRInst::new(
                                                InstrsType::Binary(BinaryOp::Add),
                                                vec![dst_reg.clone(), src_reg.clone(), tmp1],
                                            );
                                            add.set_double();
                                            self.insts.push(pool.put_inst(add));
                                            dst_reg
                                        }
                                        _ => unreachable!(),
                                    };
                                    // 最后一个溢出参数在最下方（最远离sp位置）
                                    let offset = Operand::IImm(IImm::new(
                                        -max(0, icnt - ARG_REG_COUNT) * ADDR_SIZE - ADDR_SIZE * 2,
                                    ));
                                    let mut inst = LIRInst::new(
                                        InstrsType::StoreToStack,
                                        vec![src_reg, offset],
                                    );
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                } else {
                                    // 保存在寄存器中的参数，从前往后
                                    let dst_reg =
                                        Operand::Reg(Reg::new(10 + icnt, ScalarType::Int));
                                    let src_reg = match arg.get_kind() {
                                        InstKind::Gep => {
                                            let src_reg = self.resolve_operand(
                                                func,
                                                arg.get_gep_ptr(),
                                                true,
                                                map_info,
                                                pool,
                                            );
                                            let tmp = self.resolve_operand(
                                                func,
                                                arg.get_gep_offset(),
                                                true,
                                                map_info,
                                                pool,
                                            );
                                            let dst_reg = self
                                                .resolve_operand(func, **arg, true, map_info, pool);
                                            let tmp1 = Operand::Reg(Reg::init(ScalarType::Int));
                                            self.insts.push(pool.put_inst(LIRInst::new(
                                                InstrsType::Binary(BinaryOp::Shl),
                                                vec![
                                                    tmp1.clone(),
                                                    tmp.clone(),
                                                    Operand::IImm(IImm::new(2)),
                                                ],
                                            )));
                                            let mut add = LIRInst::new(
                                                InstrsType::Binary(BinaryOp::Add),
                                                vec![dst_reg.clone(), src_reg.clone(), tmp1],
                                            );
                                            add.set_double();
                                            self.insts.push(pool.put_inst(add));
                                            dst_reg
                                        }
                                        _ => {
                                            self.resolve_operand(func, **arg, true, map_info, pool)
                                        }
                                    };

                                    // 避免覆盖
                                    // let stack_addr = &func.as_ref().stack_addr;
                                    // let last = stack_addr.front().unwrap();
                                    // let pos = last.get_pos() + ADDR_SIZE;

                                    // let slot = StackSlot::new(pos, ADDR_SIZE);
                                    // func.as_mut().stack_addr.push_front(slot);
                                    // func.as_mut()
                                    //     .spill_stack_map
                                    //     .insert(Reg::new(10 + icnt, ScalarType::Int), slot);
                                    // let mut inst = LIRInst::new(
                                    //     InstrsType::StoreParamToStack,
                                    //     vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))],
                                    // );
                                    // inst.set_double();
                                    // self.insts.push(pool.put_inst(inst));

                                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Mv),
                                        vec![tmp.clone(), src_reg],
                                    )));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Mv),
                                        vec![dst_reg.clone(), tmp],
                                    )));
                                }
                            }
                            IrType::Float => {
                                fcnt -= 1;
                                if fcnt >= ARG_REG_COUNT {
                                    let src_reg =
                                        self.resolve_operand(func, **arg, true, map_info, pool);
                                    // 最后一个溢出参数在最下方（最远离sp位置）
                                    let offset = Operand::IImm(IImm::new(
                                        -(max(0, int_param_cnt - ARG_REG_COUNT)
                                            + max(0, fcnt - ARG_REG_COUNT))
                                            * ADDR_SIZE
                                            - ADDR_SIZE * 2,
                                    ));
                                    let mut inst = LIRInst::new(
                                        InstrsType::StoreToStack,
                                        vec![src_reg, offset],
                                    );
                                    inst.set_double();
                                    self.insts.push(pool.put_inst(inst));
                                } else {
                                    // 保存在寄存器中的参数，从前往后
                                    let dst_reg = Operand::Reg(Reg::new(
                                        FLOAT_BASE + 10 + fcnt,
                                        ScalarType::Float,
                                    ));
                                    let src_reg = match arg.get_kind() {
                                        InstKind::Gep => {
                                            let src_reg = self.resolve_operand(
                                                func,
                                                arg.get_gep_ptr(),
                                                true,
                                                map_info,
                                                pool,
                                            );
                                            let tmp = self.resolve_operand(
                                                func,
                                                arg.get_gep_offset(),
                                                true,
                                                map_info,
                                                pool,
                                            );
                                            let dst_reg = self
                                                .resolve_operand(func, **arg, true, map_info, pool);
                                            let tmp1 = Operand::Reg(Reg::init(ScalarType::Int));
                                            self.insts.push(pool.put_inst(LIRInst::new(
                                                InstrsType::Binary(BinaryOp::Shl),
                                                vec![
                                                    tmp1.clone(),
                                                    tmp.clone(),
                                                    Operand::IImm(IImm::new(2)),
                                                ],
                                            )));
                                            let mut add = LIRInst::new(
                                                InstrsType::Binary(BinaryOp::Add),
                                                vec![dst_reg.clone(), src_reg.clone(), tmp1],
                                            );
                                            add.set_double();
                                            self.insts.push(pool.put_inst(add));
                                            dst_reg
                                        }
                                        _ => {
                                            self.resolve_operand(func, **arg, true, map_info, pool)
                                        }
                                    };

                                    // 避免覆盖
                                    // let stack_addr = &func.as_ref().stack_addr;
                                    // let last = stack_addr.front().unwrap();
                                    // let pos = last.get_pos() + ADDR_SIZE;

                                    // let slot = StackSlot::new(pos, ADDR_SIZE);
                                    // func.as_mut().stack_addr.push_front(slot);
                                    // func.as_mut().spill_stack_map.insert(
                                    //     Reg::new(FLOAT_BASE + 10 + fcnt, ScalarType::Float),
                                    //     slot,
                                    // );
                                    // let mut inst = LIRInst::new(
                                    //     InstrsType::StoreParamToStack,
                                    //     vec![dst_reg.clone(), Operand::IImm(IImm::new(pos))],
                                    // );
                                    // inst.set_double();
                                    // self.insts.push(pool.put_inst(inst));

                                    let tmp = Operand::Reg(Reg::init(ScalarType::Float));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Mv),
                                        vec![tmp.clone(), src_reg],
                                    )));
                                    self.insts.push(pool.put_inst(LIRInst::new(
                                        InstrsType::OpReg(SingleOp::Mv),
                                        vec![dst_reg.clone(), tmp],
                                    )));
                                }
                            }
                            _ => {
                                unreachable!("call arg type not match, either be int or float")
                            }
                        }
                    }
                    let mut lir_inst = LIRInst::new(
                        InstrsType::Call,
                        vec![Operand::Addr(func_label.to_string())],
                    );
                    lir_inst.set_param_cnts(int_param_cnt, float_param_cnt);
                    let call_type = if inst_ref.get_use_list().len() == 0 {
                        ScalarType::Void
                    } else {
                        match inst_ref.get_ir_type() {
                            IrType::Int => ScalarType::Int,
                            IrType::Float => ScalarType::Float,
                            IrType::Void => ScalarType::Void,
                            _ => unreachable!("call type not match"),
                        }
                    };

                    lir_inst.set_call_type(call_type);
                    self.insts.push(pool.put_inst(lir_inst));

                    match call_type {
                        ScalarType::Int => {
                            let dst_reg =
                                self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::OpReg(SingleOp::Mv),
                                vec![dst_reg, Operand::Reg(Reg::new(10, ScalarType::Int))],
                            )));
                        }
                        ScalarType::Float => {
                            let dst_reg =
                                self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                            self.insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::OpReg(SingleOp::Mv),
                                vec![
                                    dst_reg,
                                    Operand::Reg(Reg::new(FLOAT_BASE + 10, ScalarType::Float)),
                                ],
                            )));
                        }
                        ScalarType::Void => {}
                    }

                    // restore stack slot
                    // let mut i = 0;
                    // while i < ARG_REG_COUNT {
                    //     let iarg = Reg::new(i + 10, ScalarType::Int);
                    //     let farg = Reg::new(FLOAT_BASE + i + 10, ScalarType::Float);
                    //     if let Some(slot) = func.as_ref().spill_stack_map.get(&iarg) {
                    //         let mut inst = LIRInst::new(
                    //             InstrsType::LoadParamFromStack,
                    //             vec![
                    //                 Operand::Reg(Reg::new(i + 10, ScalarType::Int)),
                    //                 Operand::IImm(IImm::new(slot.get_pos())),
                    //             ],
                    //         );
                    //         inst.set_double();
                    //         self.insts.push(pool.put_inst(inst));
                    //     }
                    //     if let Some(slot) = func.as_ref().spill_stack_map.get(&farg) {
                    //         let mut inst = LIRInst::new(
                    //             InstrsType::LoadParamFromStack,
                    //             vec![
                    //                 Operand::Reg(Reg::new(FLOAT_BASE + i + 10, ScalarType::Float)),
                    //                 Operand::IImm(IImm::new(slot.get_pos())),
                    //             ],
                    //         );
                    //         inst.set_double();
                    //         self.insts.push(pool.put_inst(inst));
                    //     }
                    //     i += 1;
                    // }
                }
                InstKind::Return => match inst_ref.get_ir_type() {
                    IrType::Void => self.insts.push(
                        pool.put_inst(LIRInst::new(InstrsType::Ret(ScalarType::Void), vec![])),
                    ),
                    IrType::Int => {
                        let src = inst_ref.get_return_value();
                        let src_operand = self.resolve_operand(func, src, true, map_info, pool);
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Mv),
                            vec![Operand::Reg(Reg::new(10, ScalarType::Int)), src_operand],
                        )));

                        self.insts.push(
                            pool.put_inst(LIRInst::new(InstrsType::Ret(ScalarType::Int), vec![])),
                        );
                    }
                    IrType::Float => {
                        let src = inst_ref.get_return_value();
                        let src_reg = self.resolve_operand(func, src, true, map_info, pool);
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Mv),
                            vec![
                                Operand::Reg(Reg::new(FLOAT_BASE + 10, ScalarType::Float)),
                                src_reg,
                            ],
                        )));
                        self.insts.push(
                            pool.put_inst(LIRInst::new(InstrsType::Ret(ScalarType::Float), vec![])),
                        );
                    }
                    _ => panic!("cannot reach, Return false"),
                },
                InstKind::ItoF => {
                    let dst_reg = self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    let src_reg = self.resolve_operand(
                        func,
                        ir_block_inst.get_int_to_float_value(),
                        true,
                        map_info,
                        pool,
                    );
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::I2F),
                        vec![dst_reg, src_reg],
                    )));
                }
                InstKind::FtoI => {
                    let dst_reg = self.resolve_operand(func, ir_block_inst, true, map_info, pool);
                    let src_reg = self.resolve_operand(
                        func,
                        ir_block_inst.get_float_to_int_value(),
                        true,
                        map_info,
                        pool,
                    );
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::F2I),
                        vec![dst_reg, src_reg],
                    )));
                }

                InstKind::Phi => {
                    let phi_reg = self.resolve_operand(func, ir_block_inst, false, map_info, pool);
                    let kind;
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
                        ScalarType::Int => InstrsType::OpReg(SingleOp::Mv),
                        ScalarType::Float => InstrsType::OpReg(SingleOp::Mv),
                        _ => unreachable!("mv must be int or float"),
                    };

                    // log!("phi reg: {:?}, tmp: {:?}", phi_reg.clone(), temp.clone());
                    self.phis
                        .push(pool.put_inst(LIRInst::new(inst_kind, vec![phi_reg, temp.clone()])));

                    for (index, op) in ir_block_inst.get_operands().iter().enumerate() {
                        // log!("op: {:?}", op.get_kind());
                        let src_reg = match op.get_kind() {
                            InstKind::ConstInt(iimm) | InstKind::GlobalConstInt(iimm) => {
                                Operand::IImm(IImm::new(iimm))
                            }
                            InstKind::ConstFloat(fimm) | InstKind::GlobalConstFloat(fimm) => {
                                Operand::FImm(FImm::new(fimm))
                            }
                            _ => self.resolve_operand(func, *op, true, map_info, pool),
                        };
                        let mut is_float = false;
                        inst_kind = match src_reg {
                            Operand::Reg(reg) => match reg.get_type() {
                                ScalarType::Int => InstrsType::OpReg(SingleOp::Mv),
                                ScalarType::Float => InstrsType::OpReg(SingleOp::Mv),
                                _ => unreachable!("mv must be int or float"),
                            },
                            Operand::IImm(_) => InstrsType::OpReg(SingleOp::Li),
                            Operand::FImm(_) => {
                                is_float = true;
                                InstrsType::OpReg(SingleOp::Li)
                            }
                            _ => unreachable!("phi operand must be reg or imm"),
                        };
                        let mut insert_insts: Vec<ObjPtr<LIRInst>> = vec![];
                        if is_float {
                            let tmp2 = Operand::Reg(Reg::init(ScalarType::Int));
                            insert_insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::OpReg(SingleOp::LoadFImm),
                                vec![temp.clone(), tmp2.clone()],
                            )));
                            insert_insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![tmp2.clone(), src_reg.clone()],
                            )));
                        } else {
                            insert_insts.push(pool.put_inst(LIRInst::new(
                                inst_kind,
                                vec![temp.clone(), src_reg.clone()],
                            )));
                        }
                        // log!("phi kind {:?}", op.get_kind());

                        let incoming_block = map_info
                            .ir_block_map
                            .get(&ir_block_inst.get_phi_predecessor(index))
                            .unwrap()
                            .label
                            .clone();

                        if let Some(insts) = map_info.phis_to_block.get_mut(&incoming_block) {
                            // log!("insert phi inst: {:?}", obj_inst);
                            for obj_inst in insert_insts {
                                insts.push(obj_inst);
                            }
                        } else {
                            // log!("insert phi inst: {:?}", obj_inst);
                            let mut set = Vec::new();
                            for obj_inst in insert_insts {
                                set.push(obj_inst);
                            }
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

    pub fn get_prev(&self) -> &Vec<ObjPtr<BB>> {
        &self.in_edge
    }

    pub fn get_after(&self) -> &Vec<ObjPtr<BB>> {
        &self.out_edge
    }

    pub fn get_last_not_tail_inst(&self) -> ObjPtr<LIRInst> {
        self.insts[self.insts.len() - 2]
    }

    pub fn get_tail_inst(&self) -> ObjPtr<LIRInst> {
        assert!(self.insts.last() != None);
        self.insts[self.insts.len() - 1]
    }

    // ///粗糙分析需要保存的caller save寄存器的信息,并且进行前后的保存和恢复的指令插入
    // pub fn save_reg(&mut self, func: ObjPtr<Func>, pool: &mut BackendPool) {
    //     let mut index = 0;
    //     loop {
    //         if index >= self.insts.len() {
    //             break;
    //         }
    //         let inst = self.insts[index];
    //         for op in inst.operands.iter() {
    //             match op {
    //                 Operand::Reg(reg) => {
    //                     //FIXME: solve float regs
    //                     if let Some(phy_id) = func.reg_alloc_info.dstr.get(&reg.get_id()) {
    //                         let save_reg = Reg::new(*phy_id, reg.get_type());
    //                         if save_reg.is_caller_save() {
    //                             assert!(save_reg.get_type() == reg.get_type());
    //                             func.as_mut().caller_saved.insert(save_reg, *reg);
    //                         }
    //                         if save_reg.is_callee_save() {
    //                             func.as_mut().callee_saved.insert(save_reg);
    //                         }
    //                     } else {
    //                     }
    //                 }
    //                 _ => {}
    //             }
    //         }
    //         let mut caller_regs: HashSet<Reg> = HashSet::new();
    //         match inst.get_type() {
    //             InstrsType::Call => {
    //                 let (mut icnt, mut fcnt) = inst.get_param_cnts();
    //                 icnt = min(icnt, ARG_REG_COUNT);
    //                 fcnt = min(fcnt, ARG_REG_COUNT);
    //                 let mut caller_reg_cnts = func.caller_saved.len() as i32;
    //                 for (op, reg) in func.caller_saved.iter() {
    //                     if inst.get_reg_def().len() != 0
    //                         && op.get_id() == inst.get_reg_def()[0].get_id()
    //                     {
    //                         caller_reg_cnts -= 1;
    //                         continue;
    //                     }
    //                     if op.get_type() == ScalarType::Int
    //                         && op.get_id() - 10 < icnt
    //                         && op.get_id() >= 10
    //                     {
    //                         caller_reg_cnts -= 1;
    //                         continue;
    //                     }
    //                     if op.get_type() == ScalarType::Float
    //                         && (op.get_id() - 10 - FLOAT_BASE < fcnt
    //                             && op.get_id() >= 10 + FLOAT_BASE)
    //                     {
    //                         caller_reg_cnts -= 1;
    //                         continue;
    //                     }
    //                     caller_regs.insert(*reg);
    //                 }

    //                 func.as_mut().caller_saved_len = max(func.caller_saved_len, caller_reg_cnts);
    //                 let mut pos = func.stack_addr.back().unwrap().get_pos();
    //                 pos += func.stack_addr.back().unwrap().get_size();
    //                 for (i, reg) in caller_regs.iter().enumerate() {
    //                     config::record_caller_save_sl(
    //                         &self.func_label,
    //                         &self.label,
    //                         format!("sd {}", reg).as_str(),
    //                     );

    //                     let offset = pos + i as i32 * ADDR_SIZE;
    //                     let mut ins = LIRInst::new(
    //                         InstrsType::StoreToStack,
    //                         vec![Operand::Reg(*reg), Operand::IImm(IImm::new(offset))],
    //                     );
    //                     ins.set_double();
    //                     self.insts.insert(index, pool.put_inst(ins));
    //                     index += 1;
    //                 }
    //                 index += 1;
    //                 for (i, reg) in caller_regs.iter().enumerate() {
    //                     config::record_caller_save_sl(
    //                         &self.func_label,
    //                         &self.label,
    //                         format!("ld {}", reg).as_str(),
    //                     );
    //                     let offset = pos + i as i32 * ADDR_SIZE;
    //                     let mut ins = LIRInst::new(
    //                         InstrsType::LoadFromStack,
    //                         vec![Operand::Reg(*reg), Operand::IImm(IImm::new(offset))],
    //                     );
    //                     ins.set_double();
    //                     self.insts.insert(index, pool.put_inst(ins));
    //                     index += 1;
    //                 }
    //             }
    //             _ => {
    //                 index += 1;
    //             }
    //         }
    //     }
    // }

    ///使用t0-t3以及 栈操作 来处理之前spill了的没有分配到专有物理寄存器的寄存器的使用
    pub fn handle_spill(
        &mut self,
        func: ObjPtr<Func>,
        spill: &HashSet<i32>,
        pool: &mut BackendPool,
    ) {
        let mut index = 0;
        loop {
            if index >= self.insts.len() {
                break;
            }
            let inst = self.insts[index];
            let spills = inst.is_spill(spill);
            if spills.is_empty() {
                index += 1;
                continue;
            } else {
                for (i, r) in spills.iter().enumerate() {
                    let reg = match r.get_type() {
                        ScalarType::Int => Operand::Reg(Reg::new(5 + (i as i32), ScalarType::Int)),
                        ScalarType::Float => {
                            Operand::Reg(Reg::new(18 + FLOAT_BASE + (i as i32), ScalarType::Float))
                        }
                        _ => unreachable!(),
                    };
                    if let Some(stack_slot) = func.spill_stack_map.get(&r) {
                        let mut ins = LIRInst::new(
                            InstrsType::LoadFromStack,
                            vec![reg, Operand::IImm(IImm::new(stack_slot.get_pos()))],
                        );
                        ins.set_double();
                        config::record_spill(&self.func_label, &self.label, &ins.to_string());
                        self.insts.insert(index, pool.put_inst(ins));
                        index += 1;
                    } else {
                        let last_slot = func.stack_addr.back().unwrap();
                        let pos = last_slot.get_pos() + last_slot.get_size();
                        let stack_slot = StackSlot::new(pos, ADDR_SIZE);
                        func.as_mut().stack_addr.push_back(stack_slot);
                        func.as_mut().spill_stack_map.insert(*r, stack_slot);
                    }
                }
                for (i, r) in spills.iter().enumerate() {
                    match r.get_type() {
                        ScalarType::Int => inst.as_mut().replace(r.get_id(), 5 + (i as i32)),
                        ScalarType::Float => inst
                            .as_mut()
                            .replace(r.get_id(), 18 + FLOAT_BASE + (i as i32)),
                        _ => unreachable!(),
                    }
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

                for (i, r) in spills.iter().enumerate() {
                    let reg = match r.get_type() {
                        ScalarType::Int => Operand::Reg(Reg::new(5 + (i as i32), ScalarType::Int)),
                        ScalarType::Float => {
                            Operand::Reg(Reg::new(18 + FLOAT_BASE + (i as i32), ScalarType::Float))
                        }
                        ScalarType::Void => unreachable!(),
                    };
                    let stack_slot = func.spill_stack_map.get(&r).unwrap();
                    match self.insts[index - 1].get_dst() {
                        Operand::Reg(ireg) => {
                            if (ireg.get_type() == ScalarType::Int
                                && ireg.get_id() != 5 + (i as i32))
                                || (ireg.get_type() == ScalarType::Float
                                    && ireg.get_id() != 18 + FLOAT_BASE + (i as i32))
                            {
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
                    config::record_spill(&self.func_label, &self.label, &ins.to_string());
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
        // log!("{}, len: {}", self.label, self.insts.len());
        loop {
            if pos >= self.insts.len() {
                break;
            }
            let inst_ref = self.insts[pos].as_ref();
            match inst_ref.get_type() {
                InstrsType::Load | InstrsType::Store => {
                    let temp = Operand::Reg(Reg::new(8, ScalarType::Int));
                    let offset = inst_ref.get_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        pos += 1;
                        continue;
                    }
                    // log!("over offset: {}", offset);
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);

                    // load的dst是reg，lhs是src_addr
                    // store的dst是addr，lhs是val
                    match inst_ref.get_type() {
                        InstrsType::Load => {
                            let mut inst = LIRInst::new(
                                InstrsType::Binary(BinaryOp::Add),
                                vec![temp.clone(), temp.clone(), inst_ref.get_lhs().clone()],
                            );
                            inst.set_double();
                            self.insts.insert(pos, pool.put_inst(inst));
                            pos += 1;
                            self.insts[pos].as_mut().replace_op(vec![
                                inst_ref.get_dst().clone(),
                                temp,
                                Operand::IImm(IImm::new(0)),
                            ]);
                        }
                        InstrsType::Store => {
                            let mut inst = LIRInst::new(
                                InstrsType::Binary(BinaryOp::Add),
                                vec![temp.clone(), temp.clone(), inst_ref.get_lhs().clone()],
                            );
                            inst.set_double();
                            self.insts.insert(pos, pool.put_inst(inst));
                            pos += 1;
                            self.insts[pos].as_mut().replace_op(vec![
                                inst_ref.get_dst().clone(),
                                temp,
                                Operand::IImm(IImm::new(0)),
                            ]);
                        }
                        _ => {
                            unreachable!("no more case")
                        }
                    }
                }

                InstrsType::LoadFromStack | InstrsType::StoreToStack => {
                    let temp = Operand::Reg(Reg::new(8, ScalarType::Int));
                    let offset = inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        pos += 1;
                        continue;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    let mut inst: LIRInst = LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![
                            temp.clone(),
                            temp.clone(),
                            Operand::Reg(Reg::new(2, ScalarType::Int)),
                        ],
                    );
                    inst.set_double();
                    self.insts.insert(pos, pool.put_inst(inst));
                    pos += 1;
                    match inst_ref.get_type() {
                        InstrsType::LoadFromStack => {
                            self.insts[pos].as_mut().replace_kind(InstrsType::Load);
                        }
                        InstrsType::StoreToStack => {
                            self.insts[pos].as_mut().replace_kind(InstrsType::Store);
                        }
                        _ => unreachable!(),
                    }
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }

                InstrsType::LoadParamFromStack | InstrsType::StoreParamToStack => {
                    let temp = Operand::Reg(Reg::new(8, ScalarType::Int));
                    let offset =
                        func.context.get_offset() as i32 - inst_ref.get_stack_offset().get_data();
                    if operand::is_imm_12bs(offset) {
                        pos += 1;
                        continue;
                    }
                    self.resolve_overflow_sl(temp.clone(), &mut pos, offset, pool);
                    let mut inst = LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![
                            temp.clone(),
                            temp.clone(),
                            Operand::Reg(Reg::new(2, ScalarType::Int)),
                        ],
                    );
                    inst.set_double();
                    self.insts.insert(pos, pool.put_inst(inst));
                    pos += 1;
                    match inst_ref.get_type() {
                        InstrsType::LoadParamFromStack => {
                            self.insts[pos].as_mut().replace_kind(InstrsType::Load);
                        }
                        InstrsType::StoreParamToStack => {
                            self.insts[pos].as_mut().replace_kind(InstrsType::Store);
                        }
                        _ => unreachable!(),
                    }
                    self.insts[pos].as_mut().replace_op(vec![
                        inst_ref.get_dst().clone(),
                        temp,
                        Operand::IImm(IImm::new(0)),
                    ]);
                }

                InstrsType::Branch(..) => {
                    // deal with false branch
                    // let is_j = match inst_ref.get_type() {
                    //     InstrsType::Branch(..) => false,
                    //     InstrsType::Jump => true,
                    //     _ => unreachable!(),
                    // };
                    let target = match inst_ref.get_label() {
                        Operand::Addr(label) => label,
                        _ => unreachable!("branch must have a label"),
                    };

                    let mut i = 0;
                    let (mut start, mut end) = (0, 0);

                    loop {
                        if i >= func.as_ref().blocks.len() {
                            break;
                        }
                        let block_ref = func.as_ref().blocks[i];
                        if block_ref.label == self.label {
                            start = i;
                        }
                        if block_ref.label == *target {
                            end = i;
                        }
                        if start != 0 && end != 0 {
                            break;
                        }
                        i += 1;
                    }
                    let mut distance = 0;
                    if start == end {
                        pos += 1;
                        continue;
                    }

                    let (st, ed) = (min(start, end), max(start, end));
                    let rev = start > end;
                    for i in st + 1..=ed - 1 {
                        let index = if rev { ed - i + st } else { i };
                        let block = func.blocks[index];
                        distance += block.insts.len() as i32 * ADDR_SIZE;
                        if !operand::is_imm_12bs(distance) {
                            let name = format!("overflow_{}", get_tmp_bb());
                            // TODO 检查overflow
                            config::record_branch_overflow(&self.func_label, &self.label, &name);
                            let tmp = pool.put_block(BB::new(&name, &func.label));
                            tmp.as_mut().insts.push(pool.put_inst(LIRInst::new(
                                InstrsType::Jump,
                                vec![Operand::Addr(target.clone())],
                            )));
                            func.as_mut().blocks.insert(index, tmp);
                            //FIXME: 最多长跳转一次，或可考虑收敛算法
                            self.insts[pos].as_mut().replace_label(name);
                            break;
                        }
                    }
                }

                InstrsType::Call => {
                    // call 指令不会发生偏移量的溢出
                }
                InstrsType::Jump => {
                    // j 型指令认为不会overflow
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
        // let lower = (offset << 20) >> 20;
        // let upper = (offset - lower) >> 12;
        // //取得高20位
        // let op1 = Operand::IImm(IImm::new(upper));
        // //取得低12位
        // let op2 = Operand::IImm(IImm::new(lower));
        config::record_offset_overflow(&self.func_label, &self.label, &pos.to_string());
        self.insts.insert(
            *pos,
            pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Li),
                vec![temp.clone(), Operand::IImm(IImm::new(offset))],
            )),
        );
        // *pos += 1;
        // self.insts.insert(
        //     *pos,
        //     pool.put_inst(LIRInst::new(
        //         InstrsType::Binary(BinaryOp::Add),
        //         vec![temp.clone(), temp.clone(), op2],
        //     )),
        // );
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
                InstKind::ConstInt(iimm) | InstKind::GlobalConstInt(iimm) => {
                    return self.load_iimm_to_ireg(iimm, pool)
                }
                _ => {}
            }
        }

        match src.as_ref().get_kind() {
            InstKind::ConstInt(iimm) | InstKind::GlobalConstInt(iimm) => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                self.resolve_iimm(iimm, pool)
            }
            InstKind::ConstFloat(fimm) | InstKind::GlobalConstFloat(fimm) => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                self.resolve_fimm(fimm, pool)
            }
            InstKind::Parameter => self.resolve_param(src, func, map, pool),
            InstKind::GlobalInt(..) | InstKind::GlobalFloat(..) => {
                self.resolve_global(src, map, pool)
            }
            _ => {
                if map.val_map.contains_key(&src) {
                    return map.val_map.get(&src).unwrap().clone();
                }
                let op: Operand = match src.as_ref().get_ir_type() {
                    IrType::Int | IrType::IntPtr | IrType::FloatPtr => {
                        Operand::Reg(Reg::init(ScalarType::Int))
                    }
                    IrType::Float => Operand::Reg(Reg::init(ScalarType::Float)),
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

    fn resolve_fimm(&mut self, imm: f32, pool: &mut BackendPool) -> Operand {
        let reg = Operand::Reg(Reg::init(ScalarType::Float));
        let tmp = Operand::Reg(Reg::init(ScalarType::Int));
        let fimm = Operand::FImm(FImm::new(imm));
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::OpReg(SingleOp::Li),
            vec![tmp.clone(), fimm],
        )));
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::OpReg(SingleOp::LoadFImm),
            vec![reg.clone(), tmp],
        )));
        reg
    }

    fn load_iimm_to_ireg(&mut self, imm: i32, pool: &mut BackendPool) -> Operand {
        let reg = Operand::Reg(Reg::init(ScalarType::Int));
        let iimm = Operand::IImm(IImm::new(imm));
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::OpReg(SingleOp::Li),
            vec![reg.clone(), iimm],
        )));
        // let lower = (imm << 20) >> 20;
        // let upper = (imm - lower) >> 12;
        // //取得高20位
        // let op1 = Operand::IImm(IImm::new(upper));
        // //取得低12位
        // let op2 = Operand::IImm(IImm::new(lower));
        // self.insts.push(pool.put_inst(LIRInst::new(
        //     InstrsType::OpReg(SingleOp::Lui),
        //     vec![reg.clone(), op1],
        // )));
        // log!("op2: {:?}", op2);
        // self.insts.push(pool.put_inst(LIRInst::new(
        //     InstrsType::Binary(BinaryOp::Add),
        //     vec![reg.clone(), reg.clone(), op2],
        // )));
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
            // 目前将a0-a7用作保留寄存器，不参与寄存器分配
            for p in params {
                match p.as_ref().get_param_type() {
                    IrType::Int | IrType::IntPtr | IrType::FloatPtr => {
                        if src == *p {
                            if inum < ARG_REG_COUNT {
                                let inst = LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Mv),
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
                                let mut inst = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![
                                        reg.clone(),
                                        Operand::IImm(IImm::new(
                                            ((inum - ARG_REG_COUNT + max(fnum - ARG_REG_COUNT, 0))
                                                + 1)
                                                * ADDR_SIZE,
                                        )),
                                    ],
                                );
                                inst.set_double();
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));
                            }
                        }
                        inum += 1;
                    }
                    IrType::Float => {
                        if src == *p {
                            if fnum < ARG_REG_COUNT {
                                let inst = LIRInst::new(
                                    InstrsType::OpReg(SingleOp::Mv),
                                    vec![
                                        reg.clone(),
                                        Operand::Reg(Reg::new(
                                            FLOAT_BASE + fnum + 10,
                                            ScalarType::Float,
                                        )),
                                    ],
                                );
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));
                            } else {
                                let mut inst = LIRInst::new(
                                    InstrsType::LoadParamFromStack,
                                    vec![
                                        reg.clone(),
                                        Operand::IImm(IImm::new(
                                            ((fnum - ARG_REG_COUNT + max(inum - ARG_REG_COUNT, 0))
                                                + 1)
                                                * ADDR_SIZE,
                                        )),
                                    ],
                                );
                                inst.set_double();
                                func.as_mut()
                                    .get_first_block()
                                    .as_mut()
                                    .insts
                                    .insert(0, pool.put_inst(inst));
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
            let reg = Operand::Reg(Reg::init(ScalarType::Int));
            self.global_map.insert(src, reg.clone());

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
        let mut lhs_reg;
        let mut rhs_reg;
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
                let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Slt),
                    vec![tmp.clone(), rhs_reg, lhs_reg],
                )));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Xor),
                    vec![dst_reg.clone(), tmp, Operand::IImm(IImm::new(1))],
                )));
            }
            BinOp::Ge => {
                // a >= b 变为 !(a < b)
                if is_limm {
                    lhs_reg = self.resolve_operand(func, lhs, true, map, pool);
                }
                let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Slt),
                    vec![tmp.clone(), lhs_reg, rhs_reg],
                )));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Xor),
                    vec![dst_reg.clone(), tmp.clone(), Operand::IImm(IImm::new(1))],
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
                    InstrsType::OpReg(SingleOp::Mv),
                    vec![dst, Operand::IImm(IImm::new(0))],
                )));
            }
            1 => {
                if !is_neg {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Mv),
                        vec![dst, src],
                    )));
                } else {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Neg),
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
                            InstrsType::OpReg(SingleOp::Neg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else if is_opt_num(abs - 1) {
                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![
                            tmp.clone(),
                            src.clone(),
                            Operand::IImm(IImm::new(log2(abs - 1))),
                        ],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![dst.clone(), tmp.clone(), src],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Neg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else if is_opt_num(abs + 1) {
                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shl),
                        vec![
                            tmp.clone(),
                            src.clone(),
                            Operand::IImm(IImm::new(log2(abs + 1))),
                        ],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sub),
                        vec![dst.clone(), tmp.clone(), src],
                    )));
                    if is_neg {
                        self.insts.push(pool.put_inst(LIRInst::new(
                            InstrsType::OpReg(SingleOp::Neg),
                            vec![dst.clone(), dst],
                        )))
                    }
                } else {
                    self.find_opt_mul(imm, dst, src, pool);
                }
            }
        }
    }

    fn find_opt_mul(&mut self, imm: i32, dst: Operand, src: Operand, pool: &mut BackendPool) {
        let abs = imm.abs();
        let is_neg = imm < 0;
        let (mut power, mut opt_abs, mut do_add, mut can_opt) = (0, 0, false, false);
        while (1 << power) <= abs && ((abs as u32 + (1 << power)) <= 2147483647) {
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
            true => InstrsType::Binary(BinaryOp::Sub),
            false => InstrsType::Binary(BinaryOp::Add),
        };
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::Binary(BinaryOp::Shl),
            vec![temp.clone(), src.clone(), Operand::IImm(IImm::new(power))],
        )));
        let tmp2 = Operand::Reg(Reg::init(ScalarType::Int));
        self.insts.push(pool.put_inst(LIRInst::new(
            InstrsType::Binary(BinaryOp::Shl),
            vec![tmp2.clone(), src.clone(), Operand::IImm(IImm::new(bits))],
        )));
        self.insts.push(pool.put_inst(LIRInst::new(
            combine_inst_kind,
            vec![dst.clone(), tmp2.clone(), temp],
        )));
        if is_neg {
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::OpReg(SingleOp::Neg),
                vec![dst.clone(), dst],
            )))
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
                        InstrsType::OpReg(SingleOp::Neg),
                        vec![dst, src],
                    )))
                } else {
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Mv),
                        vec![dst, src],
                    )))
                }
            }
            _ => {
                if is_opt_num(abs) {
                    let bits = log2(abs);
                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sar),
                        vec![tmp.clone(), src.clone(), Operand::IImm(IImm::new(bits - 1))],
                    )));
                    let tmp2 = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shr),
                        vec![
                            tmp2.clone(),
                            tmp.clone(),
                            Operand::IImm(IImm::new(32 - bits)),
                        ],
                    )));
                    let tmp3 = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![tmp3.clone(), tmp2.clone(), src.clone()],
                    )));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sar),
                        vec![dst, tmp3, Operand::IImm(IImm::new(log2(abs)))],
                    )))
                } else {
                    let (magic, shift) = get_magic(is_neg, abs);
                    let tmp = Operand::Reg(Reg::init(ScalarType::Int));
                    // load magic number M
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::OpReg(SingleOp::Li),
                        vec![tmp.clone(), Operand::Addr(magic.to_string())],
                    )));
                    // q = floor(M * an / 2^32)
                    let tmp2 = Operand::Reg(Reg::init(ScalarType::Int));
                    let mut inst = LIRInst::new(
                        InstrsType::Binary(BinaryOp::Mul),
                        vec![tmp2.clone(), tmp.clone(), src.clone()],
                    );
                    inst.set_double();
                    self.insts.push(pool.put_inst(inst));

                    let tmp3 = Operand::Reg(Reg::init(ScalarType::Int));
                    let mut inst = LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shr),
                        vec![tmp3.clone(), tmp2.clone(), Operand::IImm(IImm::new(32))],
                    );
                    inst.set_double();
                    self.insts.push(pool.put_inst(inst));
                    // q = q >> s
                    // shrsi q, q, s
                    let tmp4 = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Sar),
                        vec![tmp4.clone(), tmp3.clone(), Operand::IImm(IImm::new(shift))],
                    )));
                    // add 1 to q if n is neg
                    // shri t, n, 31
                    let tmp5 = Operand::Reg(Reg::init(ScalarType::Int));
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Shr),
                        vec![tmp5.clone(), src.clone(), Operand::IImm(IImm::new(31))],
                    )));
                    // add q, q, t
                    self.insts.push(pool.put_inst(LIRInst::new(
                        InstrsType::Binary(BinaryOp::Add),
                        vec![dst.clone(), tmp5.clone(), tmp4.clone()],
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
        // let is_neg = imm < 0;
        if is_opt_num(abs) {
            let k = log2(abs);
            // r = ((n + t) & (2^k - 1)) - t
            // t = (n >> k - 1) >> 32 - k
            let tmp = Operand::Reg(Reg::init(ScalarType::Int));
            if k == 1 {
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Shr),
                    vec![
                        tmp.clone(),
                        lhs_reg.clone(),
                        Operand::IImm(IImm::new(32 - k)),
                    ],
                )));
            } else {
                let tmp2 = Operand::Reg(Reg::init(ScalarType::Int));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Sar),
                    vec![
                        tmp2.clone(),
                        lhs_reg.clone(),
                        Operand::IImm(IImm::new(k - 1)),
                    ],
                )));
                self.insts.push(pool.put_inst(LIRInst::new(
                    InstrsType::Binary(BinaryOp::Shr),
                    vec![tmp.clone(), tmp2.clone(), Operand::IImm(IImm::new(32 - k))],
                )));
            }
            let tmp3 = Operand::Reg(Reg::init(ScalarType::Int));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Add),
                vec![tmp3.clone(), lhs_reg.clone(), tmp.clone()],
            )));
            let abs_reg = self.resolve_iimm(abs - 1, pool);
            let tmp4 = Operand::Reg(Reg::init(ScalarType::Int));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::And),
                vec![tmp4.clone(), tmp3.clone(), abs_reg],
            )));
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Sub),
                vec![dst.clone(), tmp4.clone(), tmp.clone()],
            )));
        } else {
            let tmp1 = Operand::Reg(Reg::init(ScalarType::Int));
            let tmp2 = Operand::Reg(Reg::init(ScalarType::Int));
            self.resolve_opt_div(tmp1.clone(), lhs_reg.clone(), imm, pool);
            self.resolve_opt_mul(tmp2.clone(), tmp1, imm, pool);
            self.insts.push(pool.put_inst(LIRInst::new(
                InstrsType::Binary(BinaryOp::Sub),
                vec![dst, lhs_reg, tmp2],
            )));
        }
    }

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
        // log!("generate bb:{}", self.label);
        for inst in self.insts.iter() {
            // log!("generate inst:{:?}", inst);
            inst.as_mut().generate(context.clone(), f)?;
        }
        Ok(())
    }
}

fn is_opt_num(imm: i32) -> bool {
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

fn get_magic(is_neg: bool, abs: i32) -> (i64, i32) {
    let (two31, uabs, mut p) = (1 << 31 as u32, abs as u32, 31);
    let mut delta;
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
        if !(q1 < delta || (q1 == delta && r1 == 0)) {
            break;
        }
    }
    let mut magic = (q2 + 1) as i64;
    if is_neg {
        magic = -magic;
    }
    let shift = p - 32;
    (magic, shift)
}

///for build v2:
/// 1.handle spill v2
impl BB {
    ///使用block_handle_spill_v2之前 应该先已经给spillings的寄存器分配了对应的空间了
    /// 保底有三个寄存器t1,t2,t3能够使用 (x5,x6,x7)
    /// t1,t2,t3优先分配给之前使用它们的寄存器
    pub fn handle_spill_v2(
        &mut self,
        func: ObjPtr<Func>,
        spill: &HashSet<i32>,
        pool: &mut BackendPool,
    ) {
        //不维护上下文
        let mut rents: HashMap<Reg, Option<Reg>> = HashMap::new();
        let mut new_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
        for reg in 5..=7 {
            rents.insert(Reg::new(reg, ScalarType::Int), None);
        }
        let mut index = 0;
        loop {
            if index >= self.insts.len() {
                break;
            }
            let inst = self.insts[index];
            let spills = inst.is_spill(spill);
            if spills.is_empty() {
                new_insts.push(inst);
                index += 1;
                continue;
            } else {
                for (i, r) in spills.iter().enumerate() {
                    let reg = match r.get_type() {
                        ScalarType::Int => Operand::Reg(Reg::new(5 + (i as i32), ScalarType::Int)),
                        ScalarType::Float => {
                            Operand::Reg(Reg::new(18 + FLOAT_BASE + (i as i32), ScalarType::Float))
                        }
                        _ => unreachable!(),
                    };
                    log!("{}", r);
                    let stack_slot = func.spill_stack_map.get(r).unwrap();
                    let mut ins = LIRInst::new(
                        InstrsType::LoadFromStack,
                        vec![reg, Operand::IImm(IImm::new(stack_slot.get_pos()))],
                    );
                    ins.set_double();
                    config::record_spill(&self.func_label, &self.label, &ins.to_string());
                    new_insts.push(pool.put_inst(ins));
                }
                for (i, r) in spills.iter().enumerate() {
                    match r.get_type() {
                        ScalarType::Int => inst.as_mut().replace(r.get_id(), 5 + (i as i32)),
                        ScalarType::Float => inst
                            .as_mut()
                            .replace(r.get_id(), 18 + FLOAT_BASE + (i as i32)),
                        _ => unreachable!(),
                    }
                }
                new_insts.push(inst);
                index += 1; //更新到下一条指令

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

                for (i, r) in spills.iter().enumerate() {
                    let reg = match r.get_type() {
                        ScalarType::Int => Operand::Reg(Reg::new(5 + (i as i32), ScalarType::Int)),
                        ScalarType::Float => {
                            Operand::Reg(Reg::new(18 + FLOAT_BASE + (i as i32), ScalarType::Float))
                        }
                        ScalarType::Void => unreachable!(),
                    };
                    match inst.get_dst() {
                        Operand::Reg(ireg) => {
                            if (ireg.get_type() == ScalarType::Int
                                && ireg.get_id() != 5 + (i as i32))
                                || (ireg.get_type() == ScalarType::Float
                                    && ireg.get_id() != 18 + FLOAT_BASE + (i as i32))
                            {
                                continue;
                            }
                        }
                        _ => {}
                    }
                    let stack_slot = func.spill_stack_map.get(&r).unwrap();
                    let mut ins = LIRInst::new(
                        InstrsType::StoreToStack,
                        vec![reg, Operand::IImm(IImm::new(stack_slot.get_pos()))],
                    );
                    ins.set_double();
                    config::record_spill(&self.func_label, &self.label, &ins.to_string());
                    new_insts.push(pool.put_inst(ins));
                }
            }
        }
        self.insts = new_insts;
    }
}

//for build v3
impl BB {
    ///为handle spill进行calc live

    pub fn handle_spill_v3(&mut self, func: ObjPtr<Func>, pool: &mut BackendPool) {
        //维护一个表,表里面记录了能够使用的寄存器
        // todo!();
        self.handle_spill_v2(func, &func.reg_alloc_info.spillings, pool);
        return;

        //
    }
}

/// build reg intervals
impl BB {
    pub fn build_reg_intervals(&mut self) {
        self.reg_intervals.clear();
        let mut regs: HashMap<Reg, (i32, i32)> = HashMap::new();
        self.live_in.iter().for_each(|reg| {
            regs.insert(*reg, (-1, -1));
        });
        let mut index = 0;
        while index < self.insts.len() {
            let inst = self.insts.get(index).unwrap();
            let uses = inst.get_reg_use();
            let defs = inst.get_reg_def();
            for reg in uses {
                debug_assert!(regs.contains_key(&reg));
                regs.get_mut(&reg).unwrap().1 = index as i32;
            }
            for reg in defs {
                if regs.contains_key(&reg) {
                    let (born, die) = regs.remove(&reg).unwrap();
                    self.reg_intervals.insert((reg, born), (reg, die));
                }
                regs.insert(reg, (index as i32, index as i32));
            }
            index += 1;
        }
        debug_assert!(regs.len() >= self.live_out.len());
        regs.iter().for_each(|(reg, (born, die))| {
            let mut die = *die;
            if self.live_out.contains(reg) {
                die = index as i32;
            }
            self.reg_intervals.insert((*reg, *born), (*reg, die));
        });
    }
}
