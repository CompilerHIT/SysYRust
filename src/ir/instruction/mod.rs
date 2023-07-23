//! src/ir/Instruction/mod.rs
use std::fmt::Debug;

///! 此模块中存放有Inst和InstKind结构体的定义，还有
///! 所有Inst类型的公有方法和Head的简单实现。特定的
///! inst的相关实现放在当前目录下的其他文件中。
use super::{basicblock::BasicBlock, ir_type::IrType, user::User, IList};
use crate::utility::{ObjPool, ObjPtr};
mod alloca;
mod binary;
mod branch;
mod call;
mod float_to_int;
mod gep;
mod iconst;
mod int_to_float;
mod ireturn;
mod load;
mod parameter;
mod phi;
mod store;
mod unary;

#[derive(Clone)]
pub struct Inst {
    user: User,
    list: IList<Inst>,
    kind: InstKind,
    /// 第一个bool为true时, 如果当前数组长度为0，则是未初始化的
    /// 第二个bool为true时，如果当前i32值为0，那么这个地方其实是被一个变量初始化的
    init: ((bool, Vec<(bool, i32)>), (bool, Vec<(bool, f32)>)),
    parent_bb: Option<ObjPtr<BasicBlock>>,
}

#[derive(Clone)]
pub enum InstKind {
    // 内存相关
    Alloca(i32),
    Gep,
    Load,
    Store,

    // 计算指令
    Binary(BinOp),
    Unary(UnOp),

    // 跳转
    Branch,

    // 函数相关
    Call(String),
    Parameter,
    Return,

    // 类型转换
    FtoI,
    ItoF,

    // 常量
    ConstInt(i32),
    GlobalConstInt(i32),
    ConstFloat(f32),
    GlobalConstFloat(f32),

    // 全局变量
    GlobalInt(i32),
    GlobalFloat(f32),

    // Phi函数
    Phi,

    // 作为链表头存在，没有实际意义
    Head,
}

impl Debug for InstKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s;
        match self {
            InstKind::ConstInt(i) => s = format!("ConstInt({})", i),
            InstKind::ConstFloat(f) => s = format!("ConstFloat({})", f),
            InstKind::GlobalConstInt(i) => s = format!("GlobalConstInt({})", i),
            InstKind::GlobalConstFloat(f) => s = format!("GlobalConstFloat({})", f),
            InstKind::GlobalInt(i) => s = format!("GlobalInt({})", i),
            InstKind::GlobalFloat(f) => s = format!("GlobalFloat({})", f),
            InstKind::Alloca(i) => s = format!("Alloca({})", i),
            InstKind::Gep => s = format!("Gep"),
            InstKind::Load => s = format!("Load"),
            InstKind::Store => s = format!("Store"),
            InstKind::Binary(bop) => s = format!("Binary({:?})", bop),
            InstKind::Unary(uop) => s = format!("Unary({:?})", uop),
            InstKind::Branch => s = format!("Branch"),
            InstKind::Call(name) => s = format!("Call({})", name),
            InstKind::Parameter => s = format!("Parameter"),
            InstKind::Return => s = format!("Return"),
            InstKind::FtoI => s = format!("FtoI"),
            InstKind::ItoF => s = format!("ItoF"),
            InstKind::Phi => s = format!("Phi"),
            InstKind::Head => s = format!("Head"),
        }
        write!(f, "{}", s)
    }
}

impl Debug for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let none_bb = BasicBlock::new("GlobalZone".to_string());
        write!(
            f,
            "{:?} in {:?}",
            self.kind,
            self.parent_bb
                .unwrap_or_else(|| ObjPtr::new(&none_bb))
                .get_name()
        )
    }
}

impl PartialEq for InstKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Alloca(_), Self::Alloca(_)) => true,
            (Self::Gep, Self::Gep) => true,
            (Self::Load, Self::Load) => true,
            (Self::Store, Self::Store) => true,
            (Self::Binary(bop1), Self::Binary(bop2)) => bop1 == bop2,
            (Self::Unary(uop1), Self::Unary(uop2)) => uop1 == uop2,
            (Self::Branch, Self::Branch) => true,
            (Self::Call(_), Self::Call(_)) => true,
            (Self::Parameter, Self::Parameter) => true,
            (Self::Return, Self::Return) => true,
            (Self::FtoI, Self::FtoI) => true,
            (Self::ItoF, Self::ItoF) => true,
            (Self::ConstInt(_), Self::ConstInt(_)) => true,
            (Self::GlobalConstInt(_), Self::GlobalConstInt(_)) => true,
            (Self::ConstFloat(_), Self::ConstFloat(_)) => true,
            (Self::GlobalConstFloat(_), Self::GlobalConstFloat(_)) => true,
            (Self::GlobalInt(_), Self::GlobalInt(_)) => true,
            (Self::GlobalFloat(_), Self::GlobalFloat(_)) => true,
            (Self::Phi, Self::Phi) => true,
            (Self::Head, Self::Head) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Pos,
    Neg,
    Not,
}

