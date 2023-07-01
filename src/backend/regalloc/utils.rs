use std::collections::{HashMap, VecDeque};

extern crate biheap;
use biheap::BiHeap;

use crate::backend;

// 根据nnc指定的(item,coe)的权重关系,来对items进行排序,并且把排序结果保存到out_come_order中
pub fn sort(nnc: &HashMap<i32, i32>, out_come_order: &mut Vec<i32>) {
    // 排序过程,首先建桶 (出现次数, 颜色列表)
    let mut buckets: HashMap<i32, VecDeque<i32>> = HashMap::new();
    let mut sort_heap: BiHeap<i32> = BiHeap::new();
    // 遍历所有的颜色,取出所有出现次数和颜色列表，并且对出现次数进行排序
    for (color, times) in nnc.iter() {
        if !buckets.contains_key(times) {
            buckets.insert(*times, VecDeque::with_capacity(3));
        }
        let bucket = buckets.get_mut(times).unwrap();
        bucket.push_back(*color);
        sort_heap.push(*times);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, VecDeque};

    use biheap::BiHeap;

    #[test]
    fn test_sort() {
        // 测试一个编写的对颜色排序的操作
        let mut m: HashMap<i32, i32> = HashMap::new();
        m.insert(2, 5);
        m.insert(3, 11);
        m.insert(1, 3);
        let sort = |nnc: &HashMap<i32, i32>, out_come_order: &mut Vec<i32>| {
            // 排序过程,首先建桶 (出现次数, 颜色列表)
            let mut buckets: HashMap<i32, VecDeque<i32>> = HashMap::new();
            let mut sort_heap: BiHeap<i32> = BiHeap::new();
            // 遍历所有的颜色,取出所有出现次数和颜色列表，并且对出现次数进行排序
            for (color, times) in nnc.iter() {
                if !buckets.contains_key(times) {
                    buckets.insert(*times, VecDeque::with_capacity(3));
                }
                let bucket = buckets.get_mut(times).unwrap();
                bucket.push_back(*color);
                sort_heap.push(*times);
            }
            //
            while !sort_heap.is_empty() {
                let times = sort_heap.pop_min().unwrap();
                let color = buckets.get_mut(&times).unwrap().pop_front().unwrap();
                out_come_order.push(color);
            }
        };
        let mut order = Vec::new();
        sort(&m, &mut order);
        let to_cmp = Vec::from([1, 2, 3]);
        for i in 0..order.len() {
            assert_eq!(order.get(i).unwrap(), to_cmp.get(i).unwrap());
        }
    }
    #[test]
    fn test_deref_mut_i32() {
        let mut a = 33;
        let b = &mut a;
        *b += 1;
        assert!(a == 34);
    }
}
