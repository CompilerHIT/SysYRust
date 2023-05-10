//! src/ir/Instruction/mod.rs

use crate::utility::Pointer;
use std::any::Any;

pub mod binary_inst;
pub mod branch_inst;
pub mod call_inst;
pub mod const_int;
pub mod global_const_int;
pub mod head_inst;
pub mod return_inst;
pub mod unary_inst;

#[derive(PartialEq)]
pub enum InstructionType {
    IBinaryOpInst,
    IBranchInst,
    IConstInt,
    IGlobalConstInt,
    IUnaryOpInst,
    ICallInst,
    IReturn,

    /// 没有这个节点，你不需要获得
    IHead,
}

pub trait Instruction {
    fn get_type(&self) -> InstructionType;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// 获得下一个节点的不可变引用
    /// 如果当前节点是尾节点，则返回None
    fn next(&self) -> Option<Pointer<Box<dyn Instruction>>>;

    /// 获得上一个节点的不可变引用
    /// 如果当前节点是头节点，则返回None
    fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>>;

    /// 将node插入到当前节点之前
    fn insert_before(&mut self, node: Pointer<Box<dyn Instruction>>);

    /// 将node插入到当前节点之后
    fn insert_after(&mut self, node: Pointer<Box<dyn Instruction>>);

    /// 是否为头节点
    fn is_head(&self) -> bool;

    /// 是否为尾节点
    fn is_tail(&self) -> bool;

    /// 将当前结点从链表中移除
    fn remove_self(&mut self);

    /// 不要使用这个函数
    fn set_next(&mut self, node: Pointer<Box<dyn Instruction>>);
    /// 不要使用这个函数
    fn set_prev(&mut self, node: Pointer<Box<dyn Instruction>>);
}

struct IList {
    prev: Option<Pointer<Box<dyn Instruction>>>,
    next: Option<Pointer<Box<dyn Instruction>>>,
}

impl IList {
    fn set_next(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.next = Some(node);
    }

    fn set_prev(&mut self, node: Pointer<Box<dyn Instruction>>) {
        self.prev = Some(node);
    }

    pub fn next(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        match self.next {
            None => None,
            Some(ref node) => match node.borrow_mut().get_type() {
                InstructionType::IHead => None,
                _ => Some(node.clone()),
            },
        }
    }

    pub fn prev(&self) -> Option<Pointer<Box<dyn Instruction>>> {
        match self.prev {
            None => None,
            Some(ref node) => match node.borrow_mut().get_type() {
                InstructionType::IHead => None,
                _ => Some(node.clone()),
            },
        }
    }

    pub fn insert_before(&mut self, node: Pointer<Box<dyn Instruction>>) {
        if let Some(prev_op) = self.prev.as_mut() {
            let mut prev_in = prev_op.borrow_mut();
            let myself = prev_in.next().unwrap();
            let mut node_mut = node.borrow_mut();

            node_mut.set_next(myself);
            node_mut.set_prev(prev_op.clone());
            prev_in.set_next(node.clone());
        }
        self.prev = Some(node.clone());
    }

    pub fn insert_after(&mut self, node: Pointer<Box<dyn Instruction>>) {
        if let Some(next_op) = self.next.as_mut() {
            let mut prev_in = next_op.borrow_mut();
            let myself = prev_in.prev().unwrap();
            let mut node_mut = node.borrow_mut();

            node_mut.set_next(next_op.clone());
            node_mut.set_prev(myself);
            prev_in.set_prev(node.clone());
        }
        self.next = Some(node.clone());
    }

    pub fn is_head(&self) -> bool {
        self.prev.as_ref().unwrap().borrow().get_type() == InstructionType::IHead
    }

    pub fn is_tail(&self) -> bool {
        self.next.as_ref().unwrap().borrow().get_type() == InstructionType::IHead
    }

    pub fn remove_self(&mut self) {
        let prev = self.prev.take().unwrap();
        let next = self.next.take().unwrap();
        prev.borrow_mut().set_next(next.clone());
        next.borrow_mut().set_prev(prev.clone());
    }
}
