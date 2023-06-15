use crate::algorithm::graphalgo::Graph;
use crate::backend::block::BB;
use crate::backend::func::Func;
use crate::backend::instrs::LIRInst;
use crate::backend::regalloc::regalloc::Regalloc;
use crate::backend::regalloc::structs::{FuncAllocStat, RegUsedStat};
use crate::container::bitmap::Bitmap;
use crate::container::prioritydeque::PriorityDeque;
use crate::utility::ObjPtr;
use crate::utility::ScalarType;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

// 摆烂的深度优先指令编码简单实现的线性扫描寄存器分配
pub struct Allocator {
    depths: HashMap<ObjPtr<BB>, usize>,
    passed: HashSet<ObjPtr<BB>>,
    lines: Vec<ObjPtr<LIRInst>>,
    intervals: HashMap<i32, usize>, //key,val=虚拟寄存器号，周期结尾指令号
    base: usize,                    //用于分配指令号
}

#[derive(Eq, PartialEq)]
struct RegInterval {
    pub id: i32,
    pub end: usize,
}

impl RegInterval {
    fn new(id: i32, end: usize) -> RegInterval {
        RegInterval { id, end }
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
        let o = self.cmp(&other);
        match o {
            Ordering::Greater => self,
            Ordering::Equal => self,
            Ordering::Less => other,
        }
    }
    // fn min(self, other: Self) -> Self {

    // }
    // fn clamp(self, min: Self, max: Self) -> Self{

    // }
}

