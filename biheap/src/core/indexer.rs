use super::*; 

impl <T: Ord> BiHeap<T> {
    /// Returns the maximum element indexer of the heap. 
    /// 
    /// # Examples 
    /// ``` 
    /// use biheap::BiHeap; 
    /// let mut be = BiHeap::new(); 
    /// be.push(1); 
    /// let handle = be.max_indexer(); 
    /// ``` 
    pub fn max_indexer(&self) -> Option<Handle<T>> {
        let borrow = unsafe { &*self.0.get() }; 
        let slice = borrow.views(); 
        let slice = &slice[1]; 
        if slice.is_empty() {
            None
        } else {
            let node_ref = Rc::downgrade(&slice[0]); 
            let heap_ref = Rc::downgrade(&self.0); 
            Some(Handle {
                node_ref, 
                heap_ref, 
            })
        }
    }  
    #[deprecated(note = "Use `max_indexer` instead")] 
    pub fn max_handle(&self) -> Option<Handle<T>> {
        self.max_indexer() 
    }
    /// Returns the minimum element handle of the heap. 
    /// 
    /// # Examples 
    /// ``` 
    /// use biheap::BiHeap; 
    /// let mut be = BiHeap::new(); 
    /// be.push(1); 
    /// let handle = be.min_indexer(); 
    /// ``` 
    pub fn min_indexer(&self) -> Option<Handle<T>> {
        let borrow = unsafe { & *self.0.get() }; 
        let slice = borrow.views(); 
        let slice = &slice[0]; 
        if slice.is_empty() {
            None
        } else {
            let node_ref = Rc::downgrade(&slice[0]); 
            let heap_ref = Rc::downgrade(&self.0); 
            Some(Handle {
                node_ref, 
                heap_ref, 
            })
        } 
    }
    #[deprecated(note = "Use `min_indexer` instead")]
    pub fn min_handle(&self) -> Option<Handle<T>> {
        self.min_indexer() 
    }
}

#[cfg(feature = "threadsafe")]
unsafe impl <T> Send for Indexer<T> {} 
#[cfg(feature = "threadsafe")] 
unsafe impl <T> Sync for Indexer<T> {} 
