use std::collections::{HashSet, HashMap};

use crate::{utility::ObjPtr, ir::{basicblock::BasicBlock, tools::bfs_bb_proceess}};

struct DownStreamNode{
    down_stream_nodes:HashSet<ObjPtr<BasicBlock>>,
}

pub struct DownStreamTree{
    node_map:HashMap<ObjPtr<BasicBlock>,DownStreamNode>
}

impl DownStreamNode{
    pub fn get_downstream(bb:ObjPtr<BasicBlock>)->DownStreamNode{
        let mut node = DownStreamNode { down_stream_nodes: HashSet::new() };
        bfs_bb_proceess(bb, |bb_temp| {
            node.down_stream_nodes.insert(bb_temp);
        });
        node.down_stream_nodes.remove(&bb);
        node
    }
}

impl DownStreamTree{
    pub fn make_downstream_tree(head:ObjPtr<BasicBlock>)->DownStreamTree{
        let mut tree = DownStreamTree{node_map:HashMap::new()};
        bfs_bb_proceess(head, |bb| {
            tree.node_map.insert(bb, DownStreamNode::get_downstream(bb));
        });
        tree
    }
    pub fn is_upstream(&self,a:ObjPtr<BasicBlock>,b:ObjPtr<BasicBlock>)->bool{
        self.node_map.get(&a).unwrap().down_stream_nodes.contains(&b)
    }
}