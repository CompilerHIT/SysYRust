
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use crate::backend::instrs::LIRInst;
use crate::backend::func::Func;
use crate::backend::block::BB;
use crate::backend::regalloc::structs::{FuncAllocStat,RegUsedStat};
use crate::backend::regalloc::regalloc::Regalloc;
use crate::container::bitmap::Bitmap;
use std::cmp::Ordering;
use crate::container::prioritydeque::PriorityDeque;
use crate::algorithm::graphalgo;
use crate::algorithm::graphalgo::Graph;


// 摆烂的深度优先指令编码简单实现的线性扫描寄存器分配
pub struct Allocator{
    func :Option<Box<Func>>,
    depths: HashMap<&'static BB,usize>,
    passed:HashSet<Box<&'static BB>>,
    lines:Vec<&'static LIRInst>,
    intervals:HashMap<i32,i32>,   //key,val=虚拟寄存器号，周期结尾指令号
    base:usize, //用于分配指令号
}

#[derive(Eq,PartialEq)]
struct RegInterval {
    pub id :i32,
    pub end:i32,
}

impl RegInterval {
    fn new(id:i32,end:i32) ->RegInterval {
        RegInterval { id, end}
    }
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

struct BlockGraph {
    pub graph:Graph<Bitmap>,    //图
    pub from:i32,  //记录起点节点
    pub to:HashSet<i32> //记录终点节点
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
        for next in bb.out_edge {
            self.dfs_bbs(next.as_ref())
        }
    }
    // 指令编号
    fn inst_record(&mut self,bb:&'static BB) {
        // 根据块深度分配的结果启发对指令进行编号
        if self.passed.contains(&bb) {
            return ;
        }
        for line in bb.insts {
            self.lines.push(line.as_ref())
        }
        // then choice a block to go through 
        let mut set:HashSet<usize> =HashSet::new();
        loop {
            let mut toPass:usize=bb.out_edge.len();
            for (i,next) in bb.out_edge.iter().enumerate() {
                if set.contains(&toPass) {
                    continue;
                }
                if toPass==bb.out_edge.len() {
                    toPass=i;
                }else{
                    if self.depths.get(next.as_ref())<self.depths.get(bb.out_edge.get(toPass).unwrap().as_ref()) {
                        toPass=i;
                    }
                }
            }
            if toPass== bb.out_edge.len() {
                break;
            }
            set.insert(toPass) ;
            self.inst_record(bb.out_edge.get(toPass).unwrap().as_ref())
        }
    }

    // 指令窗口分析
    fn interval_anaylise(&mut self){
        let mut use_set:HashMap<i32,usize> =HashMap::new();
        for (i,inst) in self.lines.iter().enumerate() {
            for reg in inst.get_reg_use() {
                if !reg.is_allocable() {
                    continue;
                }
                use_set.insert(reg.get_id(), i);
            }
        }

    }

    // 从函数得到图
    fn funcToGraph(func:&Func)->BlockGraph{
        let mut out=BlockGraph{graph:Graph::new(),from:0,to:HashSet::new()};
        // TODO ,具体从函数来构建图的操作
        out
    }


    
    pub fn countStackSize(func:&Func,spillings:&HashSet<i32>) ->usize {
        // 遍历所有块,找到每个块中的spillings大小,返回其中大小的最大值,
        let mut stackSize:usize;
        let mut passed:HashSet<&BB>=HashSet::new();
        let mut walk:VecDeque<&BB>=VecDeque::new();
        walk.push_back(func.entry.unwrap().as_ref());
        passed.insert(func.entry.unwrap().as_ref());
        // TOTEST
        while !walk.is_empty() {
            let cur=walk.pop_front().unwrap();
            let mut bbspillings:HashSet<i32>=HashSet::new();
            for reg in cur.live_in {
                if spillings.contains(&reg.get_id()) {
                    bbspillings.insert(reg.get_id());
                }
            }
            // 统计spilling数量
            for inst in cur.insts {
                for reg in inst.as_ref().get_reg_def() {
                    if spillings.contains(&reg.get_id()) {
                        bbspillings.insert(reg.get_id());
                    }
                }
                for reg in inst.as_ref().get_reg_use() {
                    if spillings.contains(&reg.get_id()) {
                        bbspillings.insert(reg.get_id());
                    }
                }
                if bbspillings.len()>stackSize {
                    stackSize=bbspillings.len();
                }
            }
            
            // 扩展未扩展的节点
            for bb in cur.out_edge {
                if passed.contains(bb.as_ref()) {
                    continue
                }
                passed.insert(bb.as_ref());
                walk.push_back(bb.as_ref());
            }
        }
        stackSize
    }

