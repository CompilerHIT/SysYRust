use super::*;

#[test]
fn thread_test1() {

    let mut be = BiHeap::new(); 
    be.push(1); 
    be.push(2); 
    be.push(3); 
    std::thread::spawn(move || {
        be.push(4); 
        be.push(5); 
        be.push(6); 
        let mut v2 = 1; 
        while let Some(v) = be.pop_min() {
            assert_eq!(v, v2); 
            v2 += 1;   
        } 
    });
}