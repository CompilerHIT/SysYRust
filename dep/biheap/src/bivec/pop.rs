use super::BiVec;

impl <T> BiVec<T> {
    /// pop the last element of the BiVec 
    pub fn pop(&mut self) -> Option<(T, T)> {
        if self.len == 0 {
            return None; 
        }
        let (a, b); 
        self.len -= 1; 
        if std::mem::size_of::<T>() == 0 {
            a = unsafe { std::mem::zeroed() }; 
            b = unsafe { std::mem::zeroed() }; 
            return Some((a, b)); 
        } 
        unsafe {
            a = self.contents[0].add(self.len).read(); 
            b = self.contents[1].add(self.len).read(); 
        } 
        Some((a, b)) 
    } 
}

impl <T> BiVec<T> {
    pub fn swap_remove(&mut self, first_index: usize, second_index: usize) -> Option<(T, T)> {
        if first_index >= self.len || second_index >= self.len {
            return None; 
        }
        if std::mem::size_of::<T>() == 0 {
            return self.pop();
        } 
        let (p1, p2); 
        let (a, b); 
        self.len -= 1; 
        unsafe {
            p1 = self.contents[0].add(first_index) as *mut T; 
            a = p1.read(); 
            if first_index < self.len {
                p1.write( self.contents[0].add(self.len).read() ); 
            }
            p2 = self.contents[1].add(second_index) as *mut T; 
            b = p2.read(); 
            if second_index < self.len {
                p2.write( self.contents[1].add(self.len).read() ); 
            }
        }
        Some((a, b)) 
    } 
}

#[test] 
fn just_pop() {
    let mut bivec : BiVec<i32> = BiVec::new(); 
    let p = bivec.pop(); 
    assert_eq!(p, None); 
}

#[test] 
fn pop_one() {
    let mut bivec : BiVec<String> = BiVec::new(); 
    bivec.push("hello".to_string(), "world".to_string()); 
    let p = bivec.pop(); 
    let p = p.as_ref().map(|(a, b)| (a.as_str(), b.as_str())); 
    assert_eq!(p, Some(("hello", "world"))); 
}

#[test] 
fn pop_all() {
    let mut bivec : BiVec<f32> = BiVec::new(); 
    for i in 0..52 {
        bivec.push(i as f32, (i + 1) as f32);
    }
    for i in 0..52 {
        let p = bivec.pop(); 
        let p = p.as_ref().map(|(a, b)| (*a, *b)); 
        assert_eq!(p, Some((51. - i as f32, 52. - i as f32))); 
    } 
    let p = bivec.pop(); 
    assert_eq!(p, None); 
}

#[test] 
fn swap_last() {
    let mut bivec = BiVec::new(); 
    bivec.push(1, 2); 
    let rm = bivec.swap_remove(0, 0);
    assert_eq!(rm, Some((1, 2))); 
    assert_eq!(bivec.len(), 0); 
    let p = bivec.pop(); 
    assert_eq!(p, None); 
}

#[test] 
fn swap_first() {
    let mut bivec = BiVec::new(); 
    bivec.push(1, 2); 
    bivec.push(3, 4); 
    let rm = bivec.swap_remove(0, 0); 
    assert_eq!(rm, Some((1, 2))); 
    assert_eq!(bivec.len(), 1); 
    let p = bivec.pop(); 
    assert_eq!(p, Some((3, 4))); 
    let p = bivec.pop(); 
    assert_eq!(p, None); 
}