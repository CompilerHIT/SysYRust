use std::cmp::Ordering;

pub fn min_heapize <T, F: FnMut(&T, &T) -> Ordering> (ar: &mut [T], mut f: F) {
    if std::mem::size_of::<T>() == 0 {
        return; 
    }
    let len = ar.len(); 
    if len < 2 {
        return; 
    }
    let mut i = len; 
    loop {
        i -= 1; 
        let mut j = i; 
        while j < len {
            let mut k = j * 2 + 1; 
            if k >= len {
                break; 
            }  
            if k + 1 < len && f(&ar[k], &ar[k + 1]) != Ordering::Less {
                k += 1; 
            } 
            if f(&ar[j], &ar[k]) == Ordering::Greater {
                ar.swap(j, k); 
                j = k; 
            } else {
                break; 
            }
        }
        if i == 0 {
            break 
        }
    }
} 