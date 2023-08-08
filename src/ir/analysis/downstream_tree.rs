use std::collections::{HashSet, HashMap, VecDeque};

use crate::{utility::ObjPtr, ir::{basicblock::BasicBlock, tools::bfs_bb_proceess}};

use super::dominator_tree::{self, DominatorTree};

struct DownStreamNode{
    down_stream_nodes:HashSet<ObjPtr<BasicBlock>>,
}

pub struct DownStreamTree{
    node_map:HashMap<ObjPtr<BasicBlock>,HashSet<ObjPtr<BasicBlock>>>
}

// impl DownStreamNode{
//     pub fn get_downstream(bb:ObjPtr<BasicBlock>,dominator_tree:& DominatorTree)->DownStreamNode{
//         let mut node = DownStreamNode { down_stream_nodes: HashSet::new() };
//         let mut vec_endpoint = vec![];
//         let mut visited = HashSet::new();
//         bfs_bb_proceess(bb, |bb_temp| {
//             for up in bb.get_up_bb(){
//                 if dominator_tree.is_dominate(&bb, up){
//                     vec_endpoint.push(*up);
//                     up.remove_next_bb(bb);
//                 }
//             }
//             node.down_stream_nodes.insert(bb_temp);
//             visited.insert(bb_temp);
//         });
//         node.down_stream_nodes.remove(&bb);
//         node
//     }
// }

impl DownStreamTree{
    pub fn make_downstream_tree(head:ObjPtr<BasicBlock>,dominator_tree:&DominatorTree)->DownStreamTree{
        let mut tree = DownStreamTree{node_map:HashMap::new()};
        tree.get_downstream(head, dominator_tree);
        tree
    }
    pub fn is_upstream(&self,a:ObjPtr<BasicBlock>,b:ObjPtr<BasicBlock>)->bool{
        self.node_map.get(&a).unwrap().contains(&b)
    }
    pub fn get_downstream(&mut self,head:ObjPtr<BasicBlock>,dominator_tree:& DominatorTree)->HashSet<ObjPtr<BasicBlock>>{
        let mut set = HashSet::new();
        let ups = head.get_up_bb();
        let mut vec_endpoint = vec![];
        let mut vec_endpoint_next = vec![];
        for up in ups{
            if dominator_tree.is_dominate(&head, up){
                let mut next_vec = up.get_next_bb().clone(); 
                vec_endpoint.push(*up);
                vec_endpoint_next.push(next_vec.clone());
                let index = next_vec.iter().position(|x|*x==head).unwrap();
                next_vec.remove(index);
                up.as_mut().set_next_bb(next_vec);
            }
        }
        for next in head.get_next_bb(){
            set.insert(*next);
            set.extend(self.get_downstream(*next,dominator_tree));
        }
        let mut index_endpoint = 0;
        for endpoint in vec_endpoint{
            endpoint.as_mut().set_next_bb(vec_endpoint_next[index_endpoint].clone());
            index_endpoint +=1;
        }
        self.node_map.insert(head, set.clone());
        set
    }
}