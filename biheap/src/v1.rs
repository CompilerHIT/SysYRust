//! The v0.1.0 realization 

use std::mem::swap;
use std::cell::{RefCell, Ref};
use std::rc::Rc;
use std::cmp::Ordering;

/// Inner Node Structure. 
/// 
/// The `maximum_index` and `minimum_index` fields are used to store the index of the node in the max heap and min heap respectively. 
pub struct Node <T> {
    value: T, 
    minimum_index: usize, 
    maximum_index: usize, 
}

impl <T> Node<T> {
    // peek the value with reference. 
    pub fn value(&self) -> &T {
        &self.value
    }
} 

pub struct NodeHandle <'a, T: Ord> {
    node: Rc<RefCell<Node<T>>>, 
    heap: &'a mut BiHeap<T>,
}

impl <'a, T: Ord> NodeHandle<'a, T> {
    pub fn value<'b>(&'b self) -> Ref<'b, Node<T>> {
        self.node.borrow() 
    }
    pub fn set_value(&mut self, mut value: T) -> T {
        swap(
            &mut self.node.borrow_mut().value, 
            &mut value, 
        ); 
        self.heap.bubble_up::<true>(self.node.borrow().minimum_index); 
        self.heap.bubble_down::<true>(self.node.borrow().minimum_index);
        self.heap.bubble_up::<false>(self.node.borrow().maximum_index); 
        self.heap.bubble_down::<false>(self.node.borrow().maximum_index); 
        value 
    } 
    pub fn pop(self) -> T {
        let node = self.node; 
        let heap = self.heap; 
        let min_index = node.borrow().minimum_index; 
        let max_index = node.borrow().maximum_index; 
        heap.min_heap.swap_remove(min_index); 
        heap.min_heap.get_mut(min_index).map(|f| f.borrow_mut()).map(|mut f| f.minimum_index = min_index); 
        heap.max_heap.swap_remove(max_index); 
        heap.max_heap.get_mut(max_index).map(|f| f.borrow_mut()).map(|mut f| f.maximum_index = max_index); 
        if min_index < heap.min_heap.len() {
            heap.min_heap[min_index].borrow_mut().minimum_index = min_index; 
            heap.bubble_down::<true>(min_index); 
        } 
        if max_index < heap.max_heap.len() {
            heap.max_heap[max_index].borrow_mut().maximum_index = max_index; 
            heap.bubble_down::<false>(max_index); 
        } 
        let val = Rc::try_unwrap(node).ok().unwrap().into_inner().value;  
        val 
    }
}

pub struct BiHeap <T: Ord> {
    min_heap: Vec<Rc<RefCell<Node<T>>>>, 
    max_heap: Vec<Rc<RefCell<Node<T>>>>, 
}

impl <T: Ord> BiHeap<T> {
    pub fn max_handle(&mut self) -> Option<NodeHandle<'_, T>> { 
        let node = self.max_heap.get(0).map(Rc::clone); 
        node.map(|node| {
            NodeHandle {
                node, 
                heap: self, 
            } 
        })
    }
    pub fn min_handle(&mut self) -> Option<NodeHandle<'_, T>> { 
        let node = self.min_heap.get(0).map(Rc::clone); 
        node.map(|node| {
            NodeHandle {
                node, 
                heap: self, 
            } 
        }) 
    }
}

impl <T: Ord> BiHeap<T> {
    pub fn new() -> Self {
        BiHeap {
            min_heap: Vec::new(), 
            max_heap: Vec::new(), 
        }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        BiHeap {
            min_heap: Vec::with_capacity(capacity), 
            max_heap: Vec::with_capacity(capacity), 
        }  
    }
}

fn minimin_index_mut <T> (node: &mut Node<T>) -> &mut usize {
    &mut node.minimum_index
} 

fn maximax_index_mut <T> (node: &mut Node<T>) -> &mut usize {
    &mut node.maximum_index
} 

impl <T: Ord> BiHeap<T> {
    fn bubble_up <const IS_MIN: bool> (&mut self, index: usize) {
        if IS_MIN {
            assert!( index < self.min_heap.len() ); 
        } else {
            assert!( index < self.max_heap.len() );  
        }
        let heap = if IS_MIN { &mut self.min_heap } else { &mut self.max_heap }; 
        let accessor = if IS_MIN { minimin_index_mut } else { maximax_index_mut }; 
        let slice = &mut heap[..]; 
        let mut index = index; 
        while index != 0 {
            let pindex = (index - 1) / 2; 
            let (parent, child) = slice.split_at_mut(index); 
            let (parent, child) = (&mut parent[pindex], &mut child[0]); 
            let cmp = Ord::cmp(&parent.borrow().value, &child.borrow().value); 
            match cmp {
                Ordering::Less => {
                    if !IS_MIN {
                        *accessor(&mut parent.borrow_mut()) = index;  
                        *accessor(&mut child.borrow_mut()) = pindex; 
                        swap(parent, child); 
                    } else {
                        break; 
                    }
                }
                Ordering::Greater => {
                    if IS_MIN {
                        *accessor(&mut parent.borrow_mut()) = index;  
                        *accessor(&mut child.borrow_mut()) = pindex; 
                        swap(parent, child);  
                    } else {
                        break; 
                    }
                }
                Ordering::Equal => break, 
            }
            index = pindex; 
        }
    }
}

