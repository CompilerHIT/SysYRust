
use std::collections::HashMap;
use std::collections::HashSet;
use crate::backend::instrs::LIRInst;
use crate::backend::func::Func;
use crate::backend::block::BB;
use crate::backend::regalloc::structs::{FuncAllocStat,RegUsedStat};
use crate::backend::regalloc::regalloc::Regalloc;
use std::cmp::Ordering;
use crate::prioritydeque::PriorityDeque;


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
            // for reg in inst.get_reg_def() {
            //     let reg_id =reg.get_id();
            //     if let Some(end)=use_set.get(&reg_id) {
            //         // self.intervals.insert(reg_id, v)
            //     }else{
            //         // 否则就是定义了但是不会使用的寄存器,实际上不应该存在，因为理论上编译器前端会把这样的无用
            //         // 指令给消除
            //         // undo
            //     }
            // }
        }

    }

    // 基于剩余interval长度贪心的线性扫描寄存器分配
    fn alloc(&mut self)->FuncAllocStat{
        let mut out=FuncAllocStat::new();
        let spillings=&mut out.spillings;
        let dstr=&mut out.dstr;
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
        out
    }

}


impl Regalloc for Allocator {
    fn alloc(&mut self,func :& Func)->FuncAllocStat {
        // TODO第一次遍历，块深度标记
        self.dfs_bbs(&func.entry.unwrap());
        // 第二次遍历,指令深度标记
        self.passed.clear();
        self.inst_record(&func.entry.unwrap());
        self.passed.clear();
        // 第三次遍历,指令遍历，寄存器interval标记
        self.interval_anaylise();
        // 第四次遍历，堆滑动窗口更新获取FuncAllocStat
        self.alloc()
    }
}