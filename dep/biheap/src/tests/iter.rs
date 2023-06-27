use super::*; 

#[test] 
fn test_iter1() {
    let mut be = BiHeap::new(); 
    be.push(1); 
    be.push(2); 
    be.push(3); 
    let mut iter = be.iter(); 
    assert!(iter.next().is_some()); 
    assert!(iter.next().is_some()); 
    assert!(iter.next().is_some()); 
    assert_eq!(iter.next(), None);  
}

#[test] 
fn test_iter2() {
    let mut be = BiHeap::new(); 
    for i in 0..100 {
        be.push(i);  
    }
    for i in &be {
        if !(0..100).contains(i) {
            panic!("Iterator returned invalid value: {}", i);  
        }
    }
}

#[test] 
fn test_iter3() {
    let mut be = BiHeap::new(); 
    for i in 0..100 {
        be.push(i);  
    } 
    let iter = be.iter();
    let cnt = iter.count(); 
    assert_eq!(cnt, 100); 
}