use std::collections::{HashMap, HashSet, VecDeque};

use crate::{utility::ObjPtr, ir::{basicblock::BasicBlock, tools::bfs_bb_proceess}};

pub struct MatchBlackInfo{
    num_blk:i32,
    num_blk_map:HashMap<i32,ObjPtr<BasicBlock>>,
    blk_num_map:HashMap<ObjPtr<BasicBlock>,i32>,
    node_edge_map:HashMap<i32,Vec<i32>>,
}
impl MatchBlackInfo {
    pub fn new()->MatchBlackInfo{
        MatchBlackInfo { num_blk: 0 ,num_blk_map:HashMap::new(),node_edge_map:HashMap::new(),blk_num_map:HashMap::new()}
    }

    pub fn get_len(&self)->i32{
        self.num_blk
    }

    pub fn get_num(&mut self,bb:ObjPtr<BasicBlock>)->i32{
        if let Some(num) = self.blk_num_map.get(&bb){
            return *num;
        }else{
            self.num_blk +=1;
            self.num_blk_map.insert(self.num_blk, bb);
            self.blk_num_map.insert(bb, self.num_blk);
            return self.num_blk;
        }
    }

    pub fn get_blk(&self,num:i32)->ObjPtr<BasicBlock>{
        self.num_blk_map.get(&num).unwrap().clone()
    }

    pub fn get_node_edge_map(&self)->HashMap<i32,Vec<i32>>{
        self.node_edge_map.clone()
    }

    pub fn get_num_blk_map(&self)->HashMap<i32,ObjPtr<BasicBlock>>{
        self.num_blk_map.clone()
    }

}

pub fn get_blk_info(head:ObjPtr<BasicBlock>)->MatchBlackInfo{
    let mut blk_info = MatchBlackInfo::new();
    bfs_bb_proceess(head, |bb| {
        let node_num = blk_info.get_num(bb);
        let mut vec_edge = vec![];
        for next in bb.get_next_bb(){
            vec_edge.push(blk_info.get_num(*next));
        }
        blk_info.node_edge_map.insert(node_num, vec_edge);
    });
    blk_info
}

pub fn match_blks(head1:ObjPtr<BasicBlock>,head2:ObjPtr<BasicBlock>)->Option<HashMap<ObjPtr<BasicBlock>,ObjPtr<BasicBlock>>>{
    let blk_info1 = get_blk_info(head1);
    let blk_info2 = get_blk_info(head2);
    let mut map = HashMap::new();
    if blk_info1.get_node_edge_map()==blk_info2.get_node_edge_map(){
        for i in 0..blk_info1.get_len(){
            map.insert(blk_info1.get_blk(i), blk_info2.get_blk(i));
        }
    }else{
        return None;
    }
    Some(map)
}