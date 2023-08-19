//! src/ir/mod.rs

use crate::utility::ObjPtr;
pub mod analysis;
pub mod basicblock;
mod dump_ir;
pub mod function;
pub mod instruction;
pub mod ir_type;
pub mod module;
mod tools;
mod transform;
pub mod user;
pub mod value;

pub use analysis::call_map::{call_map_gen, CallMap};
pub use dump_ir::dump_now;
pub use transform::add_interface;
pub use transform::optimizer_run;

/// 侵入式链表
#[derive(Debug, Clone)]
pub struct IList<T: 'static> {
    prev: Option<ObjPtr<T>>,
    next: Option<ObjPtr<T>>,
}

impl<T: 'static> IList<T> {
    pub fn set_prev(&mut self, value: ObjPtr<T>) {
        self.prev = Some(value);
    }

    pub fn set_next(&mut self, value: ObjPtr<T>) {
        self.next = Some(value);
    }

    pub fn get_prev(&self) -> Option<ObjPtr<T>> {
        self.prev
    }

    pub fn get_next(&self) -> Option<ObjPtr<T>> {
        self.next
    }
}
