use std::collections::{HashMap, HashSet};

use crate::{ir::basicblock::BasicBlock, utility::ObjPtr};

use super::bfs_bb_proceess;

pub struct DominatorTree {
    dominatee: HashMap<ObjPtr<BasicBlock>, HashSet<ObjPtr<BasicBlock>>>,
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
