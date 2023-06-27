
use std::ptr;

use super::BiVec;

impl <T> BiVec<T> {
    pub fn set_capacity_or_nothing(&mut self, capacity: usize) {
        if self.capacity >= capacity {
            return; 
        } 
        if std::mem::size_of::<T>() == 0 {
            self.capacity = capacity; 
            return; 
        }
        let layout = std::alloc::Layout::array::<T>(capacity).unwrap(); 
        let ptr = unsafe { std::alloc::alloc(layout) } as *const T; 
        let ptr2 = unsafe { std::alloc::alloc(layout) } as *const T; 
        unsafe {
            ptr::copy_nonoverlapping(self.contents[0], ptr as *mut T, self.len); 
            ptr::copy_nonoverlapping(self.contents[1], ptr2 as *mut T, self.len);  
        }
        if self.capacity != 0 {
            let layout = std::alloc::Layout::array::<T>(self.capacity).unwrap(); 
            unsafe {
                std::alloc::dealloc(self.contents[0] as *mut u8, layout); 
                std::alloc::dealloc(self.contents[1] as *mut u8, layout); 
            }
        }
        self.contents[0] = ptr; 
        self.contents[1] = ptr2; 
        self.capacity = capacity; 
        return ; 
    }
    pub fn trim(&mut self) {
        if self.capacity == self.len {
            return ; 
        }
        if std::mem::size_of::<T>() == 0 {
            self.capacity = self.len; 
            return ; 
        } 
        let (ptr, ptr2); 
        if self.len != 0 {
            let layout = std::alloc::Layout::array::<T>(self.len).unwrap(); 
            ptr = unsafe { std::alloc::alloc(layout) } as *const T; 
            ptr2 = unsafe { std::alloc::alloc(layout) } as *const T; 
            unsafe {
                ptr::copy_nonoverlapping(self.contents[0], ptr as *mut T, self.len); 
                ptr::copy_nonoverlapping(self.contents[1], ptr2 as *mut T, self.len);  
            } 
        } else {
            ptr = std::ptr::null(); 
            ptr2 = std::ptr::null();   
        }
        if self.capacity != 0 {
            let layout = std::alloc::Layout::array::<T>(self.capacity).unwrap(); 
            unsafe {
                std::alloc::dealloc(self.contents[0] as *mut u8, layout); 
                std::alloc::dealloc(self.contents[1] as *mut u8, layout); 
            }
        } 
        self.contents[0] = ptr; 
        self.contents[1] = ptr2; 
        self.capacity = self.len; 
    }
    pub fn clear(&mut self) {
        if std::mem::size_of::<T>() == 0 {
            self.len = 0; 
            return ; 
        } 
        let len = self.len; 
        for content in self.contents {
            for i in 0..len {
                unsafe {
                    let p = content.add(i) as *mut T; 
                    ptr::drop_in_place(p); 
                }
            } 
        }
        self.len = 0; 
    } 
    pub fn reserve(&mut self, add_capacity: usize) {
        let capacity = self.capacity + add_capacity; 
        self.set_capacity_or_nothing(capacity); 
    } 
}

#[test] 
fn reserve_1() {
    let mut bivec : BiVec<i32> = BiVec::new(); 
    assert_eq!(bivec.capacity(), 0); 
    bivec.reserve(10); 
    assert_eq!(bivec.capacity(), 10);  
    bivec.reserve(5); 
    assert_eq!(bivec.capacity(), 15); 
    bivec.reserve(30); 
    assert_eq!(bivec.capacity(), 45); 
}

#[test] 
fn trim_1() {
    let mut bivec : BiVec<i32> = BiVec::new(); 
    bivec.reserve(10); 
    assert_eq!(bivec.capacity(), 10); 
    bivec.trim(); 
    assert_eq!(bivec.capacity(), 0); 
    bivec.push(0, 1); 
    assert_eq!(bivec.len(), 1); 
    bivec.trim(); 
    assert_eq!(bivec.capacity(), 1); 
}