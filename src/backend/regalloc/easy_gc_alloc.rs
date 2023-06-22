// a impl of graph color register alloc algo

use crate::{
    backend::{
        instrs::{Func, BB},
        operand::Reg,
    },
    log_file,
    utility::{ObjPool, ObjPtr, ScalarType}, log_file_uln,
};
use core::panic;
use std::{
    collections::{HashMap, HashSet, LinkedList},
    fmt::{self, format},
};

use super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};


pub struct Allocator {
    i_regs: LinkedList<i32>,                   //所有虚拟通用寄存器的列表
    f_regs: LinkedList<i32>,                   //所有虚拟浮点寄存器列表
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
            i_regs: LinkedList::new(),
            f_regs: LinkedList::new(),
            icolors: HashMap::new(),
            fcolors: HashMap::new(),
            costs_reg: HashMap::new(),
            regs_available: HashMap::new(),
            i_interference_graph: HashMap::new(),
            f_interference_graph: HashMap::new(),
            i_alloc_finish: false,
            f_alloc_finish: false,
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
        // 定义处理函数
        let process = |cur_bb: ObjPtr<BB>,
                       interef_graph: &mut HashMap<i32, HashSet<i32>>,
                       kind: ScalarType| {
            if func.label == "main" {
                //println!("target");
            }
            let mut livenow: HashSet<i32> = HashSet::new();
            // 冲突分析
            let mut index_ends: HashMap<i32, HashSet<i32>> = HashMap::new();
            let mut passed_regs: HashSet<Reg> = HashSet::new();
            for (index, inst) in cur_bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_use() {
                    if passed_regs.contains(&reg) {
                        continue;
                    }
                    passed_regs.insert(reg);
                    if !reg.is_virtual() || reg.get_type() != kind {
                        continue;
                    }
                    if cur_bb.live_out.contains(&reg) {
                        continue;
                    }
                    if let None = index_ends.get(&(index as i32)) {
                        index_ends.insert(index as i32, HashSet::new());
                    }
                    index_ends
                        .get_mut(&(index as i32))
                        .unwrap()
                        .insert(reg.get_id());
                }
            }