    // 基于剩余interval长度贪心的线性扫描寄存器分配
    fn allocRegister(&mut self)->(HashSet<i32>,HashMap<i32,i32>){
        let mut spillings:HashSet<i32>=HashSet::new();
        let mut dstr:HashMap<i32,i32>=HashMap::new();
        // 寄存器分配的长度限制
        // 可用寄存器
        let mut regUsedStat=RegUsedStat::new();
        let mut iwindow:PriorityDeque<RegInterval>=PriorityDeque::new();
        // 遍历指令
        for (i,it) in self.lines.iter().enumerate() {
            // 先判断有没有可以释放的寄存器
            while iwindow.len()!=0  {
                if let Some(min)=iwindow.front() {
                    if min.end<=i as i32{
                        iwindow.pop_front();
                    }
                }
            }

            for reg in it.get_reg_def() {
                if !reg.is_allocable() {continue;}
                let id=reg.get_id();
                if dstr.contains_key(&id) {
                    continue;
                }
                let end=*self.intervals.get(&id).unwrap();
                // 先判断有没有可以使用的寄存器,如果有,则分配
                if let Some(ereg)=regUsedStat.get_available_ireg() {
                    dstr.insert(id, ereg);
                    iwindow.push(RegInterval::new(id,end))
                }else{
                    // 否则判断哪个的inerval更长,去掉更长的
                    if let Some(max)=iwindow.back() {
                        if max.end>end {
                            iwindow.pop_back();
                            dstr.insert(id, *dstr.get(&max.id).unwrap());
                            dstr.remove(&max.id);
                            iwindow.push(RegInterval::new(id,end))
                        }else{
                            spillings.insert(id);
                        }
                    }
                    // spillings.insert(id);
                }
            }
        }
        (spillings,dstr)
    }



    // 统计一个块中spilling的寄存器数量
    fn count_block_spillings(bb:&BB,spillings:&HashSet<i32>)->usize {
        let mut set:HashSet<i32>=HashSet::new();
        for inst in bb.insts {
            let inst=inst.as_ref();
            for reg in inst.get_reg_def() {
                let id=reg.get_id();
                if spillings.contains(&id) {
                    set.insert(id);
                }
            }
            for reg in inst.get_reg_use() {
                let id=reg.get_id();
                if spillings.contains(&id) {
                    set.insert(id);
                }
            }
        }
        set.len() 
    }

}


impl Regalloc for Allocator {
    fn alloc(&mut self,func :& Func)->FuncAllocStat {
        // TODO第一次遍历，块深度标记
        self.dfs_bbs(&func.entry.unwrap().as_ref());
        // 第二次遍历,指令深度标记
        self.passed.clear();
        self.inst_record(&func.entry.unwrap().as_ref());
        self.passed.clear();
        // 第三次遍历,指令遍历，寄存器interval标记
        self.interval_anaylise();
        // 第四次遍历，堆滑动窗口更新获取FuncAllocStat
        let (spillings,dstr)=self.allocRegister();
        let stack_size=Allocator::countStackSize(func,&spillings);
        // let stack_size=spillings.len(); //TO REMOVE
        FuncAllocStat { stack_size, spillings, dstr}
    }
}