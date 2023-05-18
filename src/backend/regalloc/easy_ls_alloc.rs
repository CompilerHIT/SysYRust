use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use crate::backend::instrs::LIRInst;
use crate::backend::func::Func;
use crate::backend::block::BB;
use crate::backend::regalloc::structs::{FuncAllocStat,BlockAllocStat,RegUsedStat};
use crate::backend::regalloc::regalloc::Regalloc;
use crate::prioritydeque::PriorityDeque;
use std::cmp::Ordering;
use crate::prioritydeque;



// 摆烂的深度优先指令编码简单实现的线性扫描寄存器分配
pub struct Allocator{
    func :Option<Box<Func>>,
    depths: HashMap<&'static BB,usize>,
    passed:HashSet<Box<&'static BB>>,
    lines:Vec<&'static LIRInst>,
    intervals:HashMap<i32,usize>,   //key,val=虚拟寄存器号，周期结尾指令号
    base:usize, //用于分配指令号
}

#[derive(Eq,PartialEq)]
struct RegInterval {
    def :i32 ,
    end: i32,
}
impl PartialOrd for RegInterval {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.end.cmp(&other.end))
    }
}   
impl Ord for RegInterval {
    fn cmp(&self, other: &Self) -> Ordering {
        // Rust中BinaryHeap的默认实现是大根堆,我们需要的正是大根堆
        self.partial_cmp(other).unwrap()
    }
    fn max(self, other: Self) -> Self {
        let o=self.cmp(&other);
        match o {
            Ordering::Greater=> self,
            Ordering::Equal=>self,
            Ordering::Less=> other,
        }
    }
    // fn min(self, other: Self) -> Self {
        
    // }
    // fn clamp(self, min: Self, max: Self) -> Self{
        
    // }
}


impl Allocator {
    fn new()->Allocator {
        Allocator { func: Option::None, passed: HashSet::new(), lines: Vec::new(), base: 0, depths: HashMap::new(), intervals: HashMap::new() }
    }

    // 深度分配
    fn dfs_bbs(&mut self,bb :&'static BB) {
        if self.passed.contains(&bb) {
            return 
        }
        // 遍历块,更新高度
        self.depths.insert(bb,self.base);
        self.base+=1;
        // 深度优先遍历后面的块
        

    }
    // 指令编号
    fn inst_record(&mut self,bb:&'static BB) {

    }

    // 指令窗口分析
    fn interval_anaylise(&mut self){
        
    }

    // 基于剩余interval长度贪心的线性扫描寄存器分配
    fn alloc(&mut self)->FuncAllocStat{
        let mut out=FuncAllocStat::new();
        let spillings=&mut out.spillings;
        let dstr=&mut out.dstr;
        // 寄存器分配的长度限制
        let numIReg=32;
        let numFReg=32;
        // 可用寄存器
        let mut regUsedStat=RegUsedStat::new();
        let mut iwindow:PriorityDeque<RegInterval>=PriorityDeque::new();

        // 遍历指令
        for it in self.lines.iter() {
            for reg in it.get_reg_def() {
                let id=reg.get_id();
                // 先判断有没有可以释放的id

            }
        }
        out
    }

}


impl Regalloc for Allocator {
    fn alloc(&mut self,func :& Func)->FuncAllocStat {
        // TODO第一次遍历，块深度标记
        self.dfs_bbs(&func.entry.unwrap());
        // 第二次遍历,指令深度标记
        self.inst_record(&func.entry.unwrap());
        // 第三次遍历,指令遍历，寄存器interval标记
        self.interval_anaylise();
        // 第四次遍历，堆滑动窗口更新获取FuncAllocStat
        self.alloc()
    }
}