impl Inst {
    /// Inst的构造指令，建议使用各类型指令的函数来创建
    pub fn new(ir_type: IrType, kind: InstKind, operands: Vec<ObjPtr<Inst>>) -> Self {
        Self {
            user: User::new(ir_type, operands),
            list: IList {
                prev: None,
                next: None,
            },
            kind,
            init: ((false, vec![]), (false, vec![])),
            parent_bb: None,
        }
    }

    pub fn get_ir_type(&self) -> IrType {
        self.user.get_ir_type()
    }

    pub fn set_ir_type(&mut self, ir_type: IrType) {
        self.user.set_ir_type(ir_type);
    }

    pub fn get_kind(&self) -> InstKind {
        self.kind.clone()
    }

    /// 获得使用该Inst的列表
    pub fn get_use_list(&self) -> &Vec<ObjPtr<Inst>> {
        self.user.get_use_list()
    }

    /// 增加user
    pub fn add_user(&mut self, inst: &Inst) {
        self.user.add_user(inst);
    }

    /// 删除user
    pub fn remove_user(&mut self, inst: &Inst) {
        self.user.delete_user(inst);
    }

    // 链表行为
    /// 判断是否为当前bb的第一条指令
    pub fn is_head(&self) -> bool {
        if let InstKind::Head = self.get_prev().get_kind() {
            true
        } else {
            false
        }
    }

    /// 判断是否为当前bb的最后一条指令的后一条指令
    pub fn is_tail(&self) -> bool {
        if let InstKind::Head = self.get_kind() {
            true
        } else {
            false
        }
    }

    /// 获得当前指令的前一条指令。
    pub fn get_prev(&self) -> ObjPtr<Inst> {
        let prev = self.list.get_prev();
        debug_assert_ne!(prev, None, "prev is None. inst: {:?}", self);
        prev.unwrap()
    }

    /// 获得当前指令的下一条指令。
    pub fn get_next(&self) -> ObjPtr<Inst> {
        let next = self.list.get_next();
        debug_assert_ne!(next, None, "next is None. inst: {:?}", self);
        next.unwrap()
    }

    /// 在当前指令之前插入一条指令
    pub fn insert_before(&mut self, mut inst: ObjPtr<Inst>) {
        let p = self.get_prev().as_mut();
        self.list.set_prev(inst);
        p.list.set_next(inst);
        inst.list.set_prev(ObjPtr::new(p));
        inst.list.set_next(ObjPtr::new(self));

        // 更新inst的parent_bb
        inst.parent_bb = self.parent_bb;
    }

    /// 在当前指令之后插入一条指令
    pub fn insert_after(&mut self, mut inst: ObjPtr<Inst>) {
        let p = self.get_next().as_mut();
        self.list.set_next(inst);
        p.list.set_prev(inst);
        inst.list.set_prev(ObjPtr::new(self));
        inst.list.set_next(ObjPtr::new(p));

        // 更新inst的parent_bb
        inst.parent_bb = self.parent_bb;
    }

    /// 把自己从指令中移除并删除use
    pub fn remove_self(&mut self) {
        let next = self.get_next().as_mut();
        let prev = self.get_prev().as_mut();

        next.list.set_prev(ObjPtr::new(prev));
        prev.list.set_next(ObjPtr::new(next));

        self.get_operands().iter().for_each(|op| {
            op.as_mut().remove_user(self);
        });

        self.list.next = None;
        self.list.prev = None;

        self.parent_bb = None;
    }

    /// 把自己从指令序列中删除但不删除use
    pub fn move_self(&mut self) {
        let next = self.get_next().as_mut();
        let prev = self.get_prev().as_mut();

        next.list.set_prev(ObjPtr::new(prev));
        prev.list.set_next(ObjPtr::new(next));

        self.list.next = None;
        self.list.prev = None;

        self.parent_bb = None;
    }

    /// 获得当前指令所在的bb
    pub fn get_parent_bb(&self) -> ObjPtr<BasicBlock> {
        if let Some(bb) = self.parent_bb {
            bb
        } else {
            unreachable!("Inst's parent_bb is None. inst: {:?}", self);
        }
    }

    /// 构造一个Head
    pub fn make_head() -> Inst {
        Inst::new(IrType::Void, InstKind::Head, vec![])
    }
    /// 初始化Head
    pub fn init_head(&mut self, bb: ObjPtr<BasicBlock>) {
        if let InstKind::Head = self.kind {
            self.list.set_prev(ObjPtr::new(self));
            self.list.set_next(ObjPtr::new(self));
            self.kind = InstKind::Head;
            self.parent_bb = Some(bb);
        } else {
            debug_assert!(false);
        }
    }
}
