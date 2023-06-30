use super::*; 

impl <T: Ord> Default for BiHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}