use super::*; 

impl <T: Ord, const N: usize> From<[T; N]> for BiHeap<T> {
    /// # TODO 
    /// Optimize the time complexity of this function. 
    fn from(array: [T; N]) -> Self {
        let mut heap = Self::with_capacity(N); 
        for item in array { 
            heap.push(item);
        }
        heap
    }
} 

impl <T: Ord> From<Vec<T>> for BiHeap<T> {
    /// # TODO 
    /// Optimize the time complexity of this function. 
    fn from(vec: Vec<T>) -> Self {
        let mut heap = Self::with_capacity(vec.len());
        for item in vec { 
            heap.push(item);
        }
        heap
    }
} 