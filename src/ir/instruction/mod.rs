//! src/ir/Instruction/mod.rs
///! 此模块中存放有Inst和InstKind结构体的定义，还有
///! 所有Inst类型的公有方法和Head的简单实现。特定的
///! inst的相关实现放在当前目录下的其他文件中。
use super::{ir_type::IrType, user::User, IList};
use crate::utility::{ObjPool, ObjPtr};
mod alloca;
mod binary;
mod branch;
mod call;
mod gep;
mod iconst;
mod ireturn;
mod load;
mod parameter;
mod phi;
mod store;
mod unary;

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
    Binary(BinOp),
    Unary(UnOp),

    // 跳转
    Branch,

    // 函数相关
    Call(&'static str),
    Parameter,
    Return,

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

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Eq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
}

#[derive(Debug, Clone, Copy)]
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
        }
    }

    pub fn get_ir_type(&self) -> IrType {
        self.user.get_ir_type()
    }

    pub fn get_kind(&self) -> InstKind {
        self.kind
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
        match self.list.get_prev().as_ref().get_kind() {
            Head => true,
            _ => false,
        }
    }

    /// 判断是否为当前bb的最后一条指令
    pub fn is_tail(&self) -> bool {
        // 同上
        match self.list.get_next().as_ref().get_kind() {
            Head => true,
            _ => false,
        }
    }

    /// 获得当前指令的前一条指令。若为第一条指令，则返回None
    pub fn get_prev(&self) -> ObjPtr<Inst> {
        self.list.get_prev()
    }

    /// 获得当前指令的下一条指令。若为最后一条指令，则返回None
    pub fn get_next(&self) -> ObjPtr<Inst> {
        self.list.get_next()
    }

    /// 在当前指令之前插入一条指令
    pub fn insert_before(&mut self, inst: ObjPtr<Inst>) {
        let p = self.list.get_prev().as_mut();
        self.list.set_prev(inst);
        p.list.set_next(inst);
        inst.as_mut().list.set_prev(ObjPtr::new(p));
        inst.as_mut().list.set_next(ObjPtr::new(p));
    }

    /// 在当前指令之后插入一条指令
    pub fn insert_after(&mut self, inst: ObjPtr<Inst>) {
        let p = self.list.get_next().as_mut();
        self.list.set_next(inst);
        p.list.set_prev(inst);
        inst.as_mut().list.set_prev(ObjPtr::new(self));
        inst.as_mut().list.set_next(ObjPtr::new(self));
    }

    /// 把自己从指令中移除
    pub fn remove_self(&mut self) {
        let next = self.list.get_next().as_mut();
        let prev = self.list.get_prev().as_mut();

        next.list.set_prev(ObjPtr::new(prev));
        prev.list.set_next(ObjPtr::new(next));

        self.list.next = None;
        self.list.prev = None;
    }

    /// 构造一个Head
    pub fn make_head() -> Inst {
        Inst::new(IrType::Void, InstKind::Head, vec![])
    }
    /// 初始化Head
    pub fn init_head(&mut self) {
        if let Head = self.kind {
            self.list.set_prev(ObjPtr::new(self));
            self.list.set_next(ObjPtr::new(self));
        } else {
            debug_assert!(false);
        }
    }
}
