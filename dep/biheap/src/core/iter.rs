use super::*; 

pub struct RefIter<'a, T: Ord> {
    pub(crate) bi_heap: &'a BiHeap<T>,
    pub(crate) index: usize,
} 

impl <'a, T: Ord> Iterator for RefIter<'a, T> {

    type Item = &'a T; 

    fn next(&mut self) -> Option<Self::Item> {
        let node = unsafe { &*self.bi_heap.0.get() }.views()[0].get(self.index)?; 
        self.index += 1; 
        let value = unsafe { &*node.get() }; 
        let value = &value.value; 
        Some(value) 
    }

} 

impl <'a, T: Ord> IntoIterator for &'a BiHeap<T> {
    type Item = &'a T; 

    type IntoIter = RefIter<'a, T>; 

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            bi_heap: self, 
            index: 0, 
        } 
    }
}

impl <T: Ord> BiHeap<T> {
    pub fn iter(&self) -> RefIter<T> {
        self.into_iter() 
    } 
}