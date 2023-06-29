use super::BiVec;

impl <T> BiVec<T> {
    pub fn len(&self) -> usize {
        self.len 
    } 
    pub fn empty(&self) -> bool {
        self.len == 0 
    }
    pub fn capacity(&self) -> usize {
        self.capacity 
    } 
}