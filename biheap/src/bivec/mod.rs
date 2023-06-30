use std::marker::PhantomData;

pub const DEFAULT_DANGLE: *const u8 = 0x4096 as *const u8; 
pub const fn default_dangle<T>() -> *const T {
    DEFAULT_DANGLE as *const T
} 

pub struct BiVec <T> {
    pub(crate) contents: [*const T; 2], 
    pub(crate) len: usize, 
    pub(crate) capacity: usize, 
    flag: PhantomData<T>, 
}

mod construct; 
mod extend;
mod push;
mod pop;
mod property; 

pub mod view;
pub mod order;

unsafe impl <T: Send> Send for BiVec<T> {} 
unsafe impl <T: Sync> Sync for BiVec<T> {} 