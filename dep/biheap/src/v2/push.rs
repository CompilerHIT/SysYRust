use super::{BiHeap, Shared};

impl <T: Ord> BiHeap<T> {
    pub fn push(&mut self, value: T) {
        let mut bi_vec = self.bi_vec.borrow_mut(); 
        let index = bi_vec.max.len(); 
        let value = super::RawNode {
            data: value, 
            min_index: index, 
            max_index: index, 
        }; 
        #[cfg(not(threadsafe))]
        let rc = Shared::new(std::cell::RefCell::new(value));
        #[cfg(threadsafe)] 
        let rc = Shared::new(std::sync::Mutex::new(value)); 
        bi_vec.max.push(rc.clone()); 
        bi_vec.min.push(rc); 
        drop(bi_vec);
        self.bubble_pop::<true>(index).unwrap(); 
        self.bubble_pop::<false>(index).unwrap();
    }
}