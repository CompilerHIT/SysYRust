use std::cell::{Ref, RefCell};
use std::rc::Rc;

use super::{RawNode, BiHeap, Weak, RawBiVec};

pub struct View<T: Ord> {
    pub (crate) origin_heap: Weak<RawBiVec<T>>, 
    pub (crate) raw_node: Weak<RawNode<T>>,
} 

#[derive(Debug)]
pub enum TakeErr {
    HeapDroped, 
    NodeMissing,
}

impl <T: Ord> View <T> {
    pub fn take(self) -> Result<T, TakeErr> {
        let origin_heap = self.origin_heap.upgrade().ok_or(TakeErr::HeapDroped)?; 
        let raw_node: Rc<RefCell<RawNode<T>>> = self.raw_node.upgrade().ok_or(TakeErr::NodeMissing)?; 
        let mut origin_heap = origin_heap.borrow_mut(); 
        let (min_index, max_index); 
        {
            let raw_node = raw_node.borrow(); 
            min_index = raw_node.min_index; 
            max_index = raw_node.max_index; 
        }
        origin_heap.min.swap_remove(min_index); 
        origin_heap.min.get_mut(min_index).map(|node| node.borrow_mut().min_index = min_index); 
        origin_heap.max.swap_remove(max_index); 
        origin_heap.max.get_mut(max_index).map(|node| node.borrow_mut().max_index = max_index); 
        let _ = origin_heap.bubble_down::<true>(min_index); 
        let _ = origin_heap.bubble_pop::<true>(min_index); 
        let _ = origin_heap.bubble_down::<false>(max_index); 
        let _ = origin_heap.bubble_pop::<false>(max_index); 
        let val = std::rc::Rc::try_unwrap(raw_node).ok().unwrap().into_inner();
        Ok(val.data) 
    }
}

impl <T: Ord> View<T> {
    pub fn swap_with(&mut self, f: impl FnOnce() -> T) -> Result<T, TakeErr> {
        let origin_heap = self.origin_heap.upgrade().ok_or(TakeErr::HeapDroped)?; 
        let raw_node = self.raw_node.upgrade().ok_or(TakeErr::NodeMissing)?; 
        let mut origin_heap = origin_heap.borrow_mut(); 
        let (min_index, max_index); 
        {
            let raw_node = raw_node.borrow(); 
            min_index = raw_node.min_index; 
            max_index = raw_node.max_index; 
        }
        let val_node = origin_heap.min[min_index].clone(); 
        let mut f = f(); 
        std::mem::swap(&mut val_node.borrow_mut().data, &mut f); 
        drop(val_node); 
        let _ = origin_heap.bubble_down::<true>(min_index); 
        let _ = origin_heap.bubble_pop::<true>(min_index);
        let _ = origin_heap.bubble_down::<false>(max_index);
        let _ = origin_heap.bubble_pop::<false>(max_index); 
        Ok(f) 
    }
    #[inline]
    pub fn swap(&mut self, val: T) -> Result<T, TakeErr> {
        self.swap_with(|| val) 
    }
}

pub struct Wrapper<'a, T: Ord> {
    pub(crate) inner_node: Rc<RefCell<RawNode<T>>>, 
    pub(crate) origin_heap: &'a BiHeap<T>, 
} 

pub struct WrapperMut<'a, T: Ord> {
    pub(crate) is_min: bool, 
    pub(crate) origin_heap: &'a mut BiHeap<T>, 
}

impl <'a, T: Ord> Wrapper<'a, T> {
    pub fn inner_ref(&self) -> Ref<'_, T> {
        let inner = self.inner_node.borrow(); 
        Ref::map(inner, |inner| &inner.data) 
    }
}

impl <'a, T: Ord> WrapperMut<'a, T> {
    pub fn inner_ref(&self) -> Ref<'_, T> {
        // &self.
        unimplemented!()
    }
} 

impl <'a, T: Ord> WrapperMut<'a, T> {
    pub fn swap_with(&mut self, f: impl FnOnce() -> T) -> T {
        let mut f = f(); 
        let mut v = self.origin_heap.bi_vec.borrow_mut(); 
        let v2 = if self.is_min { &mut v.min } else { &mut v.max }; 
        std::mem::swap(&mut v2[0].borrow_mut().data, &mut f); 
        let b = v2[0].borrow(); 
        let min_index = b.min_index; 
        let max_index = b.max_index; 
        drop(b); 
        drop(v); 
        self.origin_heap.bubble_down::<true>(min_index).unwrap(); 
        self.origin_heap.bubble_pop::<true>(min_index).unwrap(); 
        self.origin_heap.bubble_down::<false>(max_index).unwrap(); 
        self.origin_heap.bubble_pop::<false>(max_index).unwrap(); 
        f 
    } 
    pub fn swap(&mut self, value: T) -> T {
        self.swap_with(|| value) 
    } 
}

impl <T: Ord> BiHeap<T> {
    pub fn max(&self) -> Option<Wrapper<'_, T>> {
        let bi_vec = self.bi_vec.borrow(); 
        let max = bi_vec.max.first()?; 
        let max = max.clone(); 
        Some(Wrapper {
            inner_node: max, 
            origin_heap: self, 
        }) 
    }
    pub fn min(&self) -> Option<Wrapper<'_, T>> {
        let bi_vec = self.bi_vec.borrow(); 
        let min = bi_vec.min.first()?; 
        let min = min.clone(); 
        Some(Wrapper {
            inner_node: min, 
            origin_heap: self, 
        })  
    }
}

impl <T: Ord> BiHeap<T> {
    pub fn max_mut(&mut self) -> Option<WrapperMut<'_, T>> {
        if self.bi_vec.borrow().max.is_empty() {
            return None; 
        } 
        Some(WrapperMut {
            origin_heap: self,
            is_min: false, 
        }) 
    } 
    pub fn min_mut(&mut self) -> Option<WrapperMut<'_, T>> {
        if self.bi_vec.borrow().min.is_empty() {
            return None; 
        } 
        Some(WrapperMut {
            origin_heap: self, 
            is_min: true, 
        }) 
    } 
}

impl <'a, T: Ord> Wrapper<'a, T> {
    pub fn as_view(self) -> View<T> {
        View {
            origin_heap: Rc::downgrade(&self.origin_heap.bi_vec),
            raw_node: Rc::downgrade(&self.inner_node), 
        } 
    } 
}

impl <'a, T: Ord> WrapperMut<'a, T> {
    pub fn as_view(self) -> View<T> {
        let node; 
        if self.is_min {
            node = self.origin_heap.bi_vec.borrow_mut().min[0].clone();  
        } else {
            node = self.origin_heap.bi_vec.borrow_mut().max[0].clone(); 
        } 
        let raw_node = Rc::downgrade(&node); 
        drop(node); 
        View {
            origin_heap: Rc::downgrade(&self.origin_heap.bi_vec),
            raw_node, 
        }  
    }
}