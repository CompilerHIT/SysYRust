use std::fmt::Debug;

use crate::{
    ir::basicblock::BasicBlock,
    utility::{ObjPool, ObjPtr},
};

pub mod loop_recognize;

pub struct LoopList {
    pool: ObjPool<LoopInfo>,
    loops: Vec<ObjPtr<LoopInfo>>,
}

pub struct LoopInfo {
    parent: Option<ObjPtr<LoopInfo>>,
    pre_header: Option<ObjPtr<BasicBlock>>,
    header: ObjPtr<BasicBlock>,
    latchs: Option<Vec<ObjPtr<BasicBlock>>>,
    exit_blocks: Option<Vec<ObjPtr<BasicBlock>>>,
    blocks: Vec<ObjPtr<BasicBlock>>,
    sub_loops: Vec<ObjPtr<LoopInfo>>,
}

impl LoopList {
    fn new() -> Self {
        Self {
            pool: ObjPool::new(),
            loops: Vec::new(),
        }
    }

    pub fn get_loop_list(&self) -> &Vec<ObjPtr<LoopInfo>> {
        &self.loops
    }

    pub fn remove_loops(&mut self, removing_loops: &Vec<ObjPtr<LoopInfo>>) {
        removing_loops.iter().for_each(|loop_tree| {
            self.loops.remove(
                self.loops
                    .iter()
                    .position(|loop_tree2| loop_tree2 == loop_tree)
                    .unwrap(),
            );
        });
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

impl Debug for LoopInfo {
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

impl LoopInfo {
    fn new(header: ObjPtr<BasicBlock>) -> Self {
        Self {
            parent: None,
            pre_header: None,
            header,
            latchs: None,
            exit_blocks: None,
            blocks: Vec::new(),
            sub_loops: Vec::new(),
        }
    }

    /// 判断一个块是否在当前循环中
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

    /// 判断一个块是否是当前循环的头
    pub fn is_header(&self, bb: ObjPtr<BasicBlock>) -> bool {
        self.header == bb
    }

    /// 判断一个块是否在当前循环中，不递归查找子循环
    pub fn is_in_current_loop(&self, bb: &ObjPtr<BasicBlock>) -> bool {
        self.blocks.contains(bb)
    }

    /// 在第一次设置preheader的时候使用
    pub fn set_pre_header(&mut self, pre_header: ObjPtr<BasicBlock>) {
        debug_assert_eq!(self.pre_header, None);
        self.pre_header = Some(pre_header);
        if let Some(mut parent) = self.parent {
            parent.blocks.push(pre_header);
        }
    }

    /// 获得循环头
    pub fn get_header(&self) -> ObjPtr<BasicBlock> {
        self.header
    }

    /// 获得pre_header
    pub fn get_preheader(&self) -> ObjPtr<BasicBlock> {
        debug_assert_ne!(
            self.pre_header, None,
            "No preheader in current loop: {:?}",
            self
        );
        self.pre_header.unwrap()
    }

    /// 获得latchs
    pub fn get_latchs(&self) -> Vec<ObjPtr<BasicBlock>> {
        self.header
            .get_up_bb()
            .iter()
            .filter(|bb| self.is_in_current_loop(bb))
            .cloned()
            .collect()
    }

    /// 获得当前循环的块
    pub fn get_current_loop_bb(&self) -> &Vec<ObjPtr<BasicBlock>> {
        &self.blocks
    }

    /// 增加当前循环的块
    pub fn add_bbs(&mut self, bb: Vec<ObjPtr<BasicBlock>>) {
        self.blocks.extend(bb);
    }

    /// 获得当前循环的子循环
    pub fn get_sub_loops(&self) -> &Vec<ObjPtr<LoopInfo>> {
        &self.sub_loops
    }

    /// 删除某个子循环
    pub fn remove_sub_loop(&mut self, sub_loop: ObjPtr<LoopInfo>) {
        self.sub_loops.retain(|loop_tree| *loop_tree != sub_loop);
    }

    /// 获得当前循环的父循环
    pub fn get_parent_loop(&self) -> Option<ObjPtr<LoopInfo>> {
        self.parent
    }

    /// 获得当前循环的出口块
    /// 动态计算，当exit_blocks为空时，计算一次
    pub fn get_exit_blocks(&mut self) -> Vec<ObjPtr<BasicBlock>> {
        if let Some(exit_blocks) = &self.exit_blocks {
            exit_blocks.clone()
        } else {
            let mut exit_blocks = Vec::new();
            for bb in self.blocks.iter() {
                bb.get_next_bb().iter().any(|next_bb| {
                    if !self.is_in_loop(next_bb) {
                        exit_blocks.push(*bb);
                        true
                    } else {
                        false
                    }
                });
            }
            self.exit_blocks = Some(exit_blocks.clone());
            exit_blocks
        }
    }

    /// 获得当前循环的latch块
    /// 动态计算，当latchs为空时，计算一次
    pub fn get_latch_blocks(&mut self) -> Vec<ObjPtr<BasicBlock>> {
        if let Some(latchs) = &self.latchs {
            latchs.clone()
        } else {
            let mut latchs = Vec::new();
            self.get_header().get_up_bb().iter().for_each(|bb| {
                if self.is_in_current_loop(bb) {
                    latchs.push(*bb);
                }
            });
            self.latchs = Some(latchs.clone());
            latchs
        }
    }
}
