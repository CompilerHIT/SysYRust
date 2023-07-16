// 优化的图着色寄存器分配
// 待实现优化
/*
1.基于贪心的color选择和spill选择
2.合并寄存器
 */

// 或者可以认为是没有启发的线性扫描寄存器分配

use crate::{
    backend::{
        operand::Reg,
        regalloc::{easy_gc_alloc, structs::FuncAllocStat},
    },
    container::bitmap::Bitmap,
    log_file, log_file_uln,
};
use core::panic;
use std::{
    collections::{HashMap, HashSet, LinkedList, VecDeque},
    fs,
};

use super::regalloc::{self, Regalloc};

pub struct Allocator {
    easy_gc_allocator: easy_gc_alloc::Allocator,

    k_graph: (LinkedList<Reg>, Bitmap), //用来实现弦图优化
    interference_graph_lst: HashMap<Reg, LinkedList<Reg>>, //遍历节点的冲突用
    tosave_regs: LinkedList<Reg>,       //保存等待拯救的寄存器,或者说save寄存器
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            easy_gc_allocator: easy_gc_alloc::Allocator::new(),
            k_graph: (LinkedList::new(), Bitmap::new()),
            tosave_regs: LinkedList::new(),
            interference_graph_lst: HashMap::new(),
        }
    }

    // 根据当前已分配情况优化地计算分配代价
    pub fn opt_count_spill_cost(&self) {}

    // 更新弦图,从待作色寄存器中取出悬点
    pub fn refresh_k_graph(&mut self) {
        let mut new_regs: LinkedList<Reg> = LinkedList::new();
        while !self.easy_gc_allocator.regs.is_empty() {
            let reg = self.easy_gc_allocator.regs.pop_front().unwrap();
            // 判断这个点是否是悬点,如果是,放到悬图中
            if self.is_k_graph_node(&reg) {
                let k_graph = &mut self.k_graph;
                k_graph.0.push_back(reg);
                k_graph.1.insert(reg.bit_code() as usize);
            } else {
                new_regs.push_back(reg);
            }
        }
        self.easy_gc_allocator.regs = new_regs;
    }

    pub fn is_k_graph_node(&mut self, reg: &Reg) -> bool {
        let tmp_set = HashSet::new();
        self.easy_gc_allocator
            .interference_graph
            .get(&reg)
            .unwrap_or(&tmp_set)
            .len()
            < self
                .easy_gc_allocator
                .availables
                .get(&reg)
                .unwrap()
                .num_available_regs(reg.get_type())
    }

    // 最终着色剩余悬点
    pub fn color_k_graph(&mut self) {
        // 对于悬点的作色可以任意着色,如果着色完成
        while !self.k_graph.0.is_empty() {
            let reg = self.k_graph.0.pop_front().unwrap();
            if !self.is_k_graph_node(&reg) {}
            let available = self.easy_gc_allocator.availables.get(&reg).unwrap();
            let color = available.get_available_reg(reg.get_type()).unwrap();
            self.k_graph.1.remove(reg.bit_code() as usize);
            self.easy_gc_allocator
                .color_one_with_certain_color(reg, color);
        }
    }

    // 试图拯救spilling的寄存器,如果拯救成功任何一个,返回true,如果一个都拯救不了返回false
    pub fn try_save(&mut self) -> bool {
        false
    }

    // TODO,选择最小度节点color
    pub fn opt_color(&mut self) -> bool {
        // 开始着色,着色直到无法再找到可着色的点为止,如果全部点都可以着色,返回true,否则返回false
        // 如果一个点是spill或者是悬点或者是dstr,则取出待着色列表
        let mut new_to_colors = LinkedList::new();
        let mut out = true;
        while !self.easy_gc_allocator.regs.is_empty() {
            let old_to_colors = &mut self.easy_gc_allocator.regs;
            let reg = old_to_colors.pop_front().unwrap();
            if reg.is_physic() || self.easy_gc_allocator.colors.contains_key(&reg.get_id()) {
                continue;
            }
            // 把它加入tosave
            if self.easy_gc_allocator.spillings.contains(&reg.get_id()) {
                self.tosave_regs.push_back(reg);
                continue;
            }
            // 首先试图color,color失败加入到new_to_colors中
            if !self.easy_gc_allocator.color_one(reg) {
                new_to_colors.push_back(reg);
            } else {
                out = false
            }
        }
        self.easy_gc_allocator.regs = new_to_colors;
        out
    }

    // 选择一个价值最大节点spill
    pub fn opt_spill_one(&mut self, reg: Reg) {}

    // 改变节点颜色
    pub fn opt_simplify_one(&mut self) {}
}
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> FuncAllocStat {
        self.easy_gc_allocator.build_interference_graph(func);

        let intereref_path = "interference_graph.txt";
        self.easy_gc_allocator
            .interference_graph
            .iter()
            .for_each(|(reg, neighbors)| {
                log_file_uln!(intereref_path, "node {reg}\n{{");
                neighbors
                    .iter()
                    .for_each(|neighbor| log_file_uln!(intereref_path, "({},{})", reg, neighbor));
                log_file!(intereref_path, "}}\n");
            });

        self.easy_gc_allocator.count_spill_costs(func);
        // self.refresh_k_graph();

        // TODO,加入化简后单步合并检查 以及 分配完成后合并检查
        while !self.easy_gc_allocator.color() {
            if self.easy_gc_allocator.simplify() {
                continue;
            }
            self.easy_gc_allocator.spill();
        }

        // self.color_k_graph();

        let (spillings, dstr) = self.easy_gc_allocator.alloc_register();
        let (func_stack_size, bb_sizes) = regalloc::countStackSize(func, &spillings);
        let mut out = FuncAllocStat {
            stack_size: func_stack_size,
            bb_stack_sizes: bb_sizes,
            spillings: spillings,
            dstr: dstr,
        };

        // 检查下寄存器合并前的分配结果
        let path = "befor_merge_opt.txt";
        log_file!(
            path,
            "func:{}\ndstr:{:?}\n\nspillings:{:?}",
            func.label,
            out.dstr,
            out.spillings
        );

        // 寄存器合并
        let max_times = 3;
        for i in 0..max_times {
            let ok = regalloc::merge_alloc(
                func,
                &mut out.dstr,
                &mut out.spillings,
                &mut self.easy_gc_allocator.nums_neighbor_color,
                &mut self.easy_gc_allocator.availables,
                &self.easy_gc_allocator.costs_reg,
                &mut self.easy_gc_allocator.interference_graph,
            );
            if !ok {
                log_file!(
                    "merge_times.txt",
                    "func:{},merge times:{}",
                    func.label,
                    i + 1
                );
                break;
            }
            if i == max_times - 1 {
                log_file!(
                    "merge_times.txt",
                    "unend!func:{},merge times:{}",
                    func.label,
                    i + 1
                );
            }
        }
        // regalloc::merge_alloc(func, &mut out.dstr, &mut out.spillings,
        //     & self.easy_gc_allocator.ends_index_bb, &mut self.easy_gc_allocator.nums_neighbor_color, &mut self.easy_gc_allocator.availables, &mut self.easy_gc_allocator.interference_graph);

        out
    }
}
