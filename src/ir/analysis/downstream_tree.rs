use std::collections::{HashSet, HashMap, VecDeque};

use crate::{utility::ObjPtr, ir::{basicblock::BasicBlock, tools::bfs_bb_proceess}};

use super::dominator_tree::{self, DominatorTree};

struct DownStreamNode{
    down_stream_nodes:HashSet<ObjPtr<BasicBlock>>,
}

pub struct DownStreamTree{
    node_map:HashMap<ObjPtr<BasicBlock>,HashSet<ObjPtr<BasicBlock>>>
}


impl DownStreamTree{
    pub fn make_downstream_tree(head:ObjPtr<BasicBlock>,dominator_tree:&DominatorTree)->DownStreamTree{
        let mut tree = DownStreamTree{node_map:HashMap::new()};
        tree.get_downstream(head, dominator_tree);
        // println!("一次计算结束");
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
        for up in ups.clone(){
            if dominator_tree.is_dominate(&head, &up){
                let mut next_vec = up.get_next_bb().clone(); 
                vec_endpoint.push(up);
                vec_endpoint_next.push(next_vec.clone());
                let index = next_vec.iter().position(|x|*x==head).unwrap();
                // println!("断回边bb{:?},断前有",up.get_name());

                for nn in up.get_next_bb().clone(){
                    println!("{:?}",nn.get_name());
                }
                next_vec.remove(index);
                up.as_mut().set_next_bb(next_vec);
                // println!("现有nxt");
                for nn in up.get_next_bb().clone(){
                    println!("{:?}",nn.get_name());
                }
            }
        }
        for next in head.get_next_bb(){
            set.insert(*next);
            if let Some(set_temp) = self.node_map.get(next){
                set.extend(set_temp);
            }else{
                set.extend(self.get_downstream(*next,dominator_tree));
            }
        }
        let mut index_endpoint = 0;
        for endpoint in vec_endpoint{
            endpoint.as_mut().set_next_bb(vec_endpoint_next[index_endpoint].clone());
            // println!("连回边bb{:?}",endpoint.get_name());
            // println!("现有nxt");
            for nn in endpoint.get_next_bb(){
                println!("{:?}",nn.get_name());
            }
            index_endpoint +=1;
        }
        // println!("bb:{:?}的下游包括:",head.get_name());
        // for down in &set{
        //     println!("{:?}",down.get_name());
        // }
        self.node_map.insert(head, set.clone());
        set
    }
}