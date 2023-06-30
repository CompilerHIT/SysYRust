use std::alloc::Layout;
use std::ptr;
use std::marker::PhantomData;

use super::{BiVec, default_dangle}; 

impl <T> BiVec<T> {
    pub fn new() -> Self {
        BiVec {
            // contents: [ptr::null(), ptr::null()], 
            contents: [default_dangle(), default_dangle()],
            len: 0, 
            capacity: 0, 
            flag: PhantomData, 
        }
    } 
}

impl <T> BiVec<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        if std::mem::size_of::<T>() == 0 || capacity == 0 {
            return BiVec::new(); 
        }
        let layout : Layout = Layout::array::<T>(capacity).unwrap(); 
        let ptr = unsafe { std::alloc::alloc(layout) } as *const T;  
        let ptr2 = unsafe { std::alloc::alloc(layout) } as *const T; 
        BiVec {
            contents: [ptr, ptr2], 
            len: 0, 
            capacity: capacity, 
            flag: PhantomData, 
        } 
    }
}

impl <T> Drop for BiVec<T> {
    fn drop(&mut self) {
        for ele in self.contents {
            for i in 0..self.len {
                unsafe { 
                    ptr::drop_in_place(
                        ele.add(i) as *mut T 
                    ); 
                }
            } 
        }
        if std::mem::size_of::<T>() == 0 { 
            return; 
        }
        if self.capacity != 0 {
            let layout : Layout = Layout::array::<T>(self.capacity).unwrap(); 
            unsafe { std::alloc::dealloc(self.contents[0] as *mut u8, layout) }; 
            unsafe { std::alloc::dealloc(self.contents[1] as *mut u8, layout) }; 
        }
    } 
}

#[test] 
fn test_new_private() {
    let bivec : BiVec<i32> = BiVec::new(); 
    assert_eq!(bivec.len, 0); 
    assert_eq!(bivec.capacity, 0); 
} 

#[test] 
fn test_new_public() {
    let bivec : BiVec<i32> = BiVec::new(); 
    let len = bivec.len(); 
    let capacity = bivec.capacity(); 
    assert_eq!(len, 0); 
    assert_eq!(capacity, 0); 
}

#[test] 
fn test_with_capacity_public() {
    let bivec : BiVec<i32> = BiVec::with_capacity(10); 
    let len = bivec.len(); 
    let capacity = bivec.capacity(); 
    assert_eq!(len, 0); 
    assert_eq!(capacity, 10); 
}

#[test] 
fn test_with_capacity_private() {
    let bivec : BiVec<String> = BiVec::with_capacity(20); 
    assert_eq!(bivec.len, 0); 
    assert_eq!(bivec.capacity, 20); 
}