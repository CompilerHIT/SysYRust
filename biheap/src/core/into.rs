use super::*; 

impl <T: Ord> BiHeap<T> {
    /// # TODO 
    /// Optimize the time complexity of this function. 
    pub fn into_vec(mut self) -> Vec<T> {
        let mut vec = Vec::new();
        while let Some(item) = self.pop_min() {
            vec.push(item);
        }
        vec
    } 
}