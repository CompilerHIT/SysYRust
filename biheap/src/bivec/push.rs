use std::mem::forget;

use super::BiVec;

impl <T> BiVec<T> {
    pub fn push_inplace(&mut self, element: &mut Option<(T, T)>) {
        assert!(element.is_some()); 
        if self.len == self.capacity {
            return ; 
        }
        if std::mem::size_of::<T>() == 0 {
            self.len += 1; 
            let t = element.take().unwrap(); 
            forget(t); 
            return ; 
        } 
        let (p1, p2); 
        unsafe {
            p1 = self.contents[0].add(self.len) as *mut T; 
            p2 = self.contents[1].add(self.len) as *mut T; 
        } 
        let (a, b) = element.take().unwrap(); 
        unsafe { 
            p1.write(a); 
            p2.write(b); 
        } 
        self.len += 1; 
    }
    pub fn push(&mut self, a: T, b: T) {
        let mut packed = Some((a, b)); 
        self.push_inplace(&mut packed); 
        if packed.is_some() {
            let capa = self.capacity + 2; 
            self.reserve(capa); 
            self.push_inplace(&mut packed); 
        }
        assert!(packed.is_none()); 
    }
}