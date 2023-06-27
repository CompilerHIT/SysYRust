use super::*; 

use std::fmt::Debug;

impl <T: Ord + Debug> Debug for BiHeap<T> {
    /// Unimplemneted 
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_tuple("BiHeap").field(&self.0).finish()
        // unimplemented!()
        write!(f, "BiHeap {{ ...(unimplemented) }}")
    }
}