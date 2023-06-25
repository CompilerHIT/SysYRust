use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{ir::basicblock::BasicBlock, utility::ObjPtr};

use super::bfs_bb_proceess;

pub struct DominatorTree {
    dominatee: HashMap<ObjPtr<BasicBlock>, HashSet<ObjPtr<BasicBlock>>>,
}

impl Debug for DominatorTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for (k, v) in self.dominatee.iter() {
            s += &format!("{}: ", k.get_name());
            for i in v.iter() {
                s += &format!("{} ", i.get_name());
            }
            s += "\n";
        }
        write!(f, "{}", s)
    }
}

impl DominatorTree {
    fn new(dominatee: HashMap<ObjPtr<BasicBlock>, HashSet<ObjPtr<BasicBlock>>>) -> Self {
        DominatorTree { dominatee }
    }
}

pub fn calculate_dominator(head_bb: ObjPtr<BasicBlock>) -> DominatorTree {
    let mut dominatee = HashMap::new();

    let mut all_set = HashSet::new();
    bfs_bb_proceess(head_bb, |bb| {
        all_set.insert(bb);
    });
    all_set.iter().for_each(|bb| {
        dominatee.insert(bb.clone(), HashSet::new());
    });
    dominatee.insert(head_bb.clone(), [head_bb.clone()].iter().cloned().collect());

    loop {
        let mut changed = false;
        bfs_bb_proceess(head_bb, |bb| {
            if !bb.is_entry() {
                let mut new_dominatee = dominatee.get(&bb.get_up_bb()[0]).unwrap().clone();
                bb.get_up_bb().iter().for_each(|bb| {
                    new_dominatee = new_dominatee
                        .intersection(dominatee.get(bb).unwrap())
                        .cloned()
                        .collect();
                });
                new_dominatee.insert(bb.clone());

                if new_dominatee != dominatee.get(&bb).unwrap().clone() {
                    changed = true;
                    dominatee.insert(bb.clone(), new_dominatee);
                }
            }
        });
        if !changed {
            break;
        }
    }

    DominatorTree::new(dominatee)
}

#[test]
fn dominator_test() {
    let mut bb_v = Vec::new();
    for i in 0..=7 {
        let bb = BasicBlock::new(i.to_string());
        bb_v.push(ObjPtr::new(&bb));
    }

    bb_v[0].add_next_bb(bb_v[1].clone());
    bb_v[0].add_next_bb(bb_v[2].clone());
    bb_v[1].add_next_bb(bb_v[3].clone());
    bb_v[2].add_next_bb(bb_v[1].clone());
    bb_v[2].add_next_bb(bb_v[4].clone());
    bb_v[3].add_next_bb(bb_v[2].clone());
    bb_v[3].add_next_bb(bb_v[5].clone());
    bb_v[4].add_next_bb(bb_v[6].clone());
    bb_v[5].add_next_bb(bb_v[6].clone());

    let dom_tree = calculate_dominator(bb_v[0].clone());
    crate::log!("dom tree: {:?}", dom_tree);
}
