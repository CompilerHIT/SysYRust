use std::fmt::Debug;

use crate::{
    ir::basicblock::BasicBlock,
    utility::{ObjPool, ObjPtr},
};

pub mod loop_recognize;

pub struct LoopList {
    pool: ObjPool<LoopTree>,
    loops: Vec<ObjPtr<LoopTree>>,
}

pub struct LoopTree {
    parent: Option<ObjPtr<LoopTree>>,
    header: ObjPtr<BasicBlock>,
    sub_loops: Vec<ObjPtr<LoopTree>>,
    blocks: Vec<ObjPtr<BasicBlock>>,
}

impl LoopList {
    fn new() -> Self {
        Self {
            pool: ObjPool::new(),
            loops: Vec::new(),
        }
    }
}

impl Debug for LoopList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        self.loops.iter().for_each(|loop_tree| {
            s += &format!("{:?}", loop_tree);
        });
        write!(f, "{}", s)
    }
}

impl Debug for LoopTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s += &format!("Loop: \n");
        s += &format!("header: {}\n", self.header.get_name());
        s += &format!("blocks: \n");
        for i in self.blocks.iter() {
            s += &format!("{}\n", i.get_name());
        }
        s += &format!("sub loops: \n");
        for i in self.sub_loops.iter() {
            s += &format!("{}\n", i.header.get_name());
        }

        write!(f, "{}", s)
    }
}

impl LoopTree {
    fn new(header: ObjPtr<BasicBlock>) -> Self {
        Self {
            parent: None,
            header,
            sub_loops: Vec::new(),
            blocks: Vec::new(),
        }
    }

    // 判断一个块是否在当前循环中
    pub fn is_in_loop(&self, bb: &ObjPtr<BasicBlock>) -> bool {
        if self.blocks.contains(bb) {
            true
        } else {
            let mut in_loop = false;
            self.sub_loops.iter().for_each(|sub_loop| {
                if sub_loop.is_in_loop(bb) {
                    in_loop = true;
                    return;
                }
            });
            in_loop
        }
    }
}