impl <T: Ord> BiHeap<T> {
    pub fn push(&mut self, value: T) {
        let node = Rc::new(RefCell::new(Node {
            value, 
            minimum_index: self.min_heap.len(), 
            maximum_index: self.max_heap.len(), 
        })); 
        self.min_heap.push(Rc::clone(&node)); 
        self.max_heap.push(Rc::clone(&node)); 
        drop(node); 
        self.bubble_up::<true>(self.min_heap.len() - 1);
        self.bubble_up::<false>(self.max_heap.len() - 1);
    }
} 

impl <T: Ord> BiHeap<T> {
    pub fn peek_min <'a> (&'a self) -> Option<Ref<'a, Node<T>>> {
        let r = self.min_heap.get(0); 
        r.map(|node| node.borrow()) 
    }
    pub fn peek_max <'a> (&'a self) -> Option<Ref<'a, Node<T>>> {
        let r = self.max_heap.get(0); 
        r.map(|node| node.borrow()) 
    }
}
 
impl <T: Ord> BiHeap<T> {
    pub fn pop_min(&mut self) -> Option<T> {
        if self.min_heap.len() == 1 {
            self.min_heap.pop(); 
            let max = self.max_heap.pop(); 
            let max = max.unwrap(); 
            let rc = Rc::try_unwrap(max).ok().unwrap(); 
            let rc = rc.into_inner(); 
            return Some(rc.value) 
        }
        let p = self.min_heap.pop(); 
        let Some(mut p) = p else {
            return None 
        }; 
        swap(&mut p, &mut self.min_heap[0]); 
        self.min_heap[0].borrow_mut().minimum_index = 0; 
        let max_index = p.borrow().maximum_index; 
        self.max_heap.swap_remove(max_index); 
        let is_ok = self.max_heap.get_mut(max_index).map(|a| a.borrow_mut()).map(|mut a| a.maximum_index = max_index).is_some(); 
        self.bubble_down::<true>(0); 
        is_ok.then_some(|| {
            self.bubble_down::<false>(max_index);
        }); 
        let r = Rc::try_unwrap(p).ok().unwrap(); 
        let r = r.into_inner().value; 
        return Some(r) 
    }
    pub fn pop_max(&mut self) -> Option<T> {
        if self.max_heap.len() == 1 {
            self.max_heap.pop(); 
            let min = self.min_heap.pop(); 
            let min = min.unwrap(); 
            let rc = Rc::try_unwrap(min).ok().unwrap(); 
            let rc = rc.into_inner(); 
            return Some(rc.value) 
        }
        let p = self.max_heap.pop(); 
        let Some(mut p) = p else {
            return None 
        }; 
        swap(&mut p, &mut self.max_heap[0]); 
        self.max_heap[0].borrow_mut().maximum_index = 0; 
        let min_index = p.borrow().minimum_index; 
        self.min_heap.swap_remove(min_index); 
        let is_ok = self.min_heap.get_mut(min_index).map(|a| a.borrow_mut()).map(|mut a| a.minimum_index = min_index).is_some(); 
        self.bubble_down::<false>(0); 
        is_ok.then_some(|| {
            self.bubble_down::<true>(min_index);
        }); 
        let r = Rc::try_unwrap(p).ok().unwrap(); 
        let r = r.into_inner().value; 
        return Some(r) 
    } 
}

impl <T: Ord> BiHeap<T> {
    fn bubble_down<const IS_MIN: bool> (&mut self, index: usize) {
        if IS_MIN {
            assert!( index < self.min_heap.len() ); 
        } else {
            assert!( index < self.max_heap.len() );   
        }
        let heap = if IS_MIN { &mut self.min_heap } else { &mut self.max_heap }; 
        let accessor = if IS_MIN { minimin_index_mut } else { maximax_index_mut }; 
        let slice = &mut heap[..]; 
        let mut index = index; 
        loop {
            let lindex = 2 * index + 1; 
            if lindex >= slice.len() {
                break; 
            } 
            let (parent, children) = slice.split_at_mut(lindex); 
            let (parent, children) = (&mut parent[index], &mut children[..]); 
            let (left, children) = children.split_first_mut().unwrap(); 
            let right = children.split_first_mut().map(|a| a.0); 
            let mut swap_index = lindex; 
            let mut swap_node = left; 
            if let Some(right) = right {
                let cmp = Ord::cmp(&swap_node.borrow().value, &right.borrow().value); 
                match cmp {
                    Ordering::Less => {
                        if !IS_MIN {
                            swap_index = lindex + 1; 
                            swap_node = right; 
                        }
                    }
                    Ordering::Greater => {
                        if IS_MIN {
                            swap_index = lindex + 1; 
                            swap_node = right; 
                        }
                    }
                    Ordering::Equal => (), 
                }
            }
            let cmp = Ord::cmp(&parent.borrow().value, &swap_node.borrow().value); 
            match cmp {
                Ordering::Less => {
                    if !IS_MIN {
                        *accessor(&mut parent.borrow_mut()) = swap_index;  
                        *accessor(&mut swap_node.borrow_mut()) = index; 
                        swap(parent, swap_node); 
                    } else {
                        break; 
                    }
                }
                Ordering::Greater => {
                    if IS_MIN {
                        *accessor(&mut parent.borrow_mut()) = swap_index;  
                        *accessor(&mut swap_node.borrow_mut()) = index; 
                        swap(parent,  swap_node);  
                    } else {
                        break; 
                    }
                }
                Ordering::Equal => break, 
            }
            index = swap_index; 
        } 
    }
}

impl <T: Ord> BiHeap<T> {
    pub fn size(&self) -> usize {
        self.min_heap.len() 
    } 
}