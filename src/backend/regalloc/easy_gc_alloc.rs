// a impl of graph color register alloc algo

use crate::{
    backend::{
        instrs::{Func, BB},
        operand::Reg,
    },
    container::bitmap::{self, Bitmap},
    log_file, log_file_uln,
    utility::{ObjPool, ObjPtr, ScalarType},
};
use core::panic;
use std::{
    collections::{HashMap, HashSet, LinkedList, VecDeque},
    fmt::{self, format},
    hash::Hash,
};

use super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};

pub struct Allocator {
    pub regs: LinkedList<Reg>,                 //所有虚拟寄存器的列表
    pub colors: HashMap<i32, i32>,             // 保存着色结果
    pub costs_reg: HashMap<Reg, f32>,          //记录虚拟寄存器的使用次数(作为代价)
    pub availables: HashMap<Reg, RegUsedStat>, // 保存每个点的可用寄存器集合
    pub nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>>,
    pub ends_index_bb: HashMap<(i32, ObjPtr<BB>), HashSet<Reg>>,
    pub interference_regs: HashSet<Reg>,
    pub interference_graph: HashMap<Reg, HashSet<Reg>>, //浮点寄存器冲突图
    pub spillings: HashSet<i32>,                        //记录溢出寄存器
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            regs: LinkedList::new(),
            colors: HashMap::new(),
            costs_reg: HashMap::new(),
            availables: HashMap::new(),
            interference_graph: HashMap::new(),
            interference_regs: HashSet::new(),
            spillings: HashSet::new(),
            nums_neighbor_color: HashMap::new(),
            ends_index_bb: HashMap::new(),
        }
    }

    // 判断两个虚拟寄存器是否是通用寄存器分配冲突
    // 建立虚拟寄存器之间的冲突图
    pub fn build_interference_graph(&mut self, func: &Func) {
        self.ends_index_bb = regalloc::build_ends_index_bb(func);
        let ends_index_bb = &self.ends_index_bb;
        self.interference_graph = regalloc::build_interference(func, &ends_index_bb);
        self.availables = regalloc::build_availables(func, &self.interference_graph);
        self.nums_neighbor_color = regalloc::build_nums_neighbor_color(func, ends_index_bb);
        let mut bitmap: Bitmap = Bitmap::with_cap(5000);
        let tmp_set: HashSet<Reg> = HashSet::new();
        // 建立待分配寄存器图 和 并统计悬挂点
        for cur_bb in func.blocks.iter() {
            for inst in cur_bb.insts.iter() {
                // 统计所有出现过的寄存器，包括物理寄存器和虚拟寄存器,初始化它们的可用表
                for reg in inst.get_regs() {
                    let bit_code = reg.bit_code();
                    if bitmap.contains(bit_code as usize) {
                        continue;
                    }
                    bitmap.insert(bit_code as usize);
                    self.regs.push_back(reg);
                }
            }
        }
        // 查看简历的冲突图
    }

    // 对每个虚拟寄存器的spill代价估计
    pub fn count_spill_costs(&mut self, func: &Func) {
        self.costs_reg = regalloc::estimate_spill_cost(func);
        let path = "reg_spill_costs.txt";
        log_file!(path, "func:{}", func.label);
        let mut i = 0;
        let reline = 20;
        self.costs_reg.iter().for_each(|(reg, cost)| {
            if i % reline == 19 {
                log_file!(path, "");
            }
            log_file_uln!(path, "{}:{},", reg, cost);
        });
    }

    // 寻找最小度寄存器进行着色,作色成功返回true,着色失败返回false
    pub fn color(&mut self) -> bool {
        // 开始着色,着色直到无法再找到可着色的点为止,如果全部点都可以着色,返回true,否则返回false
        let mut out = true;
        // 对通用虚拟寄存器进行着色 (已经着色的点不用着色)
        // FIXME,使用自定义的Deque的cursor api重构
        while !self.regs.is_empty() {
            let reg = *self.regs.front().unwrap();
            if !reg.is_virtual()
                || self.spillings.contains(&reg.get_id())
                || self.colors.contains_key(&reg.get_id())
            {
                self.regs.pop_front();
                continue;
            }
            let ok = self.color_one(reg);
            if ok {
                self.regs.pop_front();
            } else {
                out = false;
                Allocator::debug("interef", [&reg].into_iter().collect());
                self.interference_regs.insert(reg);
                break;
            }
        }

        out
    }

    // 简化成功返回true,简化失败返回falses
    pub fn simplify(&mut self) -> bool {
        // 简化方式
        // 处理产生冲突的寄存器
        // 方式,周围的寄存器变换颜色,直到该寄存器有不冲突颜色
        let mut out = false;

        // 不断化简,直到不能化简
        while !self.regs.is_empty() {
            let reg = *self.regs.front().unwrap();
            if !reg.is_virtual()
                || self.spillings.contains(&reg.get_id())
                || self.colors.contains_key(&reg.get_id())
            {
                self.regs.pop_front();
                continue;
            }
            if self.interference_regs.contains(&reg) {
                let ok = self.simplify_one(&reg);
                if ok {
                    self.regs.pop_front();
                    self.interference_regs.remove(&reg);
                    return true;
                }
            }
            break;
        }
        out //化简失败
    }

    // 简化失败后执行溢出操作,选择一个节点进行溢出,然后对已经作色节点进行重新分配
    pub fn spill(&mut self) {
        // 溢出寄存器之后更新节点周围节点颜色
        while !self.regs.is_empty() {
            let reg = *self.regs.front().unwrap();
            if !reg.is_virtual()
                || self.spillings.contains(&reg.get_id())
                || self.colors.contains_key(&reg.get_id())
            {
                self.regs.pop_front();
                continue;
            }
            if self.interference_regs.contains(&reg) {
                self.spill_one(reg);
                self.interference_regs.clear();
            }
            break;
        }
    }

    // 返回分配结果
    pub fn alloc_register(&mut self) -> (HashSet<i32>, HashMap<i32, i32>) {
        let mut dstr: HashMap<i32, i32> = HashMap::new();
        // 返回分配结果,根据当前着色结果,
        for (vreg, color) in self.colors.iter() {
            dstr.insert(*vreg, *color);
        }
        // println!("{:?}",self.spillings);
        (self.spillings.iter().cloned().collect(), dstr)
    }

    pub fn color_one(&mut self, reg: Reg) -> bool {
        let (colors, availables, interference_graph) = (
            &mut self.colors,
            &mut self.availables,
            &self.interference_graph,
        );
        if !reg.is_virtual() {
            panic!("try to color un virtual reg");
        }
        if self.spillings.contains(&reg.get_id()) {
            panic!("try to color spilling v reg");
        }
        let available = availables.get(&reg).unwrap();
        let color = available.get_available_reg(reg.get_type());
        if color.is_none() {
            return false;
        }
        let color = color.unwrap();
        Allocator::debug(
            "color",
            [&reg, &Reg::new(color, reg.get_type())]
                .into_iter()
                .collect(),
        );
        self.color_one_with_certain_color(reg, color);
        // colors.insert(reg.get_id(), color);
        // if let Some(neighbors) = interference_graph.get(&reg) {
        //     for neighbor in neighbors {
        //         availables.get_mut(&neighbor).unwrap().use_reg(color);
        //         let nums_neighbor_color = self.nums_neighbor_color.get_mut(neighbor).unwrap();
        //         nums_neighbor_color
        //             .insert(color, nums_neighbor_color.get(&color).unwrap_or(&0) + 1);
        //     }
        // }
        return true;
    }

    pub fn color_one_with_certain_color(&mut self, reg: Reg, color: i32) {
        if self.spillings.contains(&reg.get_id()) {
            panic!("gg");
        }
        self.colors.insert(reg.get_id(), color);
        if let Some(neighbors) = self.interference_graph.get(&reg) {
            for neighbor in neighbors {
                self.availables.get_mut(&neighbor).unwrap().use_reg(color);
                let nums_neighbor_color = self.nums_neighbor_color.get_mut(neighbor).unwrap();
                nums_neighbor_color
                    .insert(color, nums_neighbor_color.get(&color).unwrap_or(&0) + 1);
            }
        }
    }

    // 移除一个节点的颜色
    pub fn decolor_one(&mut self, reg: Reg) -> bool {
        if !reg.is_virtual() {
            panic!("try to color un virtual reg");
        }
        if self.spillings.contains(&reg.get_id()) {
            panic!("try to color spilling v reg");
        }
        let color = *self.colors.get(&reg.get_id()).unwrap();
        //
        self.colors.remove(&reg.get_id());
        Allocator::debug(
            "decolor",
            [&reg, &Reg::new(color, reg.get_type())]
                .into_iter()
                .collect(),
        );
        if let Some(neighbors) = self.interference_graph.get(&reg) {
            for neighbor in neighbors {
                let nums_neighbor_color = self.nums_neighbor_color.get_mut(neighbor).unwrap();
                let new_num = nums_neighbor_color.get(&color).unwrap() - 1;
                nums_neighbor_color.insert(color, new_num);
                if new_num == 0 {
                    self.availables
                        .get_mut(&neighbor)
                        .unwrap()
                        .release_reg(color);
                }
            }
        }
        true
    }

    //
    pub fn spill_one(&mut self, reg: Reg) {
        // 选择冲突寄存器或者是周围寄存器中的一个进行溢出,
        // 溢出的选择贪心: f=cost/degree.
        // 选择使得贪心函数最小的一个
        let spillings = &mut self.spillings;
        let colors = &mut self.colors;
        let interference_graph = &self.interference_graph;
        let cost = |reg: &Reg| {
            return *self.costs_reg.get(reg).unwrap()
                / interference_graph.get(reg).unwrap_or(&HashSet::new()).len() as f32;
        };
        let mut tospill: Reg = reg;
        let mut heuoristic_of_spill = cost(&tospill); //待消解节点的启发函数值
        let inters = interference_graph.get(&tospill).unwrap();
        for reg in inters {
            if !reg.is_virtual() {
                continue;
            }
            if spillings.contains(&reg.get_id()) {
                continue;
            }

            // TODO ,比较效果，判断是否应该把有颜色的寄存器也纳入spill范围内
            if !colors.contains_key(&reg.get_id()) {
                continue;
            } //没有着色的寄存器无法选择溢出
              //物理寄存器无法溢出
            let tmp_cost = cost(reg);
            if tmp_cost < heuoristic_of_spill {
                heuoristic_of_spill = tmp_cost;
                tospill = *reg;
            }
        }
        // TODO，使用更好的启发函数
        Allocator::debug("spill", [tospill].iter().collect());
        if self.colors.contains_key(&tospill.get_id()) {
            self.decolor_one(tospill);
        }
        self.spillings.insert(tospill.get_id());
    }

    pub fn simplify_one(&mut self, target_reg: &Reg) -> bool {
        let (colors, interference_graph) = (&mut self.colors, &self.interference_graph);
        let mut out = false;
        // 遍历目标寄存器的周围寄存器
        let neighbors = interference_graph.get(&target_reg).unwrap();
        let nums_colors = self.nums_neighbor_color.get_mut(&target_reg).unwrap();
        //统计目标寄存器周围寄存器的颜色
        // 对周围的寄存器进行颜色更换,
        // TODO,使用贪心算法选择最适合用来颜色替换的节点
        for neighbor in neighbors {
            let neighbor = *neighbor;
            let target_reg = *target_reg;
            if !neighbor.is_virtual() {
                continue;
            }
            if self.spillings.contains(&neighbor.get_id()) {
                continue;
            }
            if !colors.contains_key(&neighbor.get_id()) {
                continue;
            }
            let color = colors.get(&neighbor.get_id()).unwrap();
            if *nums_colors.get(color).unwrap() > 1 {
                continue;
            }
            if self
                .availables
                .get(&neighbor)
                .unwrap()
                .num_available_regs(target_reg.get_type())
                <= 1
            {
                continue;
            }
            // TODO, 检查,
            out = true;
            self.decolor_one(neighbor);
            self.color_one(target_reg);
            self.color_one(neighbor);
            break;
        }
        out
    }

    pub fn debug(kind: &'static str, regs: Vec<&Reg>) {
        let color_spill_path = "color_spill.txt";
        match kind {
            "interef" => log_file!(color_spill_path, "inter:{}", regs.get(0).unwrap()),
            "spill" => log_file!(color_spill_path, "tospill:{}", regs.get(0).unwrap()),
            "color" => log_file!(
                color_spill_path,
                "color:{}({})",
                regs.get(0).unwrap(),
                regs.get(1).unwrap()
            ),
            "decolor" => log_file!(
                color_spill_path,
                "decolor:{}({})",
                regs.get(0).unwrap(),
                regs.get(1).unwrap()
            ),
            "simplify" => log_file!(
                color_spill_path,
                "simplify:{}({}),{}({})",
                regs.get(0).unwrap(),
                regs.get(1).unwrap(),
                regs.get(2).unwrap(),
                regs.get(3).unwrap()
            ),
            _ => panic!("unleagal debug"),
        };
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {
        self.build_interference_graph(func);
        self.count_spill_costs(func);
        // 打印冲突图
        let intereref_path = "interference_graph.txt";
        self.interference_graph.iter().for_each(|(reg, neighbors)| {
            log_file_uln!(intereref_path, "node {reg}\n{{");
            neighbors
                .iter()
                .for_each(|neighbor| log_file_uln!(intereref_path, "({},{})", reg, neighbor));
            log_file!(intereref_path, "}}\n");
        });

        // TODO,加入化简后单步合并检查 以及 分配完成后合并检查
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
