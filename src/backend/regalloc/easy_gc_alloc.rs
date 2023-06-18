// a impl of graph color register alloc algo

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    future::IntoFuture,
};

use crate::{
    backend::{
        instrs::{Func, BB},
        operand::Reg,
    },
    utility::{ObjPool, ObjPtr, ScalarType},
};

use super::{
    easy_ls_alloc,
    regalloc::{Regalloc, self},
    structs::{FuncAllocStat, RegUsedStat},
};

pub struct Allocator {
    i_regs: Vec<i32>,                          //所有虚拟通用寄存器的列表
    f_regs: Vec<i32>,                          //所有虚拟浮点寄存器列表
    icolors: HashMap<i32, i32>,                //整数寄存器分配的着色
    fcolors: HashMap<i32, i32>,                //  浮点寄存器分配的着色
    costs_reg: HashMap<i32, i32>,              //记录虚拟寄存器的使用次数(作为代价)
    regs_available: HashMap<i32, RegUsedStat>, // 保存每个点的可用寄存器集合
    i_alloc_finish: bool,
    f_alloc_finish: bool,
    i_interence_reg: Option<i32>, //记录冲突的通用寄存器
    f_interence_reg: Option<i32>,
    f_interference_graph: HashMap<i32, HashSet<i32>>, //浮点寄存器冲突图
    i_interference_graph: HashMap<i32, HashSet<i32>>, //浮点寄存器冲突图
    dstr: HashMap<i32, i32>,                          //记录每个寄存器分配到的实际寄存器
    spillings: HashSet<i32>,                          //记录溢出寄存器
}



impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            i_regs: Vec::new(),
            f_regs: Vec::new(),
            icolors: HashMap::new(),
            fcolors: HashMap::new(),
            costs_reg: HashMap::new(),
            regs_available: HashMap::new(),
            i_interference_graph: HashMap::new(),
            f_interference_graph: HashMap::new(),
            i_alloc_finish: false,
            f_alloc_finish: true,
            i_interence_reg: None,
            f_interence_reg: None,
            dstr: HashMap::new(),
            spillings: HashSet::new(),
        }
    }

    // 判断两个虚拟寄存器是否是通用寄存器分配冲突

    // 建立虚拟寄存器之间的冲突图
    fn build_interference_graph(&mut self, func: &Func) {
        // TODO,更新cost
        // 遍历所有块,得到所有虚拟寄存器和所有虚拟寄存器之间的冲突关系
        let mut que: VecDeque<ObjPtr<BB>> = VecDeque::new(); //广度优先遍历块用到的队列
        let mut passed: HashSet<ObjPtr<BB>> = HashSet::new();
        que.push_front(func.entry.unwrap());
        let mut passed_reg: HashSet<Reg> = HashSet::new();
        // 定义处理函数
        let process = |cur_bb: ObjPtr<BB>,
                       interef_graph: &mut HashMap<i32, HashSet<i32>>,
                       kind: ScalarType| {
            // TODO,调试用
            if func.label == "main" {
                //println!("target");
            }

            let mut livenow: HashSet<i32> = HashSet::new();
            // 重构冲突分析
            let mut sets_end: HashMap<i32, HashSet<i32>> = HashMap::new();

            let mut end_poses: HashMap<i32, i32> = HashMap::new();

            let mut ends = HashSet::new();
            cur_bb.live_out.iter().for_each(|e| {
                if e.is_virtual() && e.get_type() == kind {
                    ends.insert(e.get_id());
                }
            });
            for (index, inst) in cur_bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_use() {
                    if !reg.is_virtual() || reg.get_type() != kind {
                        continue;
                    }
                    if ends.contains(&reg.get_id()) {
                        continue;
                    }
                    ends.insert(reg.get_id());
                    if let None = sets_end.get(&(index as i32)) {
                        sets_end.insert(index as i32, HashSet::new());
                    }
                    sets_end
                        .get_mut(&(index as i32))
                        .unwrap()
                        .insert(reg.get_id());
                    end_poses.insert(reg.get_id(), index as i32);
                }
            }
            //println!("ends:{:?}",end_poses);
            end_poses.clear();
            cur_bb.live_in.iter().for_each(|e| {
                if e.is_virtual() && e.get_type() == kind {
                    livenow.insert(e.get_id());
                }
            });
            for (index, inst) in cur_bb.insts.iter().enumerate() {
                // 先与reg use冲突,然后消去终结的,然后与reg def冲突,并加上新的reg def
                //println!("{}",inst.as_ref());

                for reg in inst.get_reg_use() {
                    if !reg.is_virtual() || reg.get_type() != kind {
                        continue;
                    }
                    if livenow.contains(&reg.get_id()) {
                        continue;
                    }
                    for live in livenow.iter() {
                        if let None = interef_graph.get(live) {
                            interef_graph.insert(*live, HashSet::new());
                        }
                        if let None = interef_graph.get(&reg.get_id()) {
                            interef_graph.insert(reg.get_id(), HashSet::new());
                        }
                        interef_graph.get_mut(live).unwrap().insert(reg.get_id());
                        interef_graph.get_mut(&reg.get_id()).unwrap().insert(*live);
                    }
                }

                if let Some(finishes) = sets_end.get(&(index as i32)) {
                    for finish in finishes {
                        livenow.remove(finish);
                    }
                }

                for reg in inst.get_reg_def() {
                    if !reg.is_virtual() || reg.get_type() != kind {
                        continue;
                    }
                    if end_poses.contains_key(&reg.get_id()) {
                        continue;
                    }
                    livenow.insert(reg.get_id());
                    end_poses.insert(reg.get_id(), index as i32);
                    for live in livenow.iter() {
                        if *live == reg.get_id() {
                            continue;
                        }
                        if let None = interef_graph.get(live) {
                            interef_graph.insert(*live, HashSet::new());
                        }
                        if let None = interef_graph.get(&reg.get_id()) {
                            interef_graph.insert(reg.get_id(), HashSet::new());
                        }
                        interef_graph.get_mut(live).unwrap().insert(reg.get_id());
                        interef_graph.get_mut(&reg.get_id()).unwrap().insert(*live);
                    }
                }
                //println!("live now:{:?}",livenow);
                //println!("intereference :{:?}",interef_graph);
            }
            //println!("starts:{:?}",end_poses);
            for (k, v) in interef_graph {
                //println!("{k} interefer:");
                //println!("{:?}",v);
            }
        };

        while (!que.is_empty()) {
            let cur_bb = que.pop_front().unwrap();
            if passed.contains(&cur_bb) {
                continue;
            }
            passed.insert(cur_bb);
            // 把不同类型寄存器统计加入表中
            for inst in cur_bb.insts.iter() {
                for reg in inst.get_reg_def() {
                    // 统计使用cost
                    if !reg.is_virtual() {
                        continue;
                    } //TODO,可能调整去掉这里，加入对非虚拟寄存器的处理
                    self.costs_reg.insert(
                        reg.get_id(),
                        self.costs_reg.get(&reg.get_id()).unwrap_or(&0) + 1,
                    );
                    if passed_reg.contains(&reg) {
                        continue;
                    }
                    passed_reg.insert(reg);
                    // 给寄存器初始化可用寄存器集合,
                    self.regs_available.insert(reg.get_id(), RegUsedStat::new());
                    if reg.get_type() == ScalarType::Int {
                        self.i_regs.push(reg.get_id());
                    } else {
                        self.f_regs.push(reg.get_id());
                    }
                }
            }

            //分别处理浮点寄存器的情况和通用寄存器的情况
            process(cur_bb, &mut self.i_interference_graph, ScalarType::Int);
            process(cur_bb, &mut self.f_interference_graph, ScalarType::Float);
            // 加入还没有处理过的bb
            for bb_next in cur_bb.out_edge.iter() {
                if passed.contains(bb_next) {
                    continue;
                }
                que.push_back(*bb_next);
            }
        }

        // 查看简历的冲突图
        //println!("interference graph");
    }

    // 寻找最小度寄存器进行着色,作色成功返回true,着色失败返回true
    fn color(&mut self) -> bool {
        // TODO,优化执行速度,使用双向链表保存待着色点
        // 开始着色,着色直到无法再找到可着色的点为止,如果全部点都可以着色,返回true,否则返回false
        let mut out = true;
        // 对通用虚拟寄存器进行着色 (已经着色的点不用着色)
        if !self.i_alloc_finish {
            self.i_alloc_finish = true;
            for reg in self.i_regs.iter() {
                if self.icolors.contains_key(reg) {
                    continue;
                }
                //
                let color = self.regs_available.get(reg).unwrap().get_available_ireg();
                if let Some(color) = color {
                    self.icolors.insert(*reg, color);
                    // 修改周围的颜色
                    if let Some(neighbors) = self.i_interference_graph.get(reg) {
                        for neighbor in neighbors {
                            self.regs_available
                                .get_mut(neighbor)
                                .unwrap()
                                .use_ireg(color);
                        }
                    }
                } else {
                    self.i_alloc_finish = false;
                    out = false;
                    self.i_interence_reg = Some(*reg);
                    break;
                }
            }
        }
        // 对浮点寄存器进行着色
        if !self.f_alloc_finish {
            self.f_alloc_finish = true;
            // 对虚拟浮点寄存器进行着色
            for reg in self.f_regs.iter() {
                if self.fcolors.contains_key(reg) {
                    continue;
                }
                let color = self.regs_available.get(reg).unwrap().get_available_freg();
                if let Some(color) = color {
                    self.fcolors.insert(*reg, color);
                    if let Some(neighbors) = self.f_interference_graph.get(reg) {
                        for neighbor in neighbors {
                            self.regs_available
                                .get_mut(neighbor)
                                .unwrap()
                                .use_freg(color);
                        }
                    }
                } else {
                    self.f_alloc_finish = false;
                    out = false;
                    self.f_interence_reg = Some(*reg);
                    break;
                }
            }
        }
        out
    }

    // 简化成功返回true,简化失败返回falses
    fn simplify(&mut self) -> bool {
        // 简化方式
        // 处理产生冲突的寄存器
        // 方式,周围的寄存器变换颜色,直到该寄存器有不冲突颜色
        let mut out = true;
        let refresh = |reg: i32,
                       colors: &HashMap<i32, i32>,
                       regs_available: &mut HashMap<i32, RegUsedStat>,
                       interference_graph: &HashMap<i32, HashSet<i32>>,
                       kind: ScalarType| {
            // TODO,没有加异常处理,不能发现外部不合理的修改
            // 更新某个节点周围颜色
            if let Some(neighbors) = interference_graph.get(&reg) {
                let mut new_usestat = RegUsedStat::new();
                for neighbor in neighbors {
                    //TODO fix
                    if let Some(color) = colors.get(neighbor) {
                        if kind == ScalarType::Int {
                            new_usestat.use_ireg(*color)
                        } else if kind == ScalarType::Float {
                            new_usestat.use_freg(*color)
                        }
                    }
                }
                regs_available.insert(reg, new_usestat);
            }
        };
        // 简化成功返回true,简化失败返回false
        let mut one_simplify = |target_reg: i32,
                                colors: &mut HashMap<i32, i32>,
                                interference_graph: &mut HashMap<i32, HashSet<i32>>,
                                kind: ScalarType|
         -> bool {
            // 遍历目标寄存器的周围寄存器
            let neighbors = interference_graph.get(&target_reg).unwrap();
            let mut nums_colors: HashMap<i32, i32> = HashMap::new();
            //统计目标寄存器周围寄存器的颜色
            for reg in neighbors {
                if !colors.contains_key(reg) {
                    continue;
                }
                let color = colors.get(reg).unwrap();
                let num = nums_colors.get(color).unwrap_or(&0);
                nums_colors.insert(*color, num + 1);
            }
            // 对周围的寄存器进行颜色更换,
            for reg in neighbors {
                if !colors.contains_key(reg) {
                    continue;
                }
                let mut available: Vec<i32> = Vec::new();
                match kind {
                    ScalarType::Float => {
                        available = self.regs_available.get(reg).unwrap().get_rest_fregs()
                    }
                    ScalarType::Int => {
                        available = self.regs_available.get(reg).unwrap().get_rest_iregs()
                    }
                    _ => (),
                }
                if available.len() == 0 {
                    continue;
                }
                // 否则能够进行替换,
                let num = nums_colors.get(colors.get(reg).unwrap()).unwrap();
                // TODO,把先到先换做法改成贪心做法
                if *num == 1 {
                    // TODO
                    // 如果可以替换,更新自身颜色,更新周围节点的颜色
                    colors.insert(target_reg, *colors.get(reg).unwrap());
                    colors.insert(*reg, *available.get(0).unwrap());
                    refresh(
                        target_reg,
                        &colors,
                        &mut self.regs_available,
                        &interference_graph,
                        kind,
                    );
                    // 更新周围节点颜色
                    for reg2 in neighbors {
                        refresh(
                            *reg2,
                            &colors,
                            &mut self.regs_available,
                            &interference_graph,
                            kind,
                        );
                    }
                    return true;
                }
            }
            false
        };
        // 对通用寄存器的化简
        if let Some(i_reg) = self.i_interence_reg {
            if one_simplify(
                i_reg,
                &mut self.icolors,
                &mut self.i_interference_graph,
                ScalarType::Int,
            ) {
                self.i_interence_reg = None;
            } else {
                out = false;
            }
        }
        // 对浮点寄存器的化简
        if let Some(f_reg) = self.f_interence_reg {
            if one_simplify(
                f_reg,
                &mut self.fcolors,
                &mut self.f_interference_graph,
                ScalarType::Float,
            ) {
                self.f_interence_reg = None;
            } else {
                out = false;
            }
        }
        out //化简失败
    }

    // 简化失败后执行溢出操作,选择一个节点进行溢出,然后对已经作色节点进行重新分配
    fn spill(&mut self) {
        // 选择冲突寄存器或者是周围寄存器中的一个进行溢出,
        // 溢出的选择贪心: f=cost/degree.
        // 选择使得贪心函数最小的一个
        let cost = |reg: i32, inference_graph: &HashMap<i32, HashSet<i32>>| {
            return self.costs_reg.get(&reg).unwrap()
                / inference_graph.get(&reg).unwrap_or(&HashSet::new()).len() as i32;
        };
        // 寻找周围节点中spill后能够消解冲突的节点中 启发函数值最小的一个
        if let Some(ireg) = self.i_interence_reg {
            // 有没有可能消解一个元素之后仍然无法使得代码合理?,不可能
            // 消解选择,要么消解自身,要么消解其他
            let mut tospill = ireg;
            let mut heuoristic_of_spill = cost(tospill, &self.i_interference_graph); //待消解节点的启发函数值
            for reg in self.i_interference_graph.get(&ireg).unwrap() {
                let tmp_cost = cost(*reg, &self.i_interference_graph);
                if tmp_cost < heuoristic_of_spill {
                    heuoristic_of_spill = tmp_cost;
                }
            }
            self.spillings.insert(tospill);
        }
        if let Some(freg) = self.f_interence_reg {
            let mut tospill = freg;
            let mut heuoristic_of_spill = cost(tospill, &self.f_interference_graph); //待消解节点的启发函数值
            for reg in self.i_interference_graph.get(&freg).unwrap() {
                let tmp_cost = cost(*reg, &self.f_interference_graph);
                if tmp_cost < heuoristic_of_spill {
                    heuoristic_of_spill = tmp_cost;
                }
            }
            self.spillings.insert(tospill);
        }
    }
    // 返回分配结果
    fn alloc_register(&mut self) -> (HashSet<i32>, HashMap<i32, i32>) {
        let mut dstr: HashMap<i32, i32> = HashMap::new();
        // 返回分配结果,根据当前着色结果,
        for (vreg, icolor) in self.icolors.iter() {
            self.dstr.insert(*vreg, *icolor);
        }
        for (vreg, fcolor) in self.fcolors.iter() {
            self.dstr.insert(*vreg, *fcolor);
        }
        self.dstr.iter().for_each(|(k, v)| {
            dstr.insert(*k, *v);
        });
        (self.spillings.iter().cloned().collect(), dstr)
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {
        self.build_interference_graph(func);
        // TODO
        while !self.color() {
            if self.simplify() {
                continue;
            }
            self.spill();
        }
        let (spillings, dstr) = self.alloc_register();
        let (func_stack_size, bb_sizes) =
            crate::backend::regalloc::easy_ls_alloc::Allocator
            ::countStackSize(func, &spillings);

        //println!("dstr:{:?}",self.dstr);
        //println!("spillings:{:?}",self.spillings);

        FuncAllocStat {
            stack_size: func_stack_size,
            bb_stack_sizes: bb_sizes,
            spillings,
            dstr,
        }
    }
}
