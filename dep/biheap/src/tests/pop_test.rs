use super::*; 

#[test] 
fn directly_pop() {
    let mut bh = BiHeap::<i32>::new(); 
    let max = bh.pop_max(); 
    assert_eq!(max, None); 
    let min = bh.pop_min(); 
    assert_eq!(min, None); 
}

#[test] 
fn directly_pop2() {
    let mut bh = BiHeap::<u32>::new(); 
    let min = bh.pop_min(); 
    assert_eq!(min, None); 
    let max = bh.pop_max(); 
    assert_eq!(max, None); 
}

#[test] 
fn loop_push() {
    let mut bh = BiHeap::<i32>::new(); 
    for i in 0..100 {
        bh.push(i); 
    } 
    for i in 0..100 {
        let min = bh.pop_min(); 
        assert_eq!(min, Some(i)); 
    } 
    let min = bh.pop_min(); 
    assert_eq!(min, None); 
}

#[test] 
fn loop_push2() {
    let mut bh = BiHeap::<usize>::new(); 
    for i in 0..100 {
        bh.push(i); 
    } 
    for i in (0..100).rev() {
        let max = bh.pop_max(); 
        assert_eq!(max, Some(i)); 
    } 
    let max = bh.pop_max(); 
    assert_eq!(max, None); 
}