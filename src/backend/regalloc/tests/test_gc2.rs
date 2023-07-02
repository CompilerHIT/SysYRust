#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet, VecDeque};

    use biheap::BiHeap;
    #[test]
    fn test_get_ltzero() {
        let m: Vec<i32> = Vec::new();
        let g = m.get(0 - 1);
    }

    #[test]
    fn test_clone() {
        let mut m = HashMap::new();
        m.insert(1, 3);
        m.insert(2, 4);
        let mut cm = m.clone();
        assert_eq!(m.len(), cm.len());
        for (k, v) in m.iter() {
            let ck = cm.get(k).unwrap();
            assert_eq!(ck, v);
        }
        let mut m = HashSet::new();
        m.insert(33);
        m.insert(44);
        let cm = m.clone();
        assert!(m.len() == cm.len());
        for i in m.iter() {
            assert!(cm.contains(i));
        }
    }
}
