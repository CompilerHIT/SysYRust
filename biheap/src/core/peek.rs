use std::{error::Error, fmt::Display, ops::{Deref, DerefMut}};

use super::*; 

#[derive(Debug)]
pub enum ViewErr {
    MismatchHeap, 
    MissValue, 
}

impl Error for ViewErr {} 

impl Display for ViewErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl <T: Ord> BiHeap<T> {
    pub fn peek(&self, handle: &Indexer<T>) -> Result<&T, ViewErr> { 
        let weak_ref = Rc::downgrade(&self.0); 
        if !Weak::ptr_eq(&weak_ref, &handle.heap_ref) {
            return Err(ViewErr::MismatchHeap);  
        }
        let value = handle.node_ref.upgrade().ok_or(ViewErr::MissValue)?;
        let value = unsafe { &*value.get() }; 
        let value = &value.value; 
        let value = unsafe { &*(value as *const T) }; 
        Ok(value) 
    } 
    #[deprecated]
    /// # Deprecated 
    /// Use `peek` instead. 
    pub fn as_view(&self, handle: &Indexer<T>) -> Result<&T, ViewErr> {
        self.peek(handle) 
    }
}

impl <T: Ord> BiHeap<T> {
    /// Returns a mutable reference related to the handle. 
    /// 
    /// # Examples 
    /// ``` 
    /// use biheap::BiHeap; 
    /// let mut be = BiHeap::new(); 
    /// let handle = be.push(1); 
    /// let mut view = be.peek_mut(&handle).unwrap(); 
    /// *view = 2; 
    /// drop(view); 
    /// assert_eq!(be.peek(&handle).unwrap(), &2); 
    /// ``` 
    pub fn peek_mut<'a> (&'a mut self, handle: &'_ Indexer<T>) -> Result<PeekMut<'a, T>, ViewErr> {
        let weak_ref = Rc::downgrade(&self.0); 
        if !Weak::ptr_eq(&weak_ref, &handle.heap_ref) {
            return Err(ViewErr::MismatchHeap);  
        } 
        let value = handle.node_ref.upgrade().ok_or(ViewErr::MissValue)?; 
        let view = ViewMut {
            bi_heap: self, 
            node: Some(value), 
        }; 
        Ok(view)
    }
    #[deprecated] 
    /// # Deprecated 
    /// Use `peek_mut` instead. 
    pub fn as_view_mut<'a> (&'a mut self, handle: &'_ Indexer<T>) -> Result<ViewMut<'a, T>, ViewErr> {
        self.peek_mut(handle) 
    }
}

impl <'a, T: Ord> ViewMut<'a, T> {
    #[deprecated] 
    /// # Deprecated 
    /// Use `deref` instead. 
    pub fn peek(&self) -> &T {
        self 
    }
    #[deprecated] 
    /// # Deprecated 
    /// Use `deref` instead. 
    pub fn get(&self) -> &T {
        self 
    } 
    #[deprecated]
    /// # Deprecated 
    /// Use `deref_mut` instead. 
    pub fn set(&mut self, mut value: T) -> T {
        std::mem::swap(&mut value, &mut unsafe { &mut *self.node.as_ref().unwrap().get() }.value); 
        value 
    } 
    /// Removes the value related to the handle from the heap and returns it. 
    /// 
    /// # Examples 
    /// ```
    /// use biheap::BiHeap; 
    /// let mut be = BiHeap::new(); 
    /// let handle = be.push(1); 
    /// let view = be.peek_mut(&handle).unwrap(); 
    /// assert_eq!(view.pop(), 1); 
    /// ``` 
    pub fn pop(mut self) -> T {
        let node = self.node.take().unwrap(); 
        let (min_index, max_index); 
        {
            let bor = unsafe { &*node.get() }; 
            min_index = bor.min_index; 
            max_index = bor.max_index; 
        }
        let bivec = unsafe { &mut *self.bi_heap.0.get() }; 
        bivec.swap_remove(min_index, max_index); 
        let [slice1, slice2]  = bivec.views_mut(); 
        let mut min_exist = false; 
        let mut max_exist = false; 
        slice1.get_mut(min_index).map(|f| { unsafe { &mut *f.get() } .min_index = min_index ; min_exist = true; } ); 
        slice2.get_mut(max_index).map(|f| { unsafe { &mut *f.get() } .max_index = max_index ; max_exist = true; } ); 
        drop(bivec); 
        if min_exist {
            self.bi_heap.bubble_down::<true>(min_index);
            self.bi_heap.bubble_up::<true>(min_index); 
        }
        if max_exist {
            self.bi_heap.bubble_down::<false>(max_index); 
            self.bi_heap.bubble_up::<false>(max_index); 
        }
        let node = Rc::try_unwrap(node).ok().unwrap(); 
        let node = node.into_inner(); 
        node.value 
    }
}

impl <'a, T: Ord> Drop for PeekMut<'a, T> {
    fn drop(&mut self) {
        if let Some(ref mut node) = self.node {
            let (min_index, max_index); 
            {
                let borrow = unsafe { & *node.get() }; 
                min_index = borrow.min_index; 
                max_index = borrow.max_index; 
            } 
            self.bi_heap.bubble_down::<true>(min_index); 
            self.bi_heap.bubble_up::<true>(min_index); 
            self.bi_heap.bubble_down::<false>(max_index); 
            self.bi_heap.bubble_up::<false>(max_index); 
        }
    }
}

impl <T: Ord> Deref for PeekMut<'_, T> {
    type Target = T; 

    fn deref(&self) -> &Self::Target {
        let r; 
        unsafe { 
            r = &*self.node.as_ref().unwrap().get(); 
        } 
        &r.value
    }
} 

impl <T: Ord> DerefMut for PeekMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let r; 
        unsafe { 
            r = &mut *self.node.as_ref().unwrap().get(); 
        } 
        &mut r.value 
    }
} 