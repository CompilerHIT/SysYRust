use super::*; 

#[test] 
fn empty() {
    let heap: BiHeap<i32> = BiHeap::new(); 
    assert_eq!(heap.len(), 0); 
    heap.check(); 
} 

#[test] 
fn one() {
    let mut heap = BiHeap::new(); 
    heap.check();
    heap.push(1); 
    assert_eq!(heap.len(), 1);  
    heap.check();
}

#[test] 
fn two() {
    let mut heap = BiHeap::new(); 
    heap.check();
    heap.push(1); 
    heap.check();
    heap.push(2); 
    heap.check();
    assert_eq!(heap.len(), 2); 
}

#[test] 
fn three() {
    let mut heap = BiHeap::new(); 
    heap.push(1); 
    heap.push(2); 
    heap.push(3); 
    assert_eq!(heap.len(), 3); 
    // pop max 
    heap.pop_max(); 
    assert_eq!(heap.len(), 2); 
    // pop min 
    heap.pop_min(); 
    assert_eq!(heap.len(), 1); 
    // pop max 
    heap.pop_max(); 
    assert_eq!(heap.len(), 0); 
    let p = heap.pop_min(); 
    assert!(p.is_none());
    assert_eq!(heap.len(), 0); 
} 