struct BlockGraph {
    pub graph: Graph<Bitmap>, //图
    pub from: i32,            //记录起点节点
    pub to: HashSet<i32>,     //记录终点节点
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            passed: HashSet::new(),
            lines: Vec::new(),
            base: 0,
            depths: HashMap::new(),
            intervals: HashMap::new(),
        }
    }

    // 深度分配
    fn dfs_bbs(&mut self, bb: ObjPtr<BB>) {
        let n = self.passed.len();
        ////println!("bb.length:{n}");
        if self.passed.contains(&bb) {
            return;
        }
        // 遍历块,更新高度
        self.depths.insert(bb, self.base);
        self.passed.insert(bb.clone());
        self.base += 1;
        // ////println!("before");
        // 深度优先遍历后面的块
        for next in &bb.as_ref().out_edge {
            self.dfs_bbs(next.clone());
            ////println!("after clone ");
        }
        ////println!("once end");
    }
    // 指令编号
    fn inst_record(&mut self, bb: ObjPtr<BB>) {
        // 根据块深度分配的结果启发对指令进行编号
        if self.passed.contains(&bb) {
            return;
        }
        self.passed.insert(bb.clone());
        for line in &bb.as_ref().insts {
            self.lines.push(line.clone())
        }
        // then choice a block to go through
        let mut set: HashSet<usize> = HashSet::new();
        loop {
            let mut toPass: usize = bb.as_ref().out_edge.len();
            for (i, next) in bb.as_ref().out_edge.iter().enumerate() {
                if set.contains(&i) {
                    continue;
                }
                if self.passed.contains(next) {
                    continue;
                }
                if toPass == bb.as_ref().out_edge.len() {
                    toPass = i;
                } else {
                    if self.depths.get(next)
                        < self.depths.get(bb.as_ref().out_edge.get(toPass).unwrap())
                    {
                        toPass = i;
                    }
                }
            }
            if toPass == bb.as_ref().out_edge.len() {
                break;
            }
            set.insert(toPass);
            self.inst_record(bb.as_ref().out_edge.get(toPass).unwrap().clone())
        }
    }

    // 指令窗口分析
    fn interval_anaylise(&mut self) {
        for (i, inst) in self.lines.iter().enumerate() {
            for reg in inst.as_ref().get_reg_use() {
                if !reg.is_virtual() {
                    continue;
                }
                self.intervals.insert(reg.get_id(), i);
            }
            for reg in inst.get_reg_def() {
                if !reg.is_virtual() {
                    continue;
                }
                self.intervals.insert(reg.get_id(), i);
            }
        }
    }

    // 从函数得到图
    fn funcToGraph(func: &Func) -> BlockGraph {
        let mut out = BlockGraph {
            graph: Graph::new(),
            from: 0,
            to: HashSet::new(),
        };
        // TODO ,具体从函数来构建图的操作
        out
    }

    pub fn countStackSize(
        func: &Func,
        spillings: &HashSet<i32>,
    ) -> (usize, HashMap<ObjPtr<BB>, usize>) {
        // 遍历所有块,找到每个块中的spillings大小,返回其中大小的最大值,
        let mut stackSize: usize = 0;
        let mut bb_stack_sizes: HashMap<ObjPtr<BB>, usize> = HashMap::new();
        let mut passed: HashSet<ObjPtr<BB>> = HashSet::new();
        let mut walk: VecDeque<ObjPtr<BB>> = VecDeque::new();
        walk.push_back(func.entry.unwrap().clone());
        passed.insert(func.entry.unwrap());
        // TOTEST
        while !walk.is_empty() {
            let cur = walk.pop_front().unwrap();
            let mut bbspillings: HashSet<i32> = HashSet::new();
            //println!("{}",cur.label);
            for reg in &cur.as_ref().live_in {
                if spillings.contains(&reg.get_id()) {
                    bbspillings.insert(reg.get_id());
                }
            }
            let start = bbspillings.len() * 8;
            bb_stack_sizes.insert(cur, start);
            bbspillings.clear();
            // 统计spilling数量
            for inst in &cur.as_ref().insts {
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
            }
            if bbspillings.len() * 8 + start > stackSize {
                stackSize = bbspillings.len() * 8 + start;
            }
            // 扩展未扩展的节点
            for bb in &cur.as_ref().out_edge {
                if passed.contains(&bb) {
                    continue;
                }
                passed.insert(bb.clone());
                walk.push_back(bb.clone());
            }
        }
        (stackSize, bb_stack_sizes)
    }

    // 基于剩余interval长度贪心的线性扫描寄存器分配
    fn allocRegister(&mut self) -> (HashSet<i32>, HashMap<i32, i32>) {
        let mut spillings: HashSet<i32> = HashSet::new();
        let mut dstr: HashMap<i32, i32> = HashMap::new();
        // 寄存器分配的长度限制
        // 可用寄存器
        let mut reg_used_stat = RegUsedStat::new();
        let mut iwindow: PriorityDeque<RegInterval> = PriorityDeque::new();
        let mut fwindow: PriorityDeque<RegInterval> = PriorityDeque::new();
        // 遍历指令
        for (i, it) in self.lines.iter().enumerate() {
            // 先通用寄存器窗口中判断有没有可以释放的寄存器
            while iwindow.len() != 0 {
                if let Some(min) = iwindow.front() {
                    if min.end <= i {
                        // 获取已经使用寄存器号，释放
                        let iereg: i32 = *dstr.get(&min.id).unwrap();
                        reg_used_stat.release_ireg(iereg);
                        iwindow.pop_front();
                    } else {
                        break;
                    }
                }
            }
            // 判断浮点寄存器窗口中有没有可以释放的寄存器,进行释放
            while fwindow.len() != 0 {
                if let Some(min) = fwindow.front() {
                    if min.end <= i {
                        let fereg: i32 = *dstr.get(&min.id).unwrap();
                        reg_used_stat.release_freg(fereg);
                        fwindow.pop_front();
                    } else {
                        break;
                    }
                }
            }

            for reg in it.as_ref().get_reg_def() {
                if !reg.is_virtual() {
                    continue;
                }

                let id = reg.get_id();
                // 如果已经在dstr的key中，也就是已经分配，则忽略处理
                if dstr.contains_key(&id) {
                    continue;
                }
                // 如果已经归为溢出寄存器，则不再重复处理
                if spillings.contains(&id) {
                    continue;
                }
                // 在周期表中搜索该寄存器的终结周期
                let end = *self.intervals.get(&id).unwrap();
                // 定义寄存器溢出处理流程
                let mut spill_reg_for = |tmpwindow: &mut PriorityDeque<RegInterval>| {
                    // let mut tmpwindow=&mut fwindow;
                    // let available:Option<Reg>;
                    let max = tmpwindow.back();
                    let mut maxID = 0;
                    let mut ifSpilling = false; //记录将新寄存器处理成溢出
                    match max {
                        Some(max) => {
                            if max.end > end {
                                // 溢出旧寄存器
                                ifSpilling = false;
                                maxID = max.id;
                            } else {
                                // 溢出新寄存器
                                ifSpilling = true;
                            }
                        }
                        None => (),
                    }
                    if ifSpilling {
                        spillings.insert(id);
                    } else {
                        tmpwindow.pop_back();
                        dstr.insert(id, *dstr.get(&maxID).unwrap()); //给新寄存器分配旧寄存器所有的寄存器
                        dstr.remove(&maxID); //解除旧末虚拟寄存器与实际寄存器的契约
                        tmpwindow.push(RegInterval::new(id, end)); //把心的分配结果加入窗口
                    }
                };

                // TODO,逻辑判断选择不同的分配方案
                if reg.get_type() == ScalarType::Int
                // 如果是通用寄存器
                {
                    if let Some(ereg) = reg_used_stat.get_available_ireg() {
                        // 如果还有多余的通用寄存器使用
                        dstr.insert(id, ereg);
                        reg_used_stat.use_ireg(ereg);
                        iwindow.push(RegInterval::new(id, end))
                    } else {
                        spill_reg_for(&mut iwindow);
                    }
                }
                // 如果是浮点寄存器
                else if reg.get_type() == ScalarType::Float {
                    if let Some(ereg) = reg_used_stat.get_available_freg() {
                        // 如果还有多余的浮点寄存器
                        dstr.insert(id, ereg);
                        reg_used_stat.use_freg(ereg); //记录float_entity_reg为被使用状态
                        fwindow.push(RegInterval::new(id, end))
                    } else {
                        spill_reg_for(&mut fwindow);
                    }
                }
            }
        }
        (spillings, dstr)
    }

    // 统计一个块中spilling的寄存器数量
    fn count_block_spillings(bb: &BB, spillings: &HashSet<i32>) -> usize {
        let mut set: HashSet<i32> = HashSet::new();
        for inst in &bb.insts {
            let inst = inst.as_ref();
            for reg in inst.get_reg_def() {
                let id = reg.get_id();
                if spillings.contains(&id) {
                    set.insert(id);
                }
            }
            for reg in inst.get_reg_use() {
                let id = reg.get_id();
                if spillings.contains(&id) {
                    set.insert(id);
                }
            }
        }
        set.len()
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &Func) -> FuncAllocStat {
        // TODO第一次遍历，块深度标记
        self.passed.clear();
        self.dfs_bbs(func.entry.unwrap());
        // 第二次遍历,指令深度标记
        self.passed.clear();
        self.inst_record(func.entry.unwrap());
        // 第三次遍历,指令遍历，寄存器interval标记
        self.interval_anaylise();

        // 第四次遍历，堆滑动窗口更新获取FuncAllocStat
        let (spillings, dstr) = self.allocRegister();
        // //println!("_______________________________________________");
        // //println!("{}",func.label);
        // //println!("{:?}",spillings);
        let (stack_size, bb_stack_sizes) = Allocator::countStackSize(func, &spillings);
        // let stack_size=spillings.len(); //TO REMOVE
        let mut out = FuncAllocStat {
            stack_size,
            bb_stack_sizes,
            spillings,
            dstr,
        };
        // //println!("{:?}",out.spillings);
        out
    }
}
