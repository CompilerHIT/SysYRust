use super::BiHeap;

impl <T: Ord> BiHeap<T> {
    pub fn pop_max(&mut self) -> Option<T> {
        Some ( self.max()?.as_view().take().unwrap() ) 
    }
    pub fn pop_min(&mut self) -> Option<T> {
        Some ( self.min()?.as_view().take().unwrap() ) 
    } 
}