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
extern crate biheap;
use biheap::core::BiHeap;
use core::{panic, time};
use std::{
    collections::{hash_map::Iter, HashMap, HashSet, LinkedList, VecDeque},
    fmt::{self, format},
    panic::PanicInfo,
    ptr::addr_of_mut,
};

use super::super::{
    regalloc::{self, Regalloc},
    structs::{FuncAllocStat, RegUsedStat},
};

use super::*;

// #[derive(PartialEq, Clone, Copy)]
// pub struct OperItem {
//     reg: Reg,
//     cost: f32, //对于color过程,该cost是邻接度(小优先),对于rescue过程,是spillcost的值(大优先,先拯救spill代价大的东西),
//                // 对于spill过程来说,该cost是spillcost的值(小优先),
//                //因为数据会发生改变,所以最好每轮更新一下数据
// }
// impl OperItem {
//     pub fn new(reg: &Reg, cost: &f32) -> OperItem {
//         OperItem {
//             reg: *reg,
//             cost: *cost,
//         }
//     }
// }

// impl Eq for OperItem {}

// impl PartialOrd for OperItem {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }
// impl Ord for OperItem {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         if self.cost < other.cost {
//             std::cmp::Ordering::Less
//         } else if (self.cost - other.cost).abs() < 10E-10 {
//             std::cmp::Ordering::Equal
//         } else {
//             std::cmp::Ordering::Greater
//         }
//     }
// }
// //

// pub struct AllocatorInfo {
//     pub k_graph: (BiHeap<OperItem>, Bitmap), //悬点集合,用来悬图优化,(临时悬点,可以用来切换颜色)
//     pub to_simplify: BiHeap<OperItem>,       //准备化简保留的寄存器
//     pub to_spill: BiHeap<OperItem>,          //待spill寄存器
//     pub to_color: BiHeap<OperItem>,          //待着色寄存器
//     pub colored: BiHeap<OperItem>,           //已着色寄存器
//     pub last_colors: HashSet<Reg>,           //真正的弦点,永恒悬点
//     pub spill_cost: HashMap<Reg, f32>,       //节点溢出代价 (用来启发寻找溢出代价最小的节点溢出)
//     pub all_neighbors: HashMap<Reg, LinkedList<Reg>>, //所有邻居,在恢复节点的时候考虑,该表初始化后就不改变
//     pub all_live_neighbors: HashMap<Reg, LinkedList<Reg>>, //还活着的邻居,在着色的时候动态考虑
//     pub all_live_neigbhors_bitmap: HashMap<Reg, Bitmap>, //记录还活着的邻居 TODO,
//     pub nums_neighbor_color: HashMap<Reg, HashMap<i32, i32>>, //周围节点颜色数量
//     pub availables: HashMap<Reg, RegUsedStat>,        //节点可着色资源
//     pub colors: HashMap<i32, i32>,                    //着色情况
//     pub spillings: HashSet<i32>,                      //溢出情况
// }
// #[derive(PartialEq, Eq)]
// pub enum ActionResult {
//     Finish,
//     Unfinish,
//     Success,
//     Fail,
// }

