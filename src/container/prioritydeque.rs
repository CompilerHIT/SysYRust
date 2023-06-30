// 一个双端优先队列
// 暂时先是模拟实现

use std::cmp::Ordering;

pub struct PriorityDeque<T: Ord> {
    arr: Vec<T>,
}

// 默认最小堆实现
impl<T> PriorityDeque<T>
where
    T: Ord,
{
    pub fn new() -> PriorityDeque<T> {
        PriorityDeque { arr: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.arr.len()
    }

    // 交换两个位置的元素
    fn swap(&mut self, i: usize, j: usize) {
        self.arr.swap(i, j)
    }

    pub fn pop_front(&mut self) -> Option<T> {
        // 找到最小值返回
        if self.arr.len() == 0 {
            None
        } else {
            Some(self.arr.remove(0))
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.arr.len() == 0 {
            None
        } else {
            Some(self.arr.remove(self.arr.len() - 1))
        }
    }

    pub fn front(&self) -> Option<&T> {
        if self.arr.len() == 0 {
            None
        } else {
            Some(&self.arr[0])
        }
    }

    pub fn back(&self) -> Option<&T> {
        if self.arr.len() == 0 {
            None
        } else {
            Some(&self.arr[self.arr.len() - 1])
        }
    }

    pub fn push(&mut self, v: T) {
        // find index then insert
        let mut index = 0;
        for (i, v) in self.arr.iter().enumerate() {
            match v.cmp(v) {
                Ordering::Less => (),
                Ordering::Equal => break,
                Ordering::Greater => break,
            }
            index += 1;
        }
        self.arr.insert(index, v);
    }
}

#[cfg(test)]
mod test_biheap {
    use std::collections::VecDeque;

    extern crate biheap;
    use biheap::core::BiHeap;
    #[test]
    fn test_biheap() {
        // let tmp=VecDeque::new();
        // tmp.remove(22);
        let mut m = BiHeap::new();
        m.push(222);
        m.push(333);
        m.push(111);
        m.push(111);
        assert_eq!(111, *m.peek_min().unwrap());
        m.pop_min();
        assert_eq!(111, *m.peek_min().unwrap());
        m.pop_min();
        assert_eq!(222, *m.peek_min().unwrap());
        assert_eq!(333, *m.peek_max().unwrap());
    }
}
