use super::BiHeap;

impl <T: Ord> BiHeap<T> {
    /// Returns the length of the heap 
    /// 
    /// # Examples 
    /// ```
    /// use biheap::BiHeap; 
    /// let mut be = BiHeap::new(); 
    /// assert_eq!(be.len(), 0); 
    /// be.push(1); 
    /// assert_eq!(be.len(), 1); 
    /// ``` 
    pub fn len(&self) -> usize {
        unsafe { &*self.0.get() } .len() 
    }
    /// Checks if the heap is empty. 
    /// 
    /// # Examples 
    /// ```
    /// use biheap::BiHeap;
    /// let mut be = BiHeap::new();
    /// assert!(be.is_empty());
    /// be.push(1);
    /// assert!(!be.is_empty());
    /// ``` 
    pub fn is_empty(&self) -> bool {
        self.len() == 0 
    } 
    /// Clears the heap, removing all elements. 
    /// 
    /// # Examples 
    /// ```
    /// use biheap::BiHeap;
    /// let mut be = BiHeap::new();
    /// be.push(1);
    /// be.push(2);
    /// be.push(3);
    /// be.clear();
    /// assert!(be.is_empty());
    /// ``` 
    pub fn clear(&mut self) {
        unsafe { &mut *self.0.get() }.clear();  
    }
}

impl <T: Ord + std::fmt::Debug> BiHeap<T> {
    #[cfg(test)]
    pub fn debug(&self) {
        let borrow = unsafe { &mut *self.0.get() };
        let [v1, v2] = borrow.views(); 
        let iter = v1.iter().enumerate(); 
        eprintln!("min slice");
        for (i, v) in iter {
            let min_index = unsafe { &*v.get() }.min_index; 
            let max_index = unsafe { &*v.get() }.max_index; 
            dbg!((i, min_index, max_index, &unsafe { &*v.get() }.value));
        }
        eprintln!("---\nmax slice");
        let iter = v2.iter().enumerate(); 
        for (i, v) in iter {
            let min_index = unsafe { &*v.get() }.min_index; 
            let max_index = unsafe { &*v.get() }.max_index; 
            dbg!((i, min_index, max_index, &unsafe { &*v.get() }.value));
        } 
    }
    #[cfg(test)]
    pub fn check(&self) {
        use crate::core::Rc;

        let borrow = unsafe { &*self.0.get() };
        let [v1, v2] = borrow.views(); 
        let iter = v1.iter().enumerate(); 
        for (i, v) in iter {

            let min_index = unsafe { &* v.get() }.min_index; 
            let max_index = unsafe { &* v.get() }.max_index; 
            assert_eq!(i, min_index); 
            let max = &v2[max_index]; 
            let eq = Rc::ptr_eq(&v, &max); 
            assert!(eq); 
        } 

    }
}