use std::collections::{HashMap, HashSet};

use crate::{ir::basicblock::BasicBlock, utility::ObjPtr};

use super::dominator_tree::DominatorTree;

pub struct DownStreamTree {
    node_map: HashMap<ObjPtr<BasicBlock>, HashSet<ObjPtr<BasicBlock>>>,
}

impl DownStreamTree {
    pub fn make_downstream_tree(
        head: ObjPtr<BasicBlock>,
        dominator_tree: &DominatorTree,
    ) -> DownStreamTree {
        let mut tree = DownStreamTree {
            node_map: HashMap::new(),
        };
        tree.get_downstream(head, dominator_tree);
        tree
    }
    pub fn is_upstream(&self, a: ObjPtr<BasicBlock>, b: ObjPtr<BasicBlock>) -> bool {
        self.node_map.get(&a).unwrap().contains(&b)
    }
    pub fn get_downstream(
        &mut self,
        head: ObjPtr<BasicBlock>,
        dominator_tree: &DominatorTree,
    ) -> HashSet<ObjPtr<BasicBlock>> {
        let mut set = HashSet::new();
        let ups = head.get_up_bb();
        let mut vec_endpoint = vec![];
        let mut vec_endpoint_next = vec![];
        for up in ups.clone() {
            if dominator_tree.is_dominate(&head, &up) {
                let mut next_vec = up.get_next_bb().clone();
                vec_endpoint.push(up);
                vec_endpoint_next.push(next_vec.clone());
                let index = next_vec.iter().position(|x| *x == head).unwrap();
                next_vec.remove(index);
                up.as_mut().set_next_bb(next_vec);
            }
        }
        for next in head.get_next_bb() {
            set.insert(*next);
            if let Some(set_temp) = self.node_map.get(next) {
                set.extend(set_temp);
            } else {
                set.extend(self.get_downstream(*next, dominator_tree));
            }
        }
        let mut index_endpoint = 0;
        for endpoint in vec_endpoint {
            endpoint
                .as_mut()
                .set_next_bb(vec_endpoint_next[index_endpoint].clone());
            index_endpoint += 1;
        }
        self.node_map.insert(head, set.clone());
        set
    }
}
