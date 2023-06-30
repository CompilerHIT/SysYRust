use super::*; 

impl <'a, T: 'a + Ord + Copy> Extend<&'a T> for BiHeap<T> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        for item in iter {
            self.push(*item); 
        }
    }
} 

impl <T: Ord> Extend<T> for BiHeap<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.push(item); 
        }
    } 
}