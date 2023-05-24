use std::collections::{HashMap, HashSet,VecDeque};

use crate::{container::bitmap::Bitmap, utility::ObjPtr};

// 值需要能够比较大小
pub trait Value {
    fn add(& self, other: &Self) -> Self;
    fn less(&self,other:&Self)->bool;
    fn new()->Self;
}

impl Value for i32 {
    fn add(&self, other: &Self) -> Self {
        *self + other
    }

    fn new()->Self{
        0
    }

    fn less(&self,other:&Self)->bool {
        self<other
    }

}


impl Value for Bitmap {

    fn add(&self, other: &Self) -> Self {
        Bitmap::or(self, other)
    }

    fn new()->Self {
        return Bitmap::new();
    }

    fn less(&self,other:&Self)->bool {
        self.count()<other.count()
    }
    
}

pub struct Graph<T: Value+'static> {
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

    // 合并环,如果一个图中,合并环中的所有节点为一个新节点
    // 新节点继承环上所有点的所有出入关系,且值为旧环上所有点的值的集合
    pub fn mergeCycles(&mut self) {
        // TODO,环融合,找到图中所有的环,进行融合
        loop {
            // 循环直到消去所有环为止
            //TODO 从头开始层次遍历,直到发现一个环,每次发现环就退出循环,对于
            let mut que:VecDeque<i32>=VecDeque::new();
        }
    }

    // 对于这个图,计算从起点from,到达任意终点集合时的最大代价路径,(只统计节点的代价,而且每个节点的代价只统计一次)
    pub fn countMaxNodeValuePath(&mut self, from: i32, tos: HashSet<i32>) -> T {
        // TODO,计算最大节点价值路径的值
        self.mergeCycles();
        //根据节点数量判断
        // 节点数量多,使用层次遍历获得一个估计路径节点值估计上限
        // 节点数量少,使用带减枝的深度优先遍历,来获取实际上的最大可能路径值
        if self.nodes.len()<10000 {
            self.dfs_findMaxNodeValuePath(from, tos)
        }
        // 
        else{
            self.bfs_estimateMaxNodeValuePath(from, tos)
        }
    }

    // 深度优先搜索找到起点到终点中的最大节点价值路径的价值，时间复杂度O(2**n),注意，该操作需要在去掉环之后进行
    pub fn dfs_findMaxNodeValuePath(&mut self,from:i32,tos:HashSet<i32>)->T{
        let mut out:T=T::new();
        let mut stack:VecDeque<(i32,i32,T)>=VecDeque::new();    //(adder,self,pathvalue) (把该节点压入栈的节点,该节点,该状态下对应的之前的路径价值,不包括该节点的价值)
        let mut passed:HashMap<i32,usize>=HashMap::new();  //记录某个节点压入过栈中的后继节点的数量,(也是压入下标)
        stack.push_back((-1,from,T::new().add(&self.nodes.get(&from).unwrap().v)));
        // 遍历栈,深度优先搜索所有路径
        while !stack.is_empty() {
            let (adder,cur,val)=stack.pop_back().unwrap();
            // 把所有可能扩展加入路径,
            if passed.contains_key(&cur) && *passed.get(&cur).unwrap()>=self.nodes.get(&cur).unwrap().outedges.len() {
                // 说明该节点的所有后继已经经历过了，移出栈
                passed.insert(cur, 0);
                continue
            }
            if passed.contains_key(&cur) {
                // 如果不是第一次遍历
                passed.insert(cur, 0);
            }else{
                // 如果是第一次遍历,判断是否是终点
                if tos.contains(&cur) {
                    if out.less(&val) {
                        out=val;
                    }
                    continue
                }
            }
            let i=passed.get(&cur).unwrap();
            let next_node=self.nodes.get(&cur).unwrap().outedges.get(*i).unwrap().as_ref().to.as_ref();
            passed.insert(cur,i+1);
            let adder=cur;
            let cur=next_node.id;
            let val=val.add(&next_node.v);
            stack.push_back((adder,cur,val));
        }
        out
    }   

    // 层次遍历，用每层的最大节点的值来代替该并行部分的可能最大值,来累加计算得到估计最大路径长度上限
    // 时间复杂度O(n)
    pub fn bfs_estimateMaxNodeValuePath(&mut self,from:i32,tos:HashSet<i32>)->T{
        T::new()
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
