use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

// 一个简单的bitmap,用来统计spilling情况,用位图
pub struct Bitmap {
    arr: Vec<u64>,
    len: usize, //记录元素数量
}

impl Bitmap {
    pub fn andOther(&mut self, other: &Bitmap) {
        while other.cap() > self.arr.len() {
            self.arr.push(0);
        }
        let i = 0;
        while i < self.cap() && i < other.cap() {
            self.arr[i] = self.arr[i] & other.arr[i];
        }
    }
    pub fn orOther(&mut self, other: &Bitmap) {
        while other.cap() > self.arr.len() {
            self.arr.push(0);
        }
        let i = 0;
        while i < self.cap() && i < other.cap() {
            self.arr[i] = self.arr[i] | other.arr[i];
        }
    }

    //获取元素数量
    pub fn count(&self) -> usize {
        let mut out = 0;
        for v in &self.arr {
            let mut tmp = *v;
            for _ in 0..64 {
                if tmp % 2 == 1 {
                    out += 1
                }
                tmp /= 2;
            }
        }
        out
    }
}

//  编写单元测试检查
impl Bitmap {
    pub fn new() -> Bitmap {
        Bitmap {
            arr: Vec::new(),
            len: 0,
        }
    }
    pub fn with_cap(cap: usize) -> Bitmap {
        Bitmap {
            arr: Vec::with_capacity(cap),
            len: 0,
        }
    }
    pub fn len(&self) -> usize {
        return self.len;
    }
    pub fn insert(&mut self, i: usize) {
        if !self.inline_contains(i) {
            self.len += 1;
        }
        while i / 64 >= self.arr.len() {
            self.arr.push(0);
        }
        let v = &mut self.arr[i / 64];
        *v = *v | (1 << (i as u64 % 64))
    }
    pub fn remove(&mut self, i: usize) -> bool {
        if self.inline_contains(i) {
            self.len -= 1;
        }
        if i / 64 >= self.arr.len() {
            return false;
        }
        let v = &mut self.arr[i / 64];
        if *v & (1 << (i as u64 % 64)) == 0 {
            return false;
        }
        *v = *v & (!(1 << (i as u64 % 64)));
        true
    }

    #[inline]
    fn inline_contains(&self, i: usize) -> bool {
        if i / 64 >= self.arr.len() {
            return false;
        }
        let v = self.arr[i / 64];
        if v & (1 << (i as u64 % 64)) == 0 {
            return false;
        }
        true
    }

    pub fn contains(&self, i: usize) -> bool {
        if i / 64 >= self.arr.len() {
            return false;
        }
        let v = self.arr[i / 64];
        if v & (1 << (i as u64 % 64)) == 0 {
            return false;
        }
        true
    }

    pub fn cap(&self) -> usize {
        self.arr.len()
    }

    pub fn and(a: &Bitmap, b: &Bitmap) -> Bitmap {
        let mut i: usize = a.cap();
        if i < b.cap() {
            i = b.cap();
        }
        let mut out = Bitmap::with_cap(i);
        i = 0;
        while i < a.cap() && i < b.cap() {
            out.arr[i] = a.arr[i] & b.arr[i];
            i += 1;
        }
        out
    }
    pub fn or(a: &Bitmap, b: &Bitmap) -> Bitmap {
        let mut i: usize = a.cap();
        if i < b.cap() {
            i = b.cap();
        }
        let mut out = Bitmap::with_cap(i);
        i = 0;
        while i < a.cap() && i < b.cap() {
            out.arr[i] = a.arr[i] & b.arr[i];
        }
        out
    }
}

// 为Bitmap实现Hash 和Eq trait ,等待测试
impl Hash for Bitmap {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        self.arr.hash(state);
    }
}

impl Eq for Bitmap {
    fn assert_receiver_is_total_eq(&self) {}
}
impl PartialEq for Bitmap {
    fn eq(&self, other: &Self) -> bool {
        if self.cap() != other.cap() {
            false
        } else {
            for i in 0..self.cap() {
                let v1 = self.arr[i];
                let v2 = other.arr[i];
                if v1 != v2 {
                    return false;
                }
            }
            true
        }
    }
}

#[cfg(test)]
mod test_bitmap {
    use std::collections::HashSet;

    use rand::random;

    use super::Bitmap;
    #[test]
    fn test_insert() {
        let mut bitmap = Bitmap::new();
        println!("ss");
        bitmap.insert(33);
        assert!(bitmap.contains(33));
    }

    #[test]
    fn test_remove() {
        let mut bitmap = Bitmap::new();
        bitmap.insert(33);
        assert!(bitmap.contains(33));
        bitmap.remove(33);
        assert!(!bitmap.contains(33));
    }

    #[test]
    fn test_len() {
        // TODO,检查len
        for j in 0..=100000 {
            let n = 100;
            let mode: usize = 10000;
            let mut set: HashSet<usize> = HashSet::new();
            let mut bitmap = Bitmap::new();
            for i in 0..n {
                // 获取一个随机数
                let use_val = rand::random::<usize>() % mode;
                let insert_or = rand::random::<bool>();
                let delete_or = rand::random::<bool>();
                if insert_or {
                    set.insert(use_val);
                    bitmap.insert(use_val);
                }
                if delete_or {
                    assert!(set.remove(&use_val) == bitmap.remove(use_val));
                }
            }
            assert_eq!(set.len(), bitmap.len());
        }
    }

    #[test]
    fn test_use() {
        // 创建一个bitmap,进行随机插入删除n次
        let n = 1000000;
        let mode: usize = 10000;
        let mut set: HashSet<usize> = HashSet::new();
        let mut bitmap = Bitmap::new();
        for i in 0..n {
            // 获取一个随机数
            let use_val = rand::random::<usize>() % mode;
            let insert_or = rand::random::<bool>();
            let delete_or = rand::random::<bool>();
            if insert_or {
                set.insert(use_val);
                bitmap.insert(use_val);
            }
            if delete_or {
                assert!(set.remove(&use_val) == bitmap.remove(use_val));
            }
        }

        // 然后判断它与HashSet的随机插入删除判断结果是否一致
        for value in set.iter() {
            assert!(bitmap.contains(*value));
        }
    }

    #[test]
    fn test_eq() {
        let mut bp1 = Bitmap::new();
        let mut bp2 = Bitmap::new();
        bp1.insert(33);
        bp2.insert(33);
        let mut set = HashSet::new();
        set.insert(bp1);
        assert!(set.contains(&bp2));
    }

    #[test]
    fn test_hash() {
        //TODo
    }
}
