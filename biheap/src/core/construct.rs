use super::*; 

impl <T: Ord> BiHeap<T> {
    /// Create an empty `BiHeap<T>` as a max-min-biheap. 
    /// 
    /// # Examples 
    /// ``` 
    /// use biheap::BiHeap; 
    /// let mut biheap = BiHeap::<i32>::new(); 
    /// biheap.push(4); 
    /// ``` 
    pub fn new() -> Self {
        let bivec = BiVec::new(); 
        BiHeap(Rc::new(UnsafeCell::new(bivec)))
    }
    /// Create an empty `BiHeap<T>` with at least the specified capacity. 
    /// 
    /// The `BiHeap<T>` will be able to hold at least `capacity` elements without reallocating the array, but it still allocates the memory for the reference structure. 
    /// 
    /// # Examples 
    /// ``` 
    /// use biheap::BiHeap; 
    /// let mut biheap = BiHeap::<i32>::with_capacity(10); 
    /// biheap.push(4); 
    /// ``` 
    pub fn with_capacity(capacity: usize) -> Self {
        let bivec = BiVec::with_capacity(capacity); 
        BiHeap(Rc::new(UnsafeCell::new(bivec)))
    } 
}