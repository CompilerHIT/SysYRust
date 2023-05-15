//! src/ir/mod.rs

use std::ptr::NonNull;
pub mod basicblock;
pub mod function;
pub mod instruction;
pub mod ir_type;
pub mod module;
pub mod user;
pub mod value;

/// 侵入式链表
pub struct IList<T: 'static> {
    prev: Option<NonNull<&'static T>>,
    next: Option<NonNull<&'static T>>,
}

impl<T: 'static> IList<T> {
    pub fn set_prev(&mut self, value: &'static T) {
        self.prev = unsafe { Some(NonNull::new_unchecked(value as *const _ as *mut _)) }
    }

    pub fn set_next(&mut self, value: &'static T) {
        self.next = unsafe { Some(NonNull::new_unchecked(value as *const _ as *mut _)) }
    }

    pub fn get_prev(&self) -> &'static T {
        debug_assert_ne!(self.prev, None);
        unsafe { self.prev.unwrap().as_ref() }
    }

    pub fn get_prev_mut(&mut self) -> &'static mut T {
        debug_assert_ne!(self.prev, None);
        unsafe { self.prev.unwrap().as_mut() }
    }

    pub fn get_next(&self) -> &'static T {
        debug_assert_ne!(self.next, None);
        unsafe { self.next.unwrap().as_ref() }
    }

    pub fn get_next_mut(&self) -> &'static mut T {
        debug_assert_ne!(self.next, None);
        unsafe { self.next.unwrap().as_mut() }
    }
}
