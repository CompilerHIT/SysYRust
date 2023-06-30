use std::cell::UnsafeCell;

use crate::bivec::BiVec;

pub struct BiHeap <T: Ord> (pub(crate) Rc<UnsafeCell<BiVec<Rc<UnsafeCell<Node<T>>>>>>); 

#[cfg(not(feature = "threadsafe"))]
type Rc<T> = std::rc::Rc<T>; 
#[cfg(not(feature = "threadsafe"))]
type Weak<T> = std::rc::Weak<T>; 

#[cfg(feature = "threadsafe")] 
type Rc<T> = std::sync::Arc<T>; 
#[cfg(feature = "threadsafe")] 
type Weak<T> = std::sync::Weak<T>; 

pub struct Node <T> {
    pub(crate) value: T, 
    pub(crate) min_index: usize, 
    pub(crate) max_index: usize, 
}

pub type Handle<T> = Indexer<T>; 

pub struct Indexer <T> {
    pub(crate) node_ref: Weak<UnsafeCell<Node<T>>>, 
    pub(crate) heap_ref: Weak<UnsafeCell<BiVec<Rc<UnsafeCell<Node<T>>>>>>, 
}

pub struct PeekMut <'a, T: Ord> {
    pub(crate) bi_heap: &'a mut BiHeap<T>,
    // Actaully, it should always be Some, or in my consumming method it will be None, only. 
    pub(crate) node: Option<Rc<UnsafeCell<Node<T>>>>, 
}

pub type ViewMut<'a, T> = PeekMut<'a, T>; 

mod construct;
mod push;
mod bubble;
mod indexer;
mod view;
mod peek;
mod pop;
mod property;
mod clone;
mod debug;
mod default;
mod extend;
mod from;
mod iter;
mod into;

#[cfg(feature = "threadsafe")]
unsafe impl <T> Sync for BiHeap<T> where T: Ord + Sync {} 
#[cfg(feature = "threadsafe")]
unsafe impl <T> Send for BiHeap<T> where T: Ord + Send {} 