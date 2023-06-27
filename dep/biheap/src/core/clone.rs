use super::*; 

impl <T: Ord + Clone> Clone for BiHeap<T> {
    /// # TODO 
    /// Optimize the time complexity of this function. 
    fn clone(&self) -> Self {
        let mut heap = Self::with_capacity(self.len());
        for item in self.iter() { 
            heap.push(item.clone());
        } 
        heap 
    }
}