            cur_bb.live_in.iter().for_each(|e| {
                if e.is_virtual() && e.get_type() == kind {
                    for live in livenow.iter() {
                        if let None = interef_graph.get(live) {
                            interef_graph.insert(*live, HashSet::new());
                        }
                        if let None = interef_graph.get(&e.get_id()) {
                            interef_graph.insert(e.get_id(), HashSet::new());
                        }
                        interef_graph.get_mut(live).unwrap().insert(e.get_id());
                        interef_graph.get_mut(&e.get_id()).unwrap().insert(*live);
                    }
                    livenow.insert(e.get_id());
                }
            });
            for (index, inst) in cur_bb.insts.iter().enumerate() {
                // 先与reg use冲突,然后消去终结的,然后与reg def冲突,并加上新的reg def
                if let Some(finishes) = index_ends.get(&(index as i32)) {
                    for finish in finishes {
                        livenow.remove(finish);
                    }
                }

                for reg in inst.get_reg_def() {
                    if !reg.is_virtual() || reg.get_type() != kind {
                        continue;
                    }
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
                    livenow.insert(reg.get_id());
                }
                //println!("live now:{:?}",livenow);
                //println!("intereference :{:?}",interef_graph);
            }
        };

        // 遍历所有块，分析冲突关系
        for cur_bb in func.blocks.iter() {
            let cur_bb = *cur_bb;
            // 把不同类型寄存器统计加入表中
            //分别处理浮点寄存器的情况和通用寄存器的情况
            process(cur_bb, &mut self.i_interference_graph, ScalarType::Int);
            process(cur_bb, &mut self.f_interference_graph, ScalarType::Float);
            // 加入还没有处理过的bb
        }

        let mut passed_reg: HashSet<Reg> = HashSet::new();
        // 初始化虚拟寄存器的剩余可用物理寄存器表
        for cur_bb in func.blocks.iter() {
            for inst in cur_bb.insts.iter() {
                for reg in inst.get_reg_def() {
                    // 统计使用cost
                    if !reg.is_virtual() {
                        continue;
                    } //TODO,可能调整去掉这里，加入对非虚拟寄存器的处理
                    if passed_reg.contains(&reg) {
                        continue;
                    }
                    passed_reg.insert(reg);
                    // 给寄存器初始化可用寄存器集合,
                    self.regs_available.insert(reg.get_id(), RegUsedStat::new());
                    if reg.get_type() == ScalarType::Int {
                        self.i_regs.push_back(reg.get_id());
                    } else {
                        self.f_regs.push_back(reg.get_id());
                    }
                }
            }
        }
        // 查看简历的冲突图
        //println!("interference graph");
    }

    // 对每个虚拟寄存器的spill代价估计
    fn count_spill_costs(&mut self, func: &Func) {
        self.costs_reg = regalloc::count_spill_confict(func);
    }

    // 寻找最小度寄存器进行着色,作色成功返回true,着色失败返回false
    fn color(&mut self) -> bool {
        // TODO,优化执行速度,使用双向链表保存待着色点
        // 开始着色,着色直到无法再找到可着色的点为止,如果全部点都可以着色,返回true,否则返回false
        let mut out = true;
        // 对通用虚拟寄存器进行着色 (已经着色的点不用着色)
        while !self.i_regs.is_empty() {
            let reg = *self.i_regs.front().unwrap();
            if self.icolors.contains_key(&reg)||self.spillings.contains(&reg) {
                self.i_regs.pop_front();
                continue;
            }
            // TODO ,使用self.colors
            let ok = self.color_one(reg, ScalarType::Int);
            if ok {
                self.i_regs.pop_front();
            } else {
                out=false;
                Allocator::debug("interef", [reg].into_iter().collect());
                self.i_interence_reg = Some(reg);
                break;
            }
        }
        
        while !self.f_regs.is_empty() {
            let reg = *self.f_regs.front().unwrap();
            if self.fcolors.contains_key(&reg) ||self.spillings.contains(&reg){
                self.f_regs.pop_front();
                continue;
            }
            // TODO
            let ok = self.color_one(reg, ScalarType::Float);
            if ok {
                self.f_regs.pop_front();
            } else {
                out=false;
                self.f_interence_reg = Some(reg);
                break;
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
        // 简化成功返回true,简化失败返回false化简
        if let Some(i_reg) = self.i_interence_reg {
            if self.simplify_one(i_reg, ScalarType::Int) {
                self.i_interence_reg=None;
            }else{
                out=false;
            }
        }
        // 对浮点寄存器的化简
        if let Some(f_reg) = self.f_interence_reg {
            if self.simplify_one(f_reg, ScalarType::Float){
                self.f_interence_reg=None;
            }else{
                out=false;
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
        // 溢出寄存器之后更新节点周围节点颜色

        let spill_one = |tospill: i32,
                         spillings: &mut HashSet<i32>,
                         colors: &mut HashMap<i32, i32>,
                         interference_graph: &HashMap<i32, HashSet<i32>>| {
            if tospill==43||tospill==42{
                println!("gg");
            }
            let mut tospill = tospill;
            let mut heuoristic_of_spill = cost(tospill, &self.i_interference_graph); //待消解节点的启发函数值
            let inters = interference_graph.get(&tospill).unwrap();
            for reg in inters {
                if spillings.contains(reg) {
                    continue;
                }
                if !colors.contains_key(reg) {
                    continue;
                } //没有着色的寄存器无法选择溢出
                if *reg <= 31 {
                    continue;
                } //物理寄存器无法溢出
                let tmp_cost = cost(*reg, interference_graph);
                if tmp_cost < heuoristic_of_spill {
                    heuoristic_of_spill = tmp_cost;
                    tospill = *reg;
                }
            }
            if tospill==43||tospill==42{
                println!("gg");
            }
            Allocator::debug("spill", [tospill].into_iter().collect());
            colors.remove(&tospill);
            spillings.insert(tospill);
        };
        // 寻找周围节点中spill后能够消解冲突的节点中 启发函数值最小的一个
        if let Some(ireg) = self.i_interence_reg {
            spill_one(
                ireg,
                &mut self.spillings,
                &mut self.icolors,
                &self.i_interference_graph,
            );
            self.i_interence_reg = None;
        }

        if let Some(freg) = self.f_interence_reg {
            spill_one(
                freg,
                &mut self.spillings,
                &mut self.fcolors,
                &self.f_interference_graph,
            );
            self.f_interence_reg = None;
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
        println!("{:?}",self.spillings);
        (self.spillings.iter().cloned().collect(), dstr)
    }


    fn color_one(&mut self,reg:i32,kind:ScalarType)->bool {
        let (mut colors,mut availables,interference_graph)=match kind {
            ScalarType::Float=>(&mut self.fcolors,&mut self.regs_available,&mut self.f_interference_graph),
            ScalarType::Int=>(&mut self.icolors,&mut self.regs_available,&mut self.i_interference_graph),
            _=>panic!(),
        };
        if reg==42||reg==43 {
            // debug
            println!("G1");
        }
        let available = availables.get(&reg).unwrap();
        let color = match kind {
            ScalarType::Float => available.get_available_freg(),
            ScalarType::Int => available.get_available_ireg(),
            _ => None,
        };
        if color.is_none() {
            return false;
        }
        let color = color.unwrap();
        Allocator::debug("color", [reg,color].into_iter().collect());
        colors.insert(reg, color);
        if let Some(neighbors) = interference_graph.get(&reg){
            for neighbor in neighbors{
                if self.spillings.contains(&neighbor) {
                    continue;
                }
                availables.get_mut(&neighbor).unwrap().use_reg(color);
            }
        }
        return true;
    }

    fn refresh(&mut self, reg: i32, kind: ScalarType) {
        let (colors, regs_available, interference_graph) = match kind {
            ScalarType::Float => (
                &self.fcolors,
                &mut self.regs_available,
                &self.f_interference_graph,
            ),
            ScalarType::Int => (
                &self.icolors,
                &mut self.regs_available,
                &self.i_interference_graph,
            ),
            _ => panic!(),
        };
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
    }

    fn simplify_one(&mut self,target_reg:i32,kind:ScalarType)->bool{
        let (colors,interference_graph)=match kind {
            ScalarType::Float=>(&mut self.fcolors,&mut self.f_interference_graph),
            ScalarType::Int=>(&mut self.icolors,&mut self.i_interference_graph),
            _=>panic!("unlegal reg"),
        };
        let mut out=false;
        // 遍历目标寄存器的周围寄存器
        let neighbors = interference_graph.get(&target_reg).unwrap();
        let mut nums_colors: HashMap<i32, i32> = HashMap::new();
        let mut to_refresh:Vec<i32>=Vec::new();
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
        for neighbor in neighbors { 
            if *neighbor<=31 {continue;}    //对于物理寄存器不能进行颜色交换
            if self.spillings.contains(neighbor) {continue;}
            if !colors.contains_key(neighbor) {
                continue;
            }
            let color=colors.get(neighbor).unwrap();
            if *nums_colors.get(color).unwrap()>1 {
                continue;
            }
            let available=self.regs_available.get_mut(neighbor).unwrap();
            available.use_reg(*color);
            let rest=match kind {
                ScalarType::Float=>{
                    available.get_rest_fregs()
                },
                ScalarType::Int=>{
                    available.get_rest_iregs()
                },
                _=>panic!("un match reg type!"),
            };
            if rest.len()<=0 {
                available.release_reg(*color);
                continue;
            }   
            out=true;
            Allocator::debug("simplify", [target_reg,*color,*neighbor,*rest.get(0).unwrap()].into_iter().collect());
            // TODO
            colors.insert(target_reg, *color);
            colors.insert(*neighbor, *rest.get(0).unwrap());
            to_refresh.push(target_reg);
            to_refresh.push(*neighbor);
            neighbors.iter().for_each(|reg|if reg!=neighbor {to_refresh.push(*reg)});
            interference_graph.get(&target_reg).unwrap().iter().for_each(|reg|if *reg!=target_reg { to_refresh.push(*reg)});
            break;
        }
        if out {
            to_refresh.iter().for_each(|reg|self.refresh(*reg, kind));
        }
        out
    }

    fn debug(kind:&'static str,regs:Vec<i32>){
        let color_spill_path="color_spill.txt";
        match kind {
            "interef"=>log_file!(color_spill_path,"inter:{}",regs.get(0).unwrap()),
            "spill"=>log_file!(color_spill_path,"tospill:{}",regs.get(0).unwrap()),
            "color"=>log_file!(color_spill_path,"color:{}({})",regs.get(0).unwrap(),regs.get(1).unwrap()),
            "simplify"=>log_file!(color_spill_path,"simplify:{}({}),{}({})",regs.get(0).unwrap(),regs.get(1).unwrap(),regs.get(2).unwrap(),regs.get(3).unwrap()),
            _=>panic!("unleagal debug"),
        };
    }

}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {
        self.build_interference_graph(func);
        // for r in self.i_regs.iter() {
        //     if *r==57||*r==66{
        //         let a=2;
        //     }
        // }
        self.count_spill_costs(func);
        // 打印冲突图
        let intereref_path = "interference_graph.txt";
        // self.i_interference_graph
        //     .iter()
        //     .for_each(|(reg, neighbors)| {
        //         // log_file_uln!(intereref_path, "node {reg}\n{{");
        //         // neighbors.iter().for_each(|neighbor| log_file_uln!(intereref_path,"({},{})",reg,neighbor));
        //         // log_file!(intereref_path,"}}\n");
        //     });

        // TODO
        while !self.color() {
            if self.simplify() {
                continue;
            }
            self.spill();
        }
        let (spillings, dstr) = self.alloc_register();
        let (func_stack_size, bb_sizes) = regalloc::countStackSize(func, &spillings);

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
