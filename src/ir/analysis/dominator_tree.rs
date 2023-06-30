use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use crate::{
    ir::{basicblock::BasicBlock, tools::bfs_bb_proceess},
    utility::{ObjPool, ObjPtr},
};

pub struct DominatorTree {
    pool: ObjPool<DominatorNode>,
    dominatee: HashMap<ObjPtr<BasicBlock>, HashSet<ObjPtr<BasicBlock>>>,
    head_node: ObjPtr<DominatorNode>,
}

struct DominatorNode {
    bb: ObjPtr<BasicBlock>,
    dominatee: Vec<ObjPtr<DominatorNode>>,
}

impl Debug for DominatorNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s += &format!("{}: ", self.bb.get_name());
        for i in self.dominatee.iter() {
            s += &format!("{} ", i.bb.get_name());
        }
        s += "\n";
        write!(f, "{}", s)
    }
}

impl Debug for DominatorTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        let mut queue = vec![&self.head_node];
        while let Some(node) = queue.pop() {
            s += &format!("{:?}", node);
            queue.extend(&node.dominatee);
        }
        write!(f, "{}", s)
    }
}

impl DominatorTree {
    fn new(dominatee: HashMap<ObjPtr<BasicBlock>, HashSet<ObjPtr<BasicBlock>>>) -> Self {
        let mut pool = ObjPool::new();
        let tree = Self::make_tree(&mut pool, &dominatee);
        Self {
            pool,
            dominatee,
            head_node: tree,
        }
    }

    fn make_tree(
        pool: &mut ObjPool<DominatorNode>,
        dominatee: &HashMap<ObjPtr<BasicBlock>, HashSet<ObjPtr<BasicBlock>>>,
    ) -> ObjPtr<DominatorNode> {
        // 构造支配结点
        let node_list: Vec<ObjPtr<DominatorNode>> = dominatee
            .keys()
            .cloned()
            .map(|x| pool.put(DominatorNode::new(x)))
            .collect();

        // 找到头结点
        let head = node_list.iter().find(|&x| x.bb.is_entry()).unwrap().clone();

        // 构造直接支配边
        // 支配树中的边都是直接支配边，而直接支配的定义是：
        // 在支配x的结点中，结点y支配x且被其余结点支配
        for (domee, domer_set) in dominatee.iter() {
            // 跳过头结点
            if domee.is_entry() {
                continue;
            }

            let idom = domer_set
                .iter()
                .find(|bb| {
                    domer_set.iter().all(|bb2| {
                        if &bb2 == bb {
                            true
                        } else {
                            dominatee.get(bb).unwrap().contains(bb2)
                        }
                    })
                })
                .unwrap()
                .clone();
            debug_assert_ne!(domee.clone(), idom);
            let idom_node = node_list.iter().find(|x| x.bb == idom).unwrap();
            let domee_node = node_list.iter().find(|x| x.bb == domee.clone()).unwrap();
            idom_node.as_mut().dominatee.push(domee_node.clone());
        }

        head
    }

    /// 返回true如果a支配b
    pub fn is_dominate(&self, a: ObjPtr<BasicBlock>, b: ObjPtr<BasicBlock>) -> bool {
        self.dominatee.get(&b).unwrap().contains(&a)
    }

    /// 深度后序遍历支配树
    pub fn iter_post_order(&self, mut predicate: impl FnMut(ObjPtr<BasicBlock>)) {
        let mut queue = vec![&self.head_node];
        let mut visited = HashSet::new();
        while let Some(node) = queue.pop() {
            if node.dominatee.is_empty() || node.dominatee.iter().all(|x| visited.contains(x)) {
                predicate(node.bb);
                visited.insert(node);
            } else {
                queue.push(node);
                queue.extend(&node.dominatee);
            }
        }
    }
}

impl DominatorNode {
    fn new(bb: ObjPtr<BasicBlock>) -> Self {
        Self {
            bb,
            dominatee: Vec::new(),
        }
    }
}

pub fn calculate_dominator(head_bb: ObjPtr<BasicBlock>) -> DominatorTree {
    let mut dominatee = HashMap::new();

    let mut all_set = HashSet::new();
    bfs_bb_proceess(head_bb, |bb| {
        all_set.insert(bb);
    });
    all_set.iter().for_each(|bb| {
        dominatee.insert(bb.clone(), all_set.clone());
    });
    dominatee.insert(head_bb.clone(), vec![head_bb].iter().cloned().collect());

    loop {
        let mut changed = false;
        bfs_bb_proceess(head_bb, |bb| {
            if !bb.is_entry() {
                let mut new_dominatee = dominatee.get(&bb.get_up_bb()[0]).cloned().unwrap();
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

    for (k, v) in dominatee.iter_mut() {
        v.remove(k);
    }

    DominatorTree::new(dominatee)
}
