use super::BiVec;

impl <T> BiVec<T> {
    pub fn views_mut(&mut self) -> [&mut [T]; 2] {
        let slice1 = unsafe { std::slice::from_raw_parts_mut(self.contents[0] as *mut T, self.len) }; 
        let slice2 = unsafe { std::slice::from_raw_parts_mut(self.contents[1] as *mut T, self.len) }; 
        [slice1, slice2] 
    }
    pub fn views(&self) -> [&[T]; 2] {
        let slice1 = unsafe { std::slice::from_raw_parts(self.contents[0], self.len) }; 
        let slice2 = unsafe { std::slice::from_raw_parts(self.contents[1], self.len) }; 
        [slice1, slice2]  
    }
} 