//! src/ir/Instruction/mod.rs

use super::{ir_type::IrType, user::User, IList};

pub struct Inst {
    user: User,
    list: IList<Inst>,
    kind: InstKind,
}

#[derive(Debug, Clone, Copy)]
pub enum InstKind {
    // 内存相关
    Alloca,
    Gep,
    Load,
    Store,

    // 计算指令
    Binary,
    Unary,

    // 跳转
    Branch,

    // 函数相关
    Call,
    Parameter,
    Return,

    // 常量
    ConstInt(i32),
    GlobalConstInt(i32),

    // Phi函数
    Phi,

    // 作为链表头存在，没有实际意义
    Head,
}

impl Inst {
    /// Inst的构造指令，建议使用各类型指令的函数来创建
    pub fn new(ir_type: IrType, kind: InstKind, operands: Vec<&Inst>, list: IList<Inst>) -> Self {
        Self {
            user: User::new(ir_type, operands),
            list,
            kind,
        }
    }

    pub fn get_kind(&self) -> InstKind {
        self.kind
    }

    pub fn get_use_list(&self) -> &mut Vec<&Inst> {
        self.user.get_use_list()
    }

    // 链表行为
    /// 判断是否为当前bb的第一条指令
    pub fn is_head(&self) -> bool {
        match self.list.get_prev().get_kind() {
            Head => true,
            _ => false,
        }
    }

    /// 判断是否为当前bb的最后一条指令
    pub fn is_tail(&self) -> bool {
        // 同上
        match self.list.get_next().get_kind() {
            Head => true,
            _ => false,
        }
    }

    /// 获得当前指令的前一条指令。若为第一条指令，则返回None
    pub fn get_prev(&self) -> Option<&Inst> {
        if self.is_head() {
            None
        } else {
            Some(self.list.get_prev())
        }
    }

    /// 获得当前指令的下一条指令。若为最后一条指令，则返回None
    pub fn get_next(&self) -> Option<&Inst> {
        if self.is_tail() {
            None
        } else {
            Some(self.list.get_next())
        }
    }

    /// 在当前指令之前插入一条指令
    pub fn insert_before(&mut self, inst: &mut Inst) {
        debug_assert_ne!(self.list.prev, None);

        let p = self.list.get_prev_mut();
        self.list.set_prev(inst);
        p.list.set_next(inst);
        inst.list.set_prev(p);
        inst.list.set_next(self);
    }

    /// 在当前指令之后插入一条指令
    pub fn insert_after(&mut self, inst: &mut Inst) {
        debug_assert_ne!(self.list.next, None);

        let p = self.list.get_next_mut();
        self.list.set_next(inst);
        p.list.set_prev(inst);
        inst.list.set_prev(self);
        inst.list.set_next(self);
    }

    /// 把自己从指令中移除
    pub fn remove_self(&mut self) {
        debug_assert_ne!(self.list.next, None);
        debug_assert_ne!(self.list.prev, None);

        let next = self.list.get_next_mut();
        let prev = self.list.get_prev_mut();

        next.list.set_prev(prev);
        prev.list.set_next(next);

        self.list.next = None;
        self.list.prev = None;
    }

    /// 构造一个Head
    pub fn make_head() -> Inst {
        Inst::new(
            IrType::Void,
            InstKind::Head,
            vec![],
            IList {
                prev: None,
                next: None,
            },
        )
    }
    /// 初始化Head
    pub fn init_head(&mut self) {
        if let Head = self.kind {
            self.list.set_prev(self);
            self.list.set_next(self);
        } else {
            debug_assert!(false);
        }
    }
}
