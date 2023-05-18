use std::collections::HashMap;
use std::collections::HashSet;
use crate::backend::instrs::LIRInst;
use crate::backend::func::Func;
use crate::backend::block::BB;
use crate::backend::regalloc::structs::{FuncAllocStat,BlockAllocStat,RegUsedStat};
use crate::backend::regalloc::regalloc::Regalloc;


// 摆烂的深度优先指令编码简单实现的线性扫描寄存器分配
pub struct EasyLsAllocator{
    func :Option<Box<Func>>,
    depths: HashMap<&'static BB,usize>,
    passed:HashSet<Box<&'static BB>>,
    lines:Vec<&'static LIRInst>,
    intervals:HashMap<i32,usize>,   //key,val=虚拟寄存器号，周期结尾指令号
    base:usize, //用于分配指令号
}
impl EasyLsAllocator {
    fn new()->EasyLsAllocator {
        EasyLsAllocator { func: Option::None, passed: HashSet::new(), lines: Vec::new(), base: 0, depths: HashMap::new(), intervals: HashMap::new() }
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

        // 遍历指令
        for it in self.lines.iter() {


        }

        out
    }

}


impl Regalloc for EasyLsAllocator {
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