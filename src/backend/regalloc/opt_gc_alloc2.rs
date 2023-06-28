// 最大化执行速度

use crate::{
    backend::{
        instrs::{Func, BB},
        operand::Reg,
    },
    container::bitmap::{self, Bitmap},
    log_file, log_file_uln,
    utility::{ObjPool, ObjPtr, ScalarType},
};
use biheap::core::BiHeap;
use core::panic;
use std::{
    collections::{hash_map::Iter, HashMap, HashSet, LinkedList, VecDeque},
    fmt::{self, format},
};

use super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};

#[derive(PartialEq)]
pub struct OperItem {
    reg: Reg,
    cost: f32, //对于color过程,该cost是邻接度(小优先),对于rescue过程,是spillcost的值(大优先,先拯救spill代价大的东西),
               // 对于spill过程来说,该cost是spillcost的值(小优先),
               //因为数据会发生改变,所以最好每轮更新一下数据
}
impl OperItem {
    pub fn new(reg: &Reg, cost: &f32) -> OperItem {
        OperItem {
            reg: *reg,
            cost: *cost,
        }
    }
}

impl Eq for OperItem {}

impl PartialOrd for OperItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for OperItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.cost < other.cost {
            std::cmp::Ordering::Less
        } else if (self.cost - other.cost).abs() < 10E-10 {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Greater
        }
    }
}
//

pub struct AllocatorInfo {
    pub k_graph: (BiHeap<OperItem>, Bitmap), //悬点集合,用来悬图优化,(临时悬点,可以用来切换颜色)
    pub to_simplify: BiHeap<OperItem>,       //准备化简保留的寄存器
    pub to_spill: BiHeap<OperItem>,          //待spill寄存器
    pub to_color: BiHeap<OperItem>,          //待着色寄存器
    pub last_colors: HashSet<Reg>,           //真正的弦点,永恒悬点
    pub spill_cost: HashMap<Reg, f32>,       //节点溢出代价 (用来启发寻找溢出代价最小的节点溢出)
    pub all_neighbors: HashMap<Reg, LinkedList<Reg>>, //所有邻居,在恢复节点的时候考虑,该表初始化后就不改变
    pub all_live_neighbors: HashMap<Reg, LinkedList<Reg>>, //还活着的邻居,在着色的时候动态考虑
    pub nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>>, //周围节点颜色数量
    pub availables: HashMap<Reg, RegUsedStat>,        //节点可着色资源
    pub colors: HashMap<i32, i32>,                    //着色情况
    pub spillings: HashSet<i32>,                      //溢出情况
}
#[derive(PartialEq, Eq)]
pub enum ActionResult {
    Finish,
    Unfinish,
    Success,
    Fail,
}

pub struct Allocator {
    info: Option<AllocatorInfo>,
}
impl Allocator {
    pub fn new() -> Allocator {
        Allocator { info: None }
    }

    pub fn init(&mut self, func: &Func) {
        let mut num_estimate_regs = func.num_regs();
        let mut ends_index_bb = regalloc::ends_index_bb(func);
        let mut allneighbors = regalloc::build_interference_into_lst(func, &ends_index_bb);
        let mut nums_neighbor_color = regalloc::build_nums_neighbor_color(func, &ends_index_bb);
        let mut availables = regalloc::build_availables(func, &ends_index_bb);
        let mut spill_cost = regalloc::estimate_spill_cost(func);
        let mut all_live_neigbhors: HashMap<Reg, LinkedList<Reg>> = HashMap::new();
        let mut last_colors: HashSet<Reg> = HashSet::new();
        let mut to_color: BiHeap<OperItem> = BiHeap::new();
        // 对live neighbor的更新,以及对tocolor的更新
        for (reg, neighbors) in &allneighbors {
            // 判断它是否是一个last colors,如果virtual neighbors< availables
            if reg.is_physic() {
                continue;
            }
            let num = availables
                .get(reg)
                .unwrap()
                .num_available_regs(reg.get_type());
            let mut num_v_neighbors = 0;
            for neighbor in neighbors {
                if neighbor.is_physic() {
                    continue;
                }
                num_v_neighbors += 1;
            }
            if num_v_neighbors < num {
                last_colors.insert(*reg);
            }
        }

        // 初始化tocolor以及k_graph
        for (reg, neighbors) in &allneighbors {
            if reg.is_physic() {
                continue;
            }
            let mut live_neighbors = LinkedList::new();
            for reg in neighbors {
                if reg.is_physic() {
                    continue;
                }
                if last_colors.contains(reg) {
                    continue;
                }
                live_neighbors.push_back(*reg);
            }
            to_color.push(OperItem::new(
                reg,
                &(*spill_cost.get(reg).unwrap() / (live_neighbors.len() as f32)),
            ));
            all_live_neigbhors.insert(*reg, live_neighbors);
        }

        let info = AllocatorInfo {
            to_color: BiHeap::new(),
            to_simplify: BiHeap::new(),
            to_spill: BiHeap::new(),
            k_graph: (BiHeap::new(), Bitmap::with_cap(num_estimate_regs / 8 + 1)),
            spill_cost: spill_cost,
            all_neighbors: allneighbors,
            nums_neighbor_color: nums_neighbor_color,
            availables: availables,
            colors: HashMap::new(),
            spillings: HashSet::new(),
            all_live_neighbors: all_live_neigbhors,
            last_colors: last_colors,
        };
        self.info = Some(info);
    }

