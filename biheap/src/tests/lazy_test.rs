use super::*; 

#[test]
fn middle_pop() {
    let mut bh = BiHeap::new(); 
    bh.push(1); 
    bh.push(2); 
    let value2 = bh.max_indexer().unwrap(); 
    bh.push(3); 
    assert_eq!( bh.len(), 3 ); 
    let pop = bh.peek_mut(&value2).unwrap().pop(); 
    assert_eq!( pop, 2 ); 
    assert_eq!( bh.len(), 2 ); 
    assert_eq!( bh.pop_min().unwrap(), 1 ); 
    assert_eq!( bh.len(), 1 ); 
    assert_eq!( bh.pop_min().unwrap(), 3 ); 
    assert_eq!( bh.len(), 0 ); 
} 

#[test] 
fn loss_pop() {
    let mut bh = BiHeap::new(); 
    bh.push(1); 
    let value = bh.max_indexer().unwrap(); 
    drop(bh);  
    bh = BiHeap::new(); 
    bh.push(3); 
    let view_result = bh.peek(&value); 
    assert!(view_result.is_err());
}

#[test] 
fn missing_pop() {
    let mut bh = BiHeap::new(); 
    bh.push(1); 
    let h1 = bh.min_indexer().unwrap(); 
    let h2 = bh.max_indexer().unwrap(); 
    let val = bh.peek_mut(&h1).unwrap().pop(); 
    assert_eq!( val, 1 ); 
    let peek1 = bh.peek(&h1); 
    assert!( peek1.is_err() ); 
    let peek2 = bh.peek(&h2); 
    assert!( peek2.is_err() ); 
}