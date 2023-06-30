#[cfg(not(feature = "threadsafe"))]
pub type Shared<T> = std::rc::Rc<std::cell::RefCell<T>>; 
#[cfg(feature = "threadsafe")]
pub type Shared<T> = std::sync::Arc<std::sync::Mutex<T>>; 

#[cfg(not(feature = "threadsafe"))]
pub type Weak<T> = std::rc::Weak<std::cell::RefCell<T>>; 
#[cfg(feature = "threadsafe")]
pub type Weak<T> = std::sync::Weak<std::sync::Mutex<T>>; 

pub(crate) struct RawNode <T> {
    pub data: T, 
    pub min_index: usize, 
    pub max_index: usize, 
}

/// Data structure to store the two heap 
pub(crate) struct RawBiVec <T> {
    pub max: Vec<Shared<RawNode<T>>>, 
    pub min: Vec<Shared<RawNode<T>>>, 
}

/// BiHeap is a data structure with two heap components. 
/// 
/// # Functionality 
/// BiHeap supports the following operations: 
/// `push`: Add an element to the heap. 
/// `pop_max`: Remove the maximum element from the heap. 
/// `pop_min`: Remove the minimum element from the heap. 
/// `peek_max`: Get a reference to the maximum element. 
/// `peek_min`: Get a reference to the minimum element. 
/// `len`: Get the number of elements in the heap. 
/// `is_empty`: Check if the heap is empty. 
/// `clear`: Remove all elements from the heap. 
/// 
pub struct BiHeap <T: Ord> {
    pub(crate) bi_vec: Shared<RawBiVec<T>>, 
}

impl <T: Ord> BiHeap<T> {
    pub fn len(&self) -> usize {
        #[cfg(not(feature = "threadsafe"))]
        {
            let bi_vec = self.bi_vec.borrow(); 
            let len = bi_vec.max.len(); 
            debug_assert_eq!(len, bi_vec.min.len()); 
            len 
        }
        #[cfg(feature = "threadsafe")]
        {
            let bi_vec = self.bi_vec.lock().unwrap(); 
            let len = bi_vec.max.len(); 
            debug_assert_eq!(len, bi_vec.min.len()); 
            len 
        } 
    }
}

impl <T: Ord> BiHeap<T> {
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn debug_check(&self) {
        let bi_vec = self.bi_vec.borrow(); 
        let max_len = bi_vec.max.len(); 
        let min_len = bi_vec.min.len(); 
        debug_assert_eq!(max_len, min_len); 
        for i in 0..max_len {
            let min = &bi_vec.min[i]; 
            let minr = min.borrow(); 
            assert_eq!(minr.min_index, i); 
            let max_i = minr.max_index; 
            let max = &bi_vec.max[max_i]; 
            assert!(std::rc::Rc::ptr_eq(max, min));
        } 
    }
}

pub mod constructors; 
pub mod bubble; 
pub mod push; 
pub mod view; 
pub mod utils; 
pub mod indexer; 