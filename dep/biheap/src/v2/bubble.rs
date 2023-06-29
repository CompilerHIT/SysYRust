use super::{BiHeap, RawBiVec};

pub enum BubbleOk {
    EndsAt(usize), 
    NotChange, 
}

#[derive(Debug)] 
pub enum BubbleErr {
    OutOfBounds, 
}

pub type BubbleResult = Result<BubbleOk, BubbleErr>; 

impl <T: Ord> BiHeap<T> {
    pub (crate) fn bubble_down<const IS_MIN: bool>(&mut self, index: usize) -> BubbleResult {
        #[cfg(feature = "threadsafe")] 
        {
            let mut bi_vec = self.bi_vec.lock().unwrap(); 
            bi_vec.bubble_down::<IS_MIN>(index)  
        }
        #[cfg(not(feature = "threadsafe"))] 
        {
            let mut bi_vec = self.bi_vec.borrow_mut(); 
            bi_vec.bubble_down::<IS_MIN>(index) 
        } 
    }
}

impl <T: Ord> BiHeap<T> {
    pub (crate) fn bubble_pop<const IS_MIN: bool>(&mut self, index: usize) -> BubbleResult {
        #[cfg(feature = "threadsafe")] 
        {
            let mut bi_vec = self.bi_vec.lock().unwrap(); 
            bi_vec.bubble_pop::<IS_MIN>(index)  
        }
        #[cfg(not(feature = "threadsafe"))] 
        {
            let mut bi_vec = self.bi_vec.borrow_mut(); 
            bi_vec.bubble_pop::<IS_MIN>(index) 
        } 
    }
}

impl <T: Ord> RawBiVec<T> {
    pub (crate) fn bubble_down<const IS_MIN: bool>(&mut self, index: usize) -> BubbleResult {
        let bi_vec = self; 
        let len = bi_vec.max.len(); 
        (index < len).then_some(()).ok_or(BubbleErr::OutOfBounds)?;
        let old_index = index; 
        let mut index = index; 
        let vec = if IS_MIN { bi_vec.min.as_mut_slice() } else { bi_vec.max.as_mut_slice() }; 
        loop {
            let left = index * 2 + 1; 
            if left >= len { break; }             
            let mut select = left; 
            vec.get(left+1).map(|right| {
                if IS_MIN {
                    if right.borrow().data < vec[left].borrow().data {
                        select = left + 1; 
                    }  
                } else {
                    if right.borrow().data > vec[left].borrow().data {
                        select = left + 1;  
                    } 
                }
            }); 
            if IS_MIN {
                if vec[select].borrow().data >= vec[index].borrow().data {
                    break; 
                }
            } else {
                if vec[select].borrow().data <= vec[index].borrow().data {
                    break; 
                }
            } 
            vec.swap(select, index); 
            if IS_MIN {
                vec[select].borrow_mut().min_index = select; 
                vec[index].borrow_mut().min_index = index; 
            } else {
                vec[select].borrow_mut().max_index = select; 
                vec[index].borrow_mut().max_index = index; 
            }
            index = select; 
        }
        if index == old_index {
            Ok(BubbleOk::NotChange) 
        } else {
            Ok(BubbleOk::EndsAt(index)) 
        }
    }
}

impl <T: Ord> RawBiVec<T> {
    pub (crate) fn bubble_pop<const IS_MIN: bool>(&mut self, index: usize) -> BubbleResult {
        let bi_vec = self; 
        let len = bi_vec.max.len(); 
        (index < len).then_some(()).ok_or(BubbleErr::OutOfBounds)?;
        let old_index = index; 
        let mut index = index; 
        let vec = if IS_MIN { bi_vec.min.as_mut_slice() } else { bi_vec.max.as_mut_slice() }; 
        loop {
            if index == 0 { break; } 
            let parent = (index - 1) / 2; 
            if IS_MIN {
                if vec[index].borrow().data >= vec[parent].borrow().data {
                    break; 
                }
            } else {
                if vec[index].borrow().data <= vec[parent].borrow().data {
                    break; 
                }
            } 
            vec.swap(index, parent); 
            if IS_MIN {
                vec[index].borrow_mut().min_index = index; 
                vec[parent].borrow_mut().min_index = parent; 
            } else {
                vec[index].borrow_mut().max_index = index; 
                vec[parent].borrow_mut().max_index = parent; 
            }
            index = parent; 
        }
        if index == old_index {
            Ok(BubbleOk::NotChange) 
        } else {
            Ok(BubbleOk::EndsAt(index)) 
        } 
    }
}