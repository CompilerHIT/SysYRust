pub struct BiHeap <T: Ord> {
    min_heap: Vec<*mut Node<T>>, 
    max_heap: Vec<*mut Node<T>>, 
}

impl <T: Ord> BiHeap<T> {
    pub fn new() -> Self {
        BiHeap {
            min_heap: Vec::new(), 
            max_heap: Vec::new(), 
        }
    } 
    pub fn with_capacity(capacity: usize) -> Self {
        BiHeap {
            min_heap: Vec::with_capacity(capacity), 
            max_heap: Vec::with_capacity(capacity), 
        } 
    }
    pub fn push(&mut self, value: T) {
        let size = self.max_heap.len(); 
        let node = Box::new(Node {
            value: value, 
            minimum_index: size, 
            maximum_index: size, 
        });
        let node_ptr = Box::into_raw(node);
        self.min_heap.push(node_ptr);
        self.max_heap.push(node_ptr);
        self.bubble_up(size, true); 
        self.bubble_up(size, false); 
    } 
    pub fn bubble_up (&mut self, index: usize, min: bool) {
        let mut index = index;
        while index > 0 {
            let parent_index = (index - 1) / 2;
            let parent; 
            let node; 
            if min {
                let minheap = self.min_heap.as_mut_slice().split_at_mut(index); 
                parent = &mut minheap.0[parent_index]; 
                node = &mut minheap.1[0]; 
            } else {
                let maxheap = self.max_heap.as_mut_slice().split_at_mut(index); 
                parent = &mut maxheap.0[parent_index]; 
                node = &mut maxheap.1[0]; 
            }
            let parent_value = unsafe { &node.as_ref().unwrap_unchecked().value }; 
            let node_value = unsafe { &parent.as_ref().unwrap_unchecked().value }; 
            match Ord::cmp( parent_value, node_value) { 
                Ordering::Less => {
                    if min {
                        break; 
                    } else {
                        
                        index = parent_index; 
                    }
                }
                Ordering::Greater => {
                }, 
                Ordering::Equal => break, 
            }
            // if node.value < parent.value {
            //     self.swap(index, parent_index);
            //     index = parent_index;
            // } else {
            //     break;
            // }
        }
    } 
}
