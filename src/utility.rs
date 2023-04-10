use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

/// 使用Pointer<T>来替代Rc<RefCell<T>>，以便于简化操作
pub struct Pointer<T> {
    p: Rc<RefCell<T>>,
}

impl<T> Pointer<T> {
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

    pub fn clone(&self) -> Pointer<T> {
        Pointer { p: self.p.clone() }
    }
}

#[derive(Clone, Copy, PartialEq, Hash, Eq)]
pub enum ScalarType {
    Int,
    Float,
}
