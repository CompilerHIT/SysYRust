use super::*; 
use crate::bivec::BiVec;

#[test] 
fn zero_vec1() {
    let mut v = BiVec::new();
    v.push((), ()); 
    v.push((), ()); 
    assert_eq!(v.len(), 2);
    assert_eq!(v.pop(), Some(((), ()))); 
    assert_eq!(v.len(), 1);
    assert_eq!(v.pop(), Some(((), ()))); 
    assert_eq!(v.len(), 0); 
    assert_eq!(v.pop(), None); 
}

pub mod counter {
    use std::sync::atomic::AtomicUsize;

    pub static COUNTER : AtomicUsize = AtomicUsize::new(0); 

    #[derive(PartialOrd, Ord, PartialEq, Eq)]
    pub struct Counter(()); 

    impl Counter {
        pub fn new() -> Self {
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst); 
            Self(())
        }
    }  

    impl Drop for Counter {
        fn drop(&mut self) {
            COUNTER.fetch_sub(1, std::sync::atomic::Ordering::SeqCst); 
        }
    } 

}

#[test]
fn zero_vec2() {
    let mut v = BiVec::new(); 
    v.push(counter::Counter::new(), counter::Counter::new()); 
    v.push(counter::Counter::new(), counter::Counter::new()); 
    assert_eq!(v.len(), 2); 
    assert_eq!(counter::COUNTER.load(std::sync::atomic::Ordering::SeqCst), 4); 
    drop(v) ;
    assert_eq!(counter::COUNTER.load(std::sync::atomic::Ordering::SeqCst), 0); 
}

#[test] 
fn zero_heap1() {
    let mut be = BiHeap::new(); 
    be.push(()); 
    be.push(());
    assert_eq!(be.len(), 2);
    assert_eq!(be.pop_max(), Some(()));
    assert_eq!(be.len(), 1);
    assert_eq!(be.pop_max(), Some(()));
    assert_eq!(be.len(), 0);
    assert_eq!(be.pop_max(), None);
}

#[test] 
fn zero_heap2() {
    let mut be = BiHeap::new(); 
    be.push(counter::Counter::new()); 
    be.push(counter::Counter::new()); 
    assert_eq!(be.len(), 2); 
    assert_eq!(counter::COUNTER.load(std::sync::atomic::Ordering::SeqCst), 2); 
    drop(be); 
    assert_eq!(counter::COUNTER.load(std::sync::atomic::Ordering::SeqCst), 0); 
}