    pub fn color(&mut self) -> ActionResult {
        // color度数最小的节点
        let mut out = ActionResult::Finish;
        loop {
            let info = self.info.as_mut().unwrap();
            if info.to_color.is_empty() {
                break;
            }
            let item = info.to_color.pop_max().unwrap();
            let reg = item.reg;

            // 判断该节点是否已经着色或者已经spill
            if self.if_has_been_spilled(&reg) || self.if_has_been_colored(&reg) {
                continue;
            }

            //TODO,把合适节点加入弦图
            //如果作色成功继续
            let (na, nn) = self.get_num_available_and_num_live_neighbor(&reg);
            if na > nn {
                self.info
                    .as_mut()
                    .unwrap()
                    .k_graph
                    .1
                    .insert(reg.bit_code() as usize);
                self.info.as_mut().unwrap().k_graph.0.push(item);
                continue;
            }

            // 如果不是加入弦图的点,先进行尝试着色,
            if self.color_one(&reg) {
                continue;
            }
            out = ActionResult::Unfinish;
            //如果着色失败,进行simplify流程,优先simplify spill cost大的过程
            self.info.as_mut().unwrap().to_simplify.push(item);
            break;
        }
        out
    }

    pub fn check_k_graph(&mut self) -> ActionResult {
        // 检查是否k_graph里面的值全部为真

        todo!()
    }

    pub fn simpilfy(&mut self) -> ActionResult {
        // 此处的simplify是简化color中color到的颜色
        let mut out = ActionResult::Success;
        // simpilfy,选择spill cost最大的一个
        loop {
            if self.info.as_ref().unwrap().to_simplify.is_empty() {
                break;
            }
            // 试图拯救to_rescue中spill代价最大的节点
            // 试图simplify来拯救当前节点
            let item = self.info.as_mut().unwrap().to_simplify.pop_max().unwrap();
            // 如果化简成功
            if self.simpilfy_one(item.reg) {
                self.info.as_mut().unwrap().to_color.push(item);
                continue;
            }
            out = ActionResult::Fail;
            self.info.as_mut().unwrap().to_spill.push(item);
            break;
        }
        out
    }

    pub fn spill(&mut self) -> ActionResult {
        let mut out = ActionResult::Fail;
        // sill 直到没有tospill或者直到出现新的可color的节点
        // spill先从 spillcost较小的,邻居度较大的开始
        loop {
            if self.info.as_ref().unwrap().to_simplify.is_empty() {
                break;
            }
            // 试图拯救to_rescue中spill代价最大的节点
            // 如果spill后能够出现可以着色的节点,则算spill成功,先结束这次spill
            let item = self.info.as_mut().unwrap().to_spill.pop_min().unwrap();
            //判断是否已经被拯救,
            let reg = item.reg;
            if self.if_has_been_colored(&reg) || self.if_has_been_spilled(&reg) {
                continue;
            }
            // 如果重新的spill完成
            let available = self.get_available(&reg);
            if available.is_available(item.reg.get_type()) {
                self.info.as_mut().unwrap().to_color.push(item);
                continue;
            }
            if self.spill_one(item.reg) {
                out = ActionResult::Success;
            }
        }
        out
    }

    pub fn color_k_graph(&mut self) -> ActionResult {
        todo!()
    }
    pub fn is_k_graph_node(&mut self) -> bool {
        todo!()
    }

    pub fn merge(&mut self) -> ActionResult {
        todo!()
    }

    #[inline]
    pub fn rescue(&mut self) -> ActionResult {
        todo!()
    }

    #[inline]
    pub fn color_last(&mut self) {
        // 着色最后的节点
        let last_colors = &self.info.as_ref().unwrap().last_colors;
        let spillings = &self.info.as_ref().unwrap().spillings;
        let dstr = &self.info.as_ref().unwrap().colors;
        let mut to_color: Vec<(i32, i32)> =
            Vec::with_capacity(self.info.as_ref().unwrap().last_colors.len());
        let interference_graph = &self.info.as_ref().unwrap().all_neighbors;
        for reg in last_colors {
            // 计算其available
            let mut reg_use_stat = RegUsedStat::new();
            for reg in interference_graph.get(&reg).unwrap() {
                if reg.is_physic() {
                    reg_use_stat.use_reg(reg.get_color());
                } else {
                    if spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    reg_use_stat.use_reg(*dstr.get(&reg.get_id()).unwrap());
                }
            }
            to_color.push((
                reg.get_id(),
                reg_use_stat.get_available_reg(reg.get_type()).unwrap(),
            ));
        }
        let dstr = &mut self.info.as_mut().unwrap().colors;
        for (reg, color) in to_color {
            dstr.insert(reg, color);
        }
    }

