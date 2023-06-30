use std::mem::swap;

use super::*; 

impl <T: Ord> BiHeap<T> {
    pub(crate) fn bubble_up<const IS_MIN_FIRST: bool>(&mut self, index: usize) {
        if std::mem::size_of::<T>() == 0 {
            return; 
        }
        let borrow = unsafe { &mut *self.0.get() }; 
        debug_assert!(index < borrow.len());
        let mut slice = borrow.views_mut(); 
        let slice = 
            if IS_MIN_FIRST {
                &mut slice[0] 
            } else {
                &mut slice[1] 
            };
        let mut index = index; 
        loop {
            if index == 0 {
                break; 
            }
            let parent_index = (index - 1) / 2; 
            let parent = slice.split_at_mut(index); 
            let this = parent.1.first_mut().unwrap(); 
            let parent = &mut parent.0[parent_index]; 
            let should_swap; 
            let this_v = unsafe { &mut *this.get() }; 
            let parent_v = unsafe { &mut *parent.get() }; 
            {
                should_swap = 
                    if IS_MIN_FIRST {
                        this_v.value < parent_v.value
                    } else {
                        this_v.value > parent_v.value
                    }; 
            }
            if should_swap {
                swap::<Rc<UnsafeCell<Node<T>>>>(this, parent); 
                if IS_MIN_FIRST {
                    this_v.min_index = parent_index; 
                    parent_v.min_index = index; 
                } else {
                    this_v.max_index = parent_index; 
                    parent_v.max_index = index; 
                }
                index = parent_index; 
            } else {
                break; 
            }
        }
    }
}

impl <T: Ord> BiHeap<T> {
    pub(crate) fn bubble_down<const IS_MIN_FIRST: bool>(&mut self, index: usize) {
        if std::mem::size_of::<T>() == 0 {
            return ; 
        }
        let borrow = unsafe { &mut *self.0.get() }; 
        debug_assert!(index < borrow.len()); 
        let mut slice = borrow.views_mut(); 
        let slice = 
            if IS_MIN_FIRST {
                &mut slice[0] 
            } else {
                &mut slice[1] 
            }; 
        let mut index = index; 
        loop {
            let left_index = index * 2 + 1; 
            if left_index >= slice.len() {
                break; 
            } 
            let (split1, split2) = slice.split_at_mut(left_index); 
            let this = &mut split1[index]; 
            let (left, others) = split2.split_first_mut().unwrap(); 
            let right = others.first_mut(); 
            let select; 
            let cell_ref; 
            let should_swap; 
            if let Some(right) = right {
                let left_ref = unsafe { &mut *left.get() }; 
                let right_ref = unsafe { &mut *right.get() }; 
                if IS_MIN_FIRST {
                    if left_ref.value < right_ref.value {
                        select = left_index; 
                        drop(left_ref); 
                        cell_ref = left; 
                    } else {
                        select = left_index + 1;  
                        drop(right_ref); 
                        cell_ref = right; 
                    }
                } else {
                    if left_ref.value > right_ref.value {
                        select = left_index; 
                        drop(left_ref); 
                        cell_ref = left; 
                    } else {
                        select = left_index + 1; 
                        drop(right_ref); 
                        cell_ref = right; 
                    } 
                }
            } else {
                select = left_index; 
                cell_ref = left; 
            }
            if IS_MIN_FIRST {
                let this = unsafe { &*this.get() }; 
                let cell = unsafe { &*cell_ref.get() }; 
                should_swap = this.value > cell.value; 
            } else {
                let this = unsafe { &*this.get() }; 
                let cell = unsafe { &*cell_ref.get() }; 
                should_swap = this.value < cell.value; 
            }
            if !should_swap {
                break; 
            }
            swap(this, cell_ref); 
            let mut this = unsafe { &mut *this.get() };
            let mut cell = unsafe { &mut *cell_ref.get() }; 
            if IS_MIN_FIRST {
                this.min_index = index; 
                cell.min_index = select; 
            } else {
                this.max_index = index; 
                cell.max_index = select;
            } 
            index = select; 
        }
    }
}