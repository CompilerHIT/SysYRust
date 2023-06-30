use super::{BiHeap, RawBiVec, Shared};

impl <T: Ord> BiHeap<T> {
    pub fn new() -> Self {
        let bi_vec = RawBiVec {
            max: Vec::new(), 
            min: Vec::new(), 
        }; 
        #[cfg(feature = "threadsafe")]
        let bi_vec = Shared::new(std::sync::Mutex::new(bi_vec)); 
        #[cfg(not(feature = "threadsafe"))] 
        let bi_vec = Shared::new(std::cell::RefCell::new(bi_vec)); 
        BiHeap {
            bi_vec, 
        } 
    }
    pub fn with_capacity(capacity: usize) -> Self {
        let bi_vec = RawBiVec {
            max: Vec::with_capacity(capacity), 
            min: Vec::with_capacity(capacity), 
        }; 
        #[cfg(feature = "threadsafe")] 
        let bi_vec = Shared::new(std::sync::Mutex::new(bi_vec)); 
        #[cfg(not(feature = "threadsafe"))] 
        let bi_vec = Shared::new(std::cell::RefCell::new(bi_vec)); 
        BiHeap {
            bi_vec, 
        } 
    }
}