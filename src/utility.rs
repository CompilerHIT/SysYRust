use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

/// 使用Pointer<T>来替代Rc<RefCell<T>>，以便于简化操作
#[derive(Debug)]
pub struct Pointer<T> {
    p: Rc<RefCell<T>>,
}

impl<T> Pointer<T> {
    /// make a Pointer points to cell
    pub fn new(cell: T) -> Pointer<T> {
        Pointer {
            p: Rc::new(RefCell::new(cell)),
        }
    }

    pub fn borrow(&self) -> Ref<T> {
        self.p.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<T> {
        self.p.borrow_mut()
    }

    /// Returns true if the two Rcs point to the
    /// same allocation in a vein similar to ptr::eq.
    /// See that function for caveats when comparing `dyn Trait` pointers.
    pub fn point_eq(this: &Pointer<T>, other: &Pointer<T>) -> bool {
        Rc::ptr_eq(&this.p, &other.p)
    }
}

impl<T> Clone for Pointer<T> {
    // add code here
    fn clone(&self) -> Pointer<T> {
        Pointer { p: self.p.clone() }
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub enum ScalarType {
    Int,
    Float,
}

/// 嵌入式链表
trait IList {
    type Item;
    /// 获得下一个节点的不可变引用
    /// 如果当前节点是尾节点，则返回None
    fn next(&self) -> Option<Ref<Self::Item>>;

    /// 获得上一个节点的不可变引用
    /// 如果当前节点是头节点，则返回None
    fn prev(&self) -> Option<Ref<Self::Item>>;

    /// 获得下一个节点的可变引用
    /// 如果当前节点是尾节点，则返回None
    fn next_mut(&mut self) -> Option<RefMut<Self::Item>>;

    /// 获得上一个节点的可变引用
    /// 如果当前节点是头节点，则返回None
    fn prev_mut(&mut self) -> Option<RefMut<Self::Item>>;

    /// 将node插入到当前节点之前
    fn insert_before(&mut self, node: Pointer<Self::Item>);

    /// 将node插入到当前节点之后
    fn insert_after(&mut self, node: Pointer<Self::Item>);

    /// 是否为头节点
    fn is_head(&self) -> bool;

    /// 是否为尾节点
    fn is_tail(&self) -> bool;

    /// 将当前结点从链表中移除
    fn remove_self(&mut self);

    /// 移除当前节点之后的结点
    fn remove_after(&mut self);
    /// 移除当前节点之前的结点
    fn remove_before(&mut self);
}