    #[inline]
    pub fn draw_dstr_spillings(&mut self) -> (HashMap<i32, i32>, HashSet<i32>) {
        // TODO,把to rescue中的内容交回spillings

        let dstr = self.info.as_ref().unwrap().colors.to_owned();
        let spillings = self.info.as_ref().unwrap().spillings.to_owned();
        (dstr, spillings)
    }

    #[inline]
    pub fn color_one(&mut self, reg: &Reg) -> bool {
        // TODO,选择最合适的颜色
        let available = self.get_available(&reg);
        let color = available.get_available_reg(reg.get_type());
        if let Some(color) = color {
            self.color_one_with_certain_color(reg, color);
            return true;
        }
        false
    }

    #[inline]
    pub fn simpilfy_one(&mut self, reg: Reg) -> bool {
        // 简化成功,该实例可以使用颜色,
        todo!()
    }

    #[inline]
    // 如果spill过程救活了一些节点,则返回true,否则返回false
    pub fn spill_one(&mut self, reg: Reg) -> bool {
        todo!()
    }

    #[inline]
    pub fn de_spill_one(&mut self, reg: &Reg) {}

    #[inline]
    pub fn swap_color(&mut self, reg1: Reg, reg2: Reg) {}

    #[inline]
    pub fn decolor_one(&mut self, reg: &Reg) {
        if self.if_has_been_spilled(reg) {
            panic!("unreachable!");
        }
        let color = self
            .info
            .as_mut()
            .unwrap()
            .colors
            .remove(&reg.get_id())
            .unwrap();
        let info = self.info.as_mut().unwrap();
        // 对于
        if let Some(neighbors) = info.all_live_neighbors.get(&reg) {
            for neighbor in neighbors {
                info.availables.get_mut(&neighbor).unwrap().use_reg(color);
                let nums_neighbor_color = info.nums_neighbor_color.get_mut(neighbor).unwrap();
                nums_neighbor_color
                    .insert(color, nums_neighbor_color.get(&color).unwrap_or(&0) - 1);
            }
        }
    }

    #[inline]
    pub fn color_one_with_certain_color(&mut self, reg: &Reg, color: i32) {
        if self.if_has_been_colored(reg) || self.if_has_been_colored(reg) {
            panic!("un reachable");
        }
        let info = self.info.as_mut().unwrap();
        info.colors.insert(reg.get_id(), color);

        if let Some(neighbors) = info.all_live_neighbors.get(&reg) {
            for neighbor in neighbors {
                info.availables.get_mut(&neighbor).unwrap().use_reg(color);
                let nums_neighbor_color = info.nums_neighbor_color.get_mut(neighbor).unwrap();
                nums_neighbor_color
                    .insert(color, nums_neighbor_color.get(&color).unwrap_or(&0) + 1);
            }
        }
    }

    #[inline]
    pub fn get_spill_cost_div_nn(&self, reg: &Reg) {}
    #[inline]
    pub fn get_spill_cost(&self, reg: &Reg) -> f32 {
        *self.info.as_ref().unwrap().spill_cost.get(reg).unwrap()
    }
    #[inline]
    pub fn if_has_been_spilled(&self, reg: &Reg) -> bool {
        self.info
            .as_ref()
            .unwrap()
            .spillings
            .contains(&reg.get_id())
    }
    #[inline]
    pub fn if_has_been_colored(&self, reg: &Reg) -> bool {
        self.info
            .as_ref()
            .unwrap()
            .colors
            .contains_key(&reg.get_id())
    }
    #[inline]
    pub fn get_available(&self, reg: &Reg) -> RegUsedStat {
        *self.info.as_ref().unwrap().availables.get(reg).unwrap()
    }
    #[inline]
    // 获取的可用颜色以及周围的活邻居数量
    pub fn get_num_available_and_num_live_neighbor(&self, reg: &Reg) -> (i32, i32) {
        let info = self.info.as_ref().unwrap();
        let na = info
            .availables
            .get(reg)
            .unwrap()
            .num_available_regs(reg.get_type());
        let nn = info.all_live_neighbors.len();
        (na as i32, nn as i32)
    }
}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {
        self.init(func);
        while !(self.color() == ActionResult::Finish)
            || self.check_k_graph() != ActionResult::Success
        {
            if self.simpilfy() == ActionResult::Success {
                // self.rescue();
                continue;
            }
            if self.spill() == ActionResult::Success {
                // self.rescue();
            }
        }
        self.color_k_graph();
        while self.merge() == ActionResult::Success {
            self.rescue();
        }
        self.color_last();
        let (dstr, spillings) = self.draw_dstr_spillings();
        let (func_stack_size, bb_sizes) = regalloc::countStackSize(func, &spillings);

        FuncAllocStat {
            dstr: dstr,
            spillings: spillings,
            stack_size: func_stack_size,
            bb_stack_sizes: bb_sizes,
        }
    }
}
