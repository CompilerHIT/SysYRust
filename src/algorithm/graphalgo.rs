use std::collections::{HashMap, HashSet};

use crate::{container::bitmap::Bitmap, utility::ObjPtr};

pub trait Value {
    fn add(&mut self, other: &Self) -> Self;
}

impl Value for i32 {
    fn add(&mut self, other: &Self) -> Self {
        *self + other
    }
}

impl Value for Bitmap {
    fn add(&mut self, other: &Self) -> Self {
        Bitmap::or(self, other)
    }
}

pub struct Graph<T: Value> {
    nodes: HashMap<i32, Node<T>>,
}

impl<T: Value> Graph<T> {
    pub fn new() -> Graph<T> {
        Graph {
            nodes: HashMap::new(),
        }
    }

    // 判断一个节点是否是串点 ,(串点左侧把一个图分成两半,左侧和右侧的点不重合)
    pub fn isSerial(&self, node: i32) -> bool {
        // 左右扩展,串行判断方法,从往两边同时扩展
        // 扩展过程包括in_edge和out_edge,如果遇到node，停止
        // 扩展到尽头节点，也停止

        // 如果全部扩展结束，两个集合都没有交集,则说明这个点是串点

        false
    }

    // 合并相邻节点,如果一个图中,两个节点之间互相有边指向对方的话,合并两个节点得到一个新节点
    // 新节点继承两个边的所有出入关系,而且新节点的值为旧两个节点的值的和
    pub fn mergeNeighbors(&mut self) {
        // TODO
    }

    // 对于这个图,计算从起点from,到达任意终点集合时的最大代价路径,(只统计节点的代价,而且每个节点的代价只统计一次)
    pub fn countMaxNodeCostPath(&mut self, from: i32, ends: HashSet<i32>) -> T {
        // TODO
        let out: T = self.nodes.get(&from).unwrap().v; //先加入起点的值
        let mut serials: HashSet<i32> = HashSet::new(); //串点集合
        let mut ranges: Vec<(i32, i32, usize)> = Vec::new(); //记录并行区域,以及区域中节点的数量e.m. [(from,to,num),...]
        let mut counted: HashSet<i32> = HashSet::new(); //已经统计过的点的集合
                                                        //TODO 分析节点的不平行关系,分成多个串行区

        // 获取串点集合
        for i in self.nodes {
            if self.isSerial(i.0) {
                serials.insert(i.0);
            }
        }
        // 把终点都加入串点
        for v in ends {
            serials.insert(v);
        }
        // 获取串行区,该部分时间复杂度为O(n**2)
        serials.insert(from);
        for end in serials {
            if end == from {
                continue;
            }
            // 以每个串行点为终点往前统计直到遇到另外一个串行点
            // 从end出发,沿着in-edge不断往前遍历直到遍历到遇到另一个串点,把另一个串点记录为起点
            let mut st = end;
            let mut toWalk: Vec<i32> = Vec::new();
            toWalk.push(end);
            let mut pos = 0; //从st位置开始遍历
            let mut walkPassed: HashSet<i32> = HashSet::new();
            walkPassed.insert(end);
            while pos < toWalk.len() {
                let walk = toWalk.get(pos).unwrap();
                if let Some(node) = self.nodes.get(&walk) {
                    // 然后从in_edge中找到前继,
                    for inedge in node.inedges {
                        let inedge = inedge.as_ref();
                        let fromId = inedge.from.as_ref().id;
                        if walkPassed.contains(&fromId) {
                            continue;
                        }
                        if serials.contains(&fromId) {
                            st = fromId;
                        }
                        toWalk.push(fromId);
                        walkPassed.insert(fromId);
                    }
                }
                pos += 1;
            }
            //遍历完后,walkPasssed的大小就是该子图的大小
            ranges.push((st, end, walkPassed.len()));
        }

        // 计算每个串行区的大小

        // 把末尾集合加入到串行块集合中

        // 对于串行点与串行点之间的部分
        // 一个设置为源点，一个为汇点
        // 如果之间的部分的数量小于一个阈值k,那么进行剪枝的深度优先搜索，处理所有可能
        // 找到最小代价

        // 如果之间的部分的数量大于等于k:
        // 从源点出发，进行广度优先搜索,找到一个步数最短路径到达汇点
        // 但是这个路径并不一定是最大收获路径
        out
    }
}

// 节点的价值要实现一个合并接口
pub struct Node<T: Value> {
    pub id: i32, //节点的id
    pub v:T,    //节点的值
    pub inedges: Vec<ObjPtr<Edge<T>>>,
    pub outedges: Vec<ObjPtr<Edge<T>>>,
}

pub struct Edge<T: Value> {
    pub from: ObjPtr<Node<T>>,
    pub to: ObjPtr<Node<T>>,
}