// pub struct Allocator {
//     info: Option<AllocatorInfo>,
// }

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
        let mut all_live_neighbors_bitmap: HashMap<Reg, Bitmap> = HashMap::new();
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
            let mut live_neigbhors_bitmap = Bitmap::with_cap(10);
            for reg in neighbors {
                if reg.is_physic() {
                    continue;
                }
                if last_colors.contains(reg) {
                    continue;
                }
                live_neighbors.push_back(*reg);
                live_neigbhors_bitmap.insert(reg.bit_code() as usize);
            }
            to_color.push(OperItem::new(
                reg,
                &(*spill_cost.get(reg).unwrap() / (live_neighbors.len() as f32)),
            ));
            all_live_neigbhors.insert(*reg, live_neighbors);
            all_live_neighbors_bitmap.insert(*reg, live_neigbhors_bitmap);
        }

        let info = AllocatorInfo {
            to_color: BiHeap::new(),
            to_simplify: BiHeap::new(),
            to_spill: BiHeap::new(),
            colored: BiHeap::new(),
            k_graph: (BiHeap::new(), Bitmap::with_cap(num_estimate_regs / 8 + 1)),
            spill_cost: spill_cost,
            all_neighbors: allneighbors,
            nums_neighbor_color: nums_neighbor_color,
            availables: availables,
            colors: HashMap::new(),
            spillings: HashSet::new(),
            all_live_neighbors: all_live_neigbhors,
            last_colors: last_colors,
            all_live_neigbhors_bitmap: all_live_neighbors_bitmap,
        };
        self.info = Some(info);
    }

    /// color:选择一个合适的颜色进行着色
    /// * 如果着色成功,把项目加入到colored中
    /// * 如果着色失败了,把项目加入到to_simplify中
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
                // todo,修改k_color_neigbhor中节点衡量的方法
                self.info.as_mut().unwrap().k_graph.0.push(item);
                continue;
            }
            // 如果不是加入弦图的点,先进行尝试着色,
            if self.color_one(&reg) {
                out = ActionResult::Success;
                self.info.as_mut().unwrap().colored.push(item);
            } else {
                out = ActionResult::Fail;
                self.info.as_mut().unwrap().to_simplify.push(item);
            }
            break;
        }
        out
    }

    // 检查是否当前k_graph中的节点都已经是合理的节点
    pub fn check_k_graph(&mut self) -> ActionResult {
        // 检查是否k_graph里面的值全部为真
        let mut out = ActionResult::Finish;
        let mut new_biheap: BiHeap<OperItem> = BiHeap::new();
        loop {
            if self.info.as_ref().unwrap().k_graph.0.len() == 0 {
                break;
            }
            let item = self.info.as_mut().unwrap().k_graph.0.pop_min().unwrap();
            let map = &self.info.as_ref().unwrap().k_graph.1;
            if !map.contains(item.reg.bit_code() as usize) {
                // 如果不在k graph中了,则继续
                continue;
            }
            let reg = item.reg;
            if !self.is_k_graph_node(&reg) {
                out = ActionResult::Unfinish;
                let new_item = self.draw_spill_div_nlc_item(&reg);
                self.info.as_mut().unwrap().to_color.push(new_item);
                continue;
            }
            let new_item = self.draw_spill_div_nlc_item(&reg);
            new_biheap.push(new_item);
        }
        if self.info.as_ref().unwrap().k_graph.0.len() == 0 {
            self.info.as_mut().unwrap().k_graph.0 = new_biheap;
        } else {
            new_biheap.iter().for_each(|item| {
                self.info.as_mut().unwrap().k_graph.0.push(*item);
            });
        }
        out
    }

    /// 简化
    /// * 简化to_simpilify列表中的一个最优简化项目
    /// * 简化成功返回Success,并且把simplify对象加回 to_color
    /// * 如果to_simpilfy列表为空返回Finish
    /// * 简化失败返回Fail,把simplify对象加入 to_spill
    pub fn simpilfy(&mut self) -> ActionResult {
        // 此处的simplify是简化color中color到的颜色
        // simpilfy,选择spill cost最大的一个
        if self.info.as_ref().unwrap().to_simplify.is_empty() {
            return ActionResult::Finish;
        }
        // 试图拯救to_rescue中spill代价最大的节点
        // 试图simplify来拯救当前节点
        let item = self.info.as_mut().unwrap().to_simplify.pop_max().unwrap();
        // 如果化简成功,返回true
        if self.simpilfy_one(item.reg) {
            self.info.as_mut().unwrap().to_color.push(item);
            return ActionResult::Success;
        }
        self.info.as_mut().unwrap().to_spill.push(item);
        return ActionResult::Fail;
    }

    /// 溢出
    /// * 从待溢出列表中选择一个最优溢出项目进行溢出处理
    /// * 如果溢出列表为空,返回Finish
    /// * 溢出成功返回Success  (溢出是肯定能够成功的)
    /// * 溢出失败返回Fail (比如to_spill对象已经过期,被着色了/被spill了 )
    pub fn spill(&mut self) -> ActionResult {
        // sill 直到没有tospill或者直到出现新的可color的节点
        // spill先从 spillcost较小的,邻居度较大的开始
        if self.info.as_ref().unwrap().to_simplify.is_empty() {
            return ActionResult::Finish;
        }
        // 试图拯救to_rescue中spill代价最大的节点
        // 如果spill后能够出现可以着色的节点,则算spill成功,先结束这次spill
        let item = self.info.as_mut().unwrap().to_spill.pop_min().unwrap();
        //判断是否已经被拯救,
        let reg = item.reg;
        if self.if_has_been_colored(&reg) || self.if_has_been_spilled(&reg) {
            return ActionResult::Fail;
        }
        //
        let tospill = self.choose_spill(&reg);
        if tospill != reg {
            // 如果要溢出的寄存器不等于选择的寄存器,需要把选择的寄存器再加入to_color中
            let item = self.draw_spill_div_nlc_item(&reg);
            self.info.as_mut().unwrap().to_color.push(item);
        }
        // 溢出操作一定成功
        if self.spill_one(tospill) {
            return ActionResult::Success;
        }
        panic!("gg");
        ActionResult::Fail
    }

    /// 在color_k_graph之前应该check k graph<br>
    ///  给剩余地悬点进行着色  (悬点并未进入spilling中,所以仍然获取到周围地颜色)
    pub fn color_k_graph(&mut self) -> ActionResult {
        // 对最后的k个节点进行着色
        loop {
            let k_graph = &mut self.info.as_mut().unwrap().k_graph;
            if k_graph.0.is_empty() {
                break;
            }
            let item = k_graph.0.pop_min().unwrap();
            let reg = item.reg;
            let available = self.draw_available_and_num_neigbhor_color(&reg);
        }

        ActionResult::Success
    }

    // 判断某个就节点是否是悬点
    #[inline]
    pub fn is_k_graph_node(&mut self, reg: &Reg) -> bool {
        self.get_available(reg).num_available_regs(reg.get_type())
            > self.get_num_of_live_neighbors(reg)
    }

    pub fn merge(&mut self) -> ActionResult {
        // 合并具有合并属性的寄存器,衡量合并收获,有选择地进行合并
        todo!()
    }

    #[inline]
    pub fn rescue(&mut self) -> ActionResult {
        // 从已经spill的寄存器旁边,根据spill cost删掉几个周围的寄存器,然后把脱离color的寄存器加入
        // 删除的操作可以局限与一轮,也可以局限于2轮
        // 在局部节点中判断是否能够产生优化操作
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

    ///着色某个寄存器
    ///
    #[inline]
    pub fn color_one(&mut self, reg: &Reg) -> bool {
        let color = self.choose_color(reg);
        if color.is_none() {
            return false;
        }
        let color = color.unwrap();
        self.color_one_with_certain_color(reg, color);
        true
    }

    #[inline]
    pub fn simpilfy_one(&mut self, reg: Reg) -> bool {
        if self.if_has_been_colored(&reg) || self.if_has_been_spilled(&reg) {
            //
            panic!("");
            return false;
        }

        //简化成功,该实例可以使用颜色,则化简成功,否则化简失败(但是化简失败也可能让别的spill能够恢复可着色状态)
        // 首先获取其nnc,从颜色最少的节点开始尝试,判断是否周围的节点能够与其他的地方交换颜色从而化简
        let nnc = self
            .info
            .as_mut()
            .unwrap()
            .nums_neighbor_color
            .get(&reg)
            .unwrap();
        // 对nnc进行堆排序找到一个可以开始的节点,并对节点进行尝试
        let mut order: Vec<i32> = Vec::with_capacity(32);
        // 获取颜色排序
        let sort = crate::backend::regalloc::utils::sort;
        sort(nnc, &mut order); //按照颜色在邻居节点出现数量数量从小到大升序排序
        let tmp_regusestat = RegUsedStat::init_for_reg(reg.get_type());
        // 判断是否能够化简成功,如果能够化简成功,返回交换队列以及产生的代价,以及是否能够成功 (如果化简失败回回退自己的化简操作)
        let try_simplify =
            |allocator: &mut Allocator, color: i32, reg: &Reg| -> (Vec<(Reg, Reg)>, f32, bool) {
                // 模拟simplify过程,如果模拟成功了,则进行spimlify
                // 遍历所有邻居,找到所有颜色为color的节点,然后判断是否它与附近的颜色有可以交换的
                // 如果可以,则进行交换,并记录在交换表中,
                // 一直交换下去直到交换完成,返回是否交换成功
                let mut num_live_neigbhors = allocator
                    .info
                    .as_ref()
                    .unwrap()
                    .all_live_neighbors
                    .get(reg)
                    .unwrap()
                    .len();
                let mut simpilfy_cost: f32 = 0.0;
                let mut swap_list: Vec<(Reg, Reg)> = Vec::new();
                while num_live_neigbhors > 0 {
                    num_live_neigbhors -= 1;
                    let neighbor = allocator
                        .info
                        .as_mut()
                        .unwrap()
                        .all_live_neighbors
                        .get_mut(reg)
                        .unwrap()
                        .pop_front()
                        .unwrap();
                    let neighbor_bitmap = allocator
                        .info
                        .as_ref()
                        .unwrap()
                        .all_live_neigbhors_bitmap
                        .get(reg)
                        .unwrap();
                    if !neighbor_bitmap.contains(neighbor.bit_code() as usize) {
                        continue;
                    }
                    allocator
                        .info
                        .as_mut()
                        .unwrap()
                        .all_live_neighbors
                        .get_mut(reg)
                        .unwrap()
                        .push_back(neighbor);
                    //
                    if !allocator.if_has_been_colored(&neighbor) {
                        continue;
                    }
                    // 判断是否和周围存在寄存器可以交换颜色
                    let mut neighbor_to_swap_to: Option<Reg> = None;
                    for ntst in allocator
                        .info
                        .as_ref()
                        .unwrap()
                        .all_live_neighbors
                        .get(&neighbor)
                        .unwrap()
                        .iter()
                    {
                        if !allocator.if_has_been_colored(ntst) {
                            continue;
                        }
                        neighbor_to_swap_to = Some(*ntst);
                        break;
                    }
                    if let Some(neighbor_to_swap_to) = neighbor_to_swap_to {
                        // 如果可以交换颜色,获取交换颜色造成的代价
                        simpilfy_cost += allocator.eval_swap(&neighbor, &neighbor_to_swap_to);
                        allocator.swap_color(neighbor, neighbor_to_swap_to);
                        swap_list.push((neighbor, neighbor_to_swap_to));
                    } else {
                        for (reg1, reg2) in swap_list.iter().rev() {
                            allocator.swap_color(*reg1, *reg2);
                        }
                        return (swap_list, simpilfy_cost, false);
                    }
                }
                (swap_list, simpilfy_cost, true)
            };
        // 指定预算下尝试化简,如果化简超过预算或者化简失败返回false (todo,替换try_simplify加速)
        // let try_simplify_with_budget =
        //     |allocator: &mut Allocator, color: i32, reg: &Reg, budget: i32| -> bool {
        //         todo!();
        //     };
        //回退化简操作
        let undo_simpilify = |allocator: &mut Allocator, swaplist: Vec<(Reg, Reg)>| {
            for (reg1, reg2) in swaplist.iter().rev() {
                allocator.swap_color(*reg1, *reg2);
            }
        };

        let spill_cost = *self.info.as_ref().unwrap().spill_cost.get(&reg).unwrap();
        // 暂时先尝试交换最少的两种颜色的交换
        for i in 0..2 {
            let color = *order.get(i).unwrap();
            // 判断这个颜色是否是合理的颜色
            if !tmp_regusestat.is_available_reg(color) {
                continue;
            }
            let (swap_list, simpilfy_cost, ok) = try_simplify(self, color, &reg);
            if !ok {
                continue;
            } else if simpilfy_cost > spill_cost {
                // 如果可以分配,但是分配代价高昂,回退
                undo_simpilify(self, swap_list);
                continue;
            } else {
                //TOCHECK,化简成功,而且代价合适,把当前的寄存器加回tocolor
                let item = self.draw_spill_div_nlc_item(&reg);
                self.info.as_mut().unwrap().to_color.push(item);
                return true;
            }
        }
        // // todo,尝试所有能够腾出的颜色
        // for color in order.iter() {
        //     if try_simplify(*color, &reg) {
        //         // 模拟成功,把当前节点作色
        //         self.color_one_with_certain_color(&reg, *color);
        //         return true;
        //     }
        // }
        false
    }

    /// 选择spill节点
    /// 在reg和reg邻居节点中选择一个最适合spill的节点
    ///
    #[inline]
    pub fn choose_spill(&self, reg: &Reg) -> Reg {
        //在该节点和该节点的周围节点中选择一个最适合spill的节点
        // 最适合spill的节点就是spill代价最小的节点
        // spill代价计算:  活邻居越多,spill代价越小,spill_cost越大,spill代价越大,
        // 能够救回的节点的代价越大,spiLl代价越小
        // val[reg]=reg.spill_cost/num_live_neighbor[reg] - sum(rescue.spill_cost/num_live_neighbor[reg])
        let val = |allocator: &Allocator, reg: &Reg| -> f32 {
            // 计算价值,首先,获取当前节点本身的spill cost(简单地使用spill cost来计算节省地内容)
            let mut out_val = self.get_spill_cost_div_lnn(reg);
            // 如果当前节点在colors里面,则spill cost还要减去消去它的颜色后能够救回的spill cost
            // 对该节点地邻居进行一次遍历(如果该节点有颜色的话)
            let color = self.get_color(reg);
            if color.is_none() {
                return out_val;
            }
            // TODO, 考虑边迹效应,遇到能够拯救多个节点的情况,调整下增加/减少权重的系数
            let color = *color.unwrap();
            for neighbor in self.info.as_ref().unwrap().all_neighbors.get(reg).unwrap() {
                if neighbor.is_physic()
                    || self.if_has_been_colored(neighbor)
                    || self.is_last_colored(neighbor)
                {
                    continue;
                }
                let nnc = self
                    .info
                    .as_ref()
                    .unwrap()
                    .nums_neighbor_color
                    .get(neighbor)
                    .unwrap();
                if *nnc.get(&color).unwrap() == 1 {
                    out_val -= self.get_spill_cost_div_lnn(neighbor);
                }
            }
            out_val
        };
        // 遍历节点reg和它周围节点
        let mut tospill = *reg;
        let info = self.info.as_ref().unwrap().to_owned();
        let all_live_neigbhors = &info.all_live_neighbors;
        let all_live_neigbors_bitmap = &info.all_live_neigbhors_bitmap;
        let mut tospill_val = val(self, reg);
        let bitmap = all_live_neigbors_bitmap.get(reg).unwrap();
        // 只在活着的节点(也就是没有被spill的节点中选择)
        //
        for neighbor in all_live_neigbhors.get(reg).unwrap() {
            let neigbor = *neighbor;
            if !bitmap.contains(neigbor.bit_code() as usize) {
                continue;
            }
            // 获取价值
            let tmp_tospill_val = val(self, &neigbor);
            if tmp_tospill_val < tospill_val {
                tospill = neigbor;
                tospill_val = tmp_tospill_val;
            }
        }
        tospill
    }

    #[inline]
    // 如果spill过程救活了一些节点,则返回true,否则返回false
    pub fn spill_one(&mut self, reg: Reg) -> bool {
        // spill reg本身或者周围的某个有色寄存器,选择一个结果好的,判断丢弃寄存器后是否产生新的好处
        // spill reg本身,
        if self.if_has_been_spilled(&reg) {
            panic!("u");
        }
        if self.if_has_been_colored(&reg) {
            let out = self.decolor_one(&reg);
            self.spill_one(reg);
            return out;
        }
        self.info.as_mut().unwrap().spillings.insert(reg.get_id());
        //从它的所有周围节点中去除该spill
        let mut num_live_neigbhors = self
            .info
            .as_ref()
            .unwrap()
            .all_live_neighbors
            .get(&reg)
            .unwrap()
            .len();
        while num_live_neigbhors > 0 {
            num_live_neigbhors -= 1;
            let live_neigbhors = self
                .info
                .as_mut()
                .unwrap()
                .all_live_neighbors
                .get_mut(&reg)
                .unwrap();
            let neighbor = live_neigbhors.pop_front().unwrap();
            if self
                .info
                .as_ref()
                .unwrap()
                .spillings
                .contains(&neighbor.get_id())
            {
                continue;
            }
            // 对于邻居非spilling的情况
            let info = &mut self.info.as_mut().unwrap();
            // 首先把节点放回live_neigbhors
            info.all_live_neighbors
                .get_mut(&reg)
                .unwrap()
                .push_back(neighbor);
            // 然后去除neighbor的 liveneigbhore标记中的reg
            info.all_live_neigbhors_bitmap
                .get_mut(&neighbor)
                .unwrap()
                .remove(reg.bit_code() as usize);
        }
        false
    }

    #[inline]
    pub fn despill_one(&mut self, reg: &Reg) {
        // 从spill中取东西回来要把东西加回live negibhores中
        // 需要修改live_neigbhors,用到allneighbors,spillings,
        if !self.if_has_been_spilled(reg) || self.if_has_been_colored(reg) {
            panic!("gg");
        }
        //刷新available和 nums_neighbor_color
        let (available, nnc) = self.draw_available_and_num_neigbhor_color(reg);
        self.info
            .as_mut()
            .unwrap()
            .availables
            .insert(*reg, available);
        self.info
            .as_mut()
            .unwrap()
            .nums_neighbor_color
            .insert(*reg, nnc);
        // 首先从spill移除
        self.info.as_mut().unwrap().spillings.remove(&reg.get_id());

        // 恢复该spill reg的 live_neigbhor和 live_neighbor_bitmap,
        // 并且刷新neighbor对该spill的感知
        let mut num_all_neigbhors = self
            .info
            .as_ref()
            .unwrap()
            .all_neighbors
            .get(reg)
            .unwrap()
            .len();
        let mut new_live_neighbors: LinkedList<Reg> = LinkedList::new();
        let mut new_live_bitmap = Bitmap::with_cap(num_all_neigbhors / 2 / 8 + 1);
        while num_all_neigbhors > 0 {
            num_all_neigbhors -= 1;
            let neighbors = self
                .info
                .as_mut()
                .unwrap()
                .all_neighbors
                .get_mut(reg)
                .unwrap();
            let neighbor = neighbors.pop_front().unwrap();
            neighbors.push_back(neighbor);
            if neighbor.is_physic() || self.is_last_colored(&neighbor) {
                continue;
            }
            if self
                .info
                .as_mut()
                .unwrap()
                .spillings
                .contains(&neighbor.get_id())
            {
                continue;
            }
            new_live_neighbors.push_back(neighbor);
            new_live_bitmap.insert(neighbor.bit_code() as usize);

            if let Some(nn_live_bitmap) = self
                .info
                .as_mut()
                .unwrap()
                .all_live_neigbhors_bitmap
                .get_mut(&neighbor)
            {
                if nn_live_bitmap.contains(reg.bit_code() as usize) {
                    continue;
                }
                nn_live_bitmap.insert(reg.bit_code() as usize);
                let nn_live_neighbors = self
                    .info
                    .as_mut()
                    .unwrap()
                    .all_live_neighbors
                    .get_mut(&neighbor)
                    .unwrap();
                nn_live_neighbors.push_back(*reg);
            } else {
                panic!("g");
            }
        }
        self.info
            .as_mut()
            .unwrap()
            .all_live_neigbhors_bitmap
            .insert(*reg, new_live_bitmap);
        self.info
            .as_mut()
            .unwrap()
            .all_live_neighbors
            .insert(*reg, new_live_neighbors);
    }

    // 给某个虚拟寄存器挑选可以用来作色的颜色
    #[inline]
    pub fn choose_color(&mut self, reg: &Reg) -> Option<i32> {
        // TOCHECK
        // TODO, improve,加入贪心,根据所在的指令类型，以及周围已经分配的颜色的情况选择颜色
        // 比如,获取周围的周围的颜色,按照它们的周围的颜色的数量进行排序
        // 找到color所在的地方
        let available = self.get_available(reg).get_rest_regs_for(reg.get_type());
        let mut colors_weights = HashMap::new();
        for color in available.iter() {
            colors_weights.insert(*color, 0);
        }
        // 遍历邻居节点的所有活节点
        let mut passed_regs = Bitmap::new();
        for neighbor in self
            .info
            .as_ref()
            .unwrap()
            .all_live_neighbors
            .get(reg)
            .unwrap()
        {
            for nn in self
                .info
                .as_ref()
                .unwrap()
                .all_live_neighbors
                .get(neighbor)
                .unwrap()
            {
                if !self.if_has_been_colored(nn) {
                    continue;
                }
                if passed_regs.contains(nn.bit_code() as usize) {
                    continue;
                }
                passed_regs.insert(nn.bit_code() as usize);
                let color = self.get_color(reg).unwrap();
                if !colors_weights.contains_key(&color) {
                    continue;
                }
                *colors_weights.get_mut(&color).unwrap() += 1;
            }
        }

        let sort = crate::backend::regalloc::utils::sort;
        let mut order = Vec::new();
        sort(&colors_weights, &mut order);
        match order.get(0) {
            None => None,
            Some(color) => Some(*color),
        }
    }

    #[inline]
    pub fn eval_swap(&mut self, reg1: &Reg, reg2: &Reg) -> f32 {
        //衡量交换的价值
        let color1 = *self.get_color(reg1).unwrap();
        let color2 = *self.get_color(reg2).unwrap();
        if color1 == color2 {
            panic!("理论上不处理相同颜色之间的swap操作");
            return 0.0;
        }
        let mut cost = 0.0; //记录能够造成的溢出/节省的溢出
                            // 集合所有能够从spillings中拯救的寄存器
        let mut regs = LinkedList::new();
        let mut map = Bitmap::new();

        for neighbor in self
            .info
            .as_ref()
            .unwrap()
            .all_neighbors
            .get(reg1)
            .unwrap()
            .iter()
        {
            if neighbor.is_physic() || self.is_last_colored(neighbor) {
                continue;
            }
            if map.contains(neighbor.bit_code() as usize) {
                continue;
            }
            map.insert(neighbor.bit_code() as usize);
            regs.push_back(*neighbor);
        }
        while !regs.is_empty() {
            let reg = regs.pop_front().unwrap();
            let live_bitmap = self
                .info
                .as_ref()
                .unwrap()
                .all_live_neigbhors_bitmap
                .get(&reg)
                .unwrap();
            let nnc = self
                .info
                .as_ref()
                .unwrap()
                .nums_neighbor_color
                .get(&reg)
                .unwrap();
            if live_bitmap.contains(reg1.bit_code() as usize)
                && live_bitmap.contains(reg2.bit_code() as usize)
            {
                continue;
            }
            let mut regusestat = *self.info.as_ref().unwrap().availables.get(&reg).unwrap();
            let mut tmp_d_cost = 0.0;
            if live_bitmap.contains(reg1.bit_code() as usize) {
                if nnc.get(&color1).unwrap_or(&0) == &1 {
                    tmp_d_cost -= self.get_spill_cost_div_lnn2(&reg);
                    regusestat.release_reg(color1);
                }
                if nnc.get(&color2).unwrap_or(&0) == &0 {
                    tmp_d_cost += self.get_spill_cost_div_lnn2(&reg);
                }
                regusestat.use_reg(color2);
            } else if live_bitmap.contains(reg2.bit_code() as usize) {
                if nnc.get(&color2).unwrap_or(&0) == &1 {
                    tmp_d_cost -= self.get_spill_cost_div_lnn2(&reg);
                    regusestat.release_freg(color2);
                }
                if nnc.get(&color1).unwrap_or(&0) == &0 {
                    tmp_d_cost += self.get_spill_cost_div_lnn2(&reg);
                }
                regusestat.use_reg(color1);
            } else {
                panic!("un reachable!");
            }
            if self.if_has_been_spilled(&reg) && regusestat.is_available(reg.get_type()) {
                // 拯救了一个寄存器
                cost -= self.get_spill_cost_div_lnn(&reg);
            } else if !self.if_has_been_spilled(&reg) && !regusestat.is_available(reg.get_type()) {
                // 抛弃了一个虚拟寄存器
                cost += self.get_spill_cost_div_lnn(&reg);
            } else {
                // 否则就是
                cost += tmp_d_cost;
            }
        }
        // 遍历reg2的寄存器
        cost
    }

    #[inline]
    pub fn swap_color(&mut self, reg1: Reg, reg2: Reg) {
        let info = self.info.as_ref().unwrap();
        let color1 = *info.colors.get(&reg1.get_id()).unwrap();
        let color2 = *info.colors.get(&reg1.get_id()).unwrap();
        self.decolor_one(&reg1);
        self.decolor_one(&reg2);
        self.color_one_with_certain_color(&reg1, color2);
        self.color_one_with_certain_color(&reg2, color1);
    }

    // 移除某个节点的颜色
    #[inline]
    pub fn decolor_one(&mut self, reg: &Reg) -> bool {
        if self.if_has_been_spilled(reg) || !self.if_has_been_colored(reg) {
            panic!("unreachable!");
        }
        // 移除着色并且取出颜色
        let color = self
            .info
            .as_mut()
            .unwrap()
            .colors
            .remove(&reg.get_id())
            .unwrap();
        let mut out = false;
        let mut to_despill = LinkedList::new(); //暂存decolor过程中发现的能够拯救回来的寄存器
                                                // todo
        let mut num_all_neighbors = self
            .info
            .as_ref()
            .unwrap()
            .all_neighbors
            .get(reg)
            .unwrap()
            .len();

        while num_all_neighbors > 0 {
            num_all_neighbors -= 1;
            let neighbors = self
                .info
                .as_mut()
                .unwrap()
                .all_neighbors
                .get_mut(reg)
                .unwrap();
            if neighbors.is_empty() {
                break;
            }
            let neighbor = neighbors.pop_front().unwrap();
            neighbors.push_back(neighbor);
            if neighbor.is_physic() || self.is_last_colored(&neighbor) {
                continue;
            }
            let nums_neighbor_color = self
                .info
                .as_mut()
                .unwrap()
                .nums_neighbor_color
                .get_mut(&neighbor)
                .unwrap();
            let new_num = nums_neighbor_color.get(&color).unwrap_or(&0) - 1;
            nums_neighbor_color.insert(color, new_num);
            if new_num == 0 {
                // self.in
                self.get_mut_available(&neighbor).release_reg(color);
                if self.if_has_been_spilled(&neighbor) {
                    out = true;
                    to_despill.push_back(neighbor);
                }
            } else if new_num < 0 {
                panic!("gg");
            }
        }
        while !to_despill.is_empty() {
            let to_despill_one = to_despill.pop_front().unwrap();
            self.despill_one(&to_despill_one);
        }
        out
    }

    /// 给某个虚拟寄存器使用某种特定颜色进行着色
    /// 如果着色成功,
    #[inline]
    pub fn color_one_with_certain_color(&mut self, reg: &Reg, color: i32) {
        if self.if_has_been_colored(reg) || self.if_has_been_colored(reg) {
            panic!("un reachable");
        }
        let info = self.info.as_mut().unwrap();
        if !info.availables.get(reg).unwrap().is_available_reg(color) {
            panic!("g");
        }
        info.colors.insert(reg.get_id(), color);
        let mut num = info.all_live_neighbors.get(reg).unwrap().len();
        while num > 0 {
            num -= 1;
            let live_neighbors = info.all_live_neighbors.get_mut(reg).unwrap();
            let neighbor = live_neighbors.pop_front().unwrap();
            let live_neigbhors_bitmap = info.all_live_neigbhors_bitmap.get(reg).unwrap();
            if !live_neigbhors_bitmap.contains(neighbor.bit_code() as usize) {
                continue;
            }
            live_neighbors.push_back(neighbor);
            info.availables.get_mut(&neighbor).unwrap().use_reg(color);
            let nums_neighbor_color = info.nums_neighbor_color.get_mut(&neighbor).unwrap();
            nums_neighbor_color.insert(color, nums_neighbor_color.get(&color).unwrap_or(&0) + 1);
        }
    }

    ///获取寄存器的一些属性
    /// * 周围已有的各色物理寄存器数量
    /// * 自身剩余可着色空间
    /// * 自身是否已经着色
    /// * 自身是否已经spill
    #[inline]
    pub fn get_spill_cost_div_lnn2(&self, reg: &Reg) -> f32 {
        let spill_cost = self.info.as_ref().unwrap().spill_cost.get(reg).unwrap();
        let nn = self
            .info
            .as_ref()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get(reg)
            .unwrap();
        spill_cost / (nn.len() * nn.len()) as f32
    }
    #[inline]
    pub fn get_spill_cost_div_lnn(&self, reg: &Reg) -> f32 {
        let spill_cost = self.info.as_ref().unwrap().spill_cost.get(reg).unwrap();
        let nn = self
            .info
            .as_ref()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get(reg)
            .unwrap();
        spill_cost / nn.len() as f32
    }

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

    ///判断是否已经加入到k-graph
    #[inline]
    pub fn if_has_been_added_to_k_graph(&self, reg: &Reg) -> bool {
        self.info
            .as_ref()
            .unwrap()
            .k_graph
            .1
            .contains(reg.bit_code() as usize)
    }

    #[inline]
    pub fn if_swapable_for_color(&self, reg1: &Reg, reg2: &Reg) -> bool {
        // 判断两个寄存器的颜色是否能够发生交换
        if !self.if_has_been_colored(reg1) || !self.if_has_been_colored(reg2) {
            return false;
        }
        // 判断
        let color1 = *self.get_color(reg1).unwrap();
        let color2 = *self.get_color(reg2).unwrap();
        let nncs = &self.info.as_ref().unwrap().nums_neighbor_color;
        let color2_times_around_reg1 = nncs.get(reg1).unwrap().get(&color2).unwrap_or(&0);
        let color1_times_arount_reg2 = nncs.get(reg2).unwrap().get(&color1).unwrap_or(&0);
        if self
            .info
            .as_ref()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get(reg1)
            .unwrap()
            .contains(reg2.bit_code() as usize)
        {
            if *color2_times_around_reg1 == 1 && *color1_times_arount_reg2 == 1 {
                return true;
            }
            return false;
        }
        if color1_times_arount_reg2 == &0 || color2_times_around_reg1 == &0 {
            return true;
        }
        false
    }
    #[inline]
    pub fn is_last_colored(&self, reg: &Reg) -> bool {
        self.info.as_ref().unwrap().last_colors.contains(reg)
    }

    #[inline]
    // 根据总冲突图刷新并返回regusestat和num neighbor color
    pub fn draw_available_and_num_neigbhor_color(
        &self,
        reg: &Reg,
    ) -> (RegUsedStat, HashMap<i32, i32>) {
        let mut available = RegUsedStat::new();
        let mut nnc = HashMap::with_capacity(32);
        // todo!();
        // 遍历all_neigbhor得到available和nnc
        let info = self.info.as_ref().unwrap();
        for neighbor in info.all_neighbors.get(reg).unwrap() {
            if neighbor.is_physic() || self.is_last_colored(neighbor) {
                continue;
            }
            if info.spillings.contains(&neighbor.get_id()) {
                continue;
            }
            let color = *info.colors.get(&neighbor.get_id()).unwrap();
            available.use_reg(color);
            let new_num = nnc.get(&color).unwrap_or(&0) + 1;
            nnc.insert(color, new_num);
        }
        (available, nnc)
    }

    ///绘制item, 绘制(reg,spill_cost/num_live_neigbhor) item
    #[inline]
    pub fn draw_spill_div_nlc_item(&self, reg: &Reg) -> OperItem {
        let spill_cost = self.info.as_ref().unwrap().spill_cost.get(reg).unwrap();
        let nlc = self
            .info
            .as_ref()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get(reg)
            .unwrap()
            .len();
        OperItem {
            reg: *reg,
            cost: *spill_cost / (nlc as f32 + 1.0),
        }
    }

    #[inline]
    pub fn get_color(&self, reg: &Reg) -> Option<&i32> {
        if reg.is_physic() {
            panic!("gg");
        }
        self.info.as_ref().unwrap().colors.get(&reg.get_id())
    }

    #[inline]
    fn get_available(&self, reg: &Reg) -> RegUsedStat {
        *self.info.as_ref().unwrap().availables.get(reg).unwrap()
    }
    #[inline]
    fn get_mut_available(&mut self, reg: &Reg) -> &mut RegUsedStat {
        self.info.as_mut().unwrap().availables.get_mut(reg).unwrap()
    }

    #[inline]
    fn get_num_of_live_neighbors(&self, reg: &Reg) -> usize {
        self.info
            .as_ref()
            .unwrap()
            .all_live_neighbors
            .get(reg)
            .unwrap()
            .len()
    }

    // 获取的可用颜色以及周围的活邻居数量
    fn get_num_available_and_num_live_neighbor(&self, reg: &Reg) -> (i32, i32) {
        let info = self.info.as_ref().unwrap();
        let na = info
            .availables
            .get(reg)
            .unwrap()
            .num_available_regs(reg.get_type());
        let nn = self
            .info
            .as_ref()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get(reg)
            .unwrap()
            .len();
        (na as i32, nn as i32)
    }
}
