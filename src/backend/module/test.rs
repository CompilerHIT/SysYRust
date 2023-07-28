use super::*;
#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    #[test]
    pub fn test_hash() {
        let mut set = HashSet::new();
        for i in 0..=10000000 {
            let if_insert: bool = rand::random();
            if if_insert {
                set.insert(i);
            }
        }
        let set2 = set.clone();
        assert!(set.len() == set2.len());
        for v in set.iter() {
            assert!(set2.contains(v));
        }
    }

    //该测试表明HashMap的clone也是深clone
    #[test]
    pub fn test_hash_map() {
        let mut set = HashMap::new();
        for i in 0..=100000 {
            let if_insert: bool = rand::random();
            if if_insert {
                let vb: bool = rand::random();
                set.insert(i, vb);
            }
        }
        let set2 = set.clone();
        assert!(set.len() == set2.len());
        for (k, v) in set.iter() {
            assert!(set2.contains_key(k) && set2.get(k).unwrap() == v);
        }
    }
}
