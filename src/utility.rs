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
