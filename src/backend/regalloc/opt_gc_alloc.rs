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
    log_file,
};
use std::{
    collections::{HashMap, HashSet, LinkedList},
    fs,
};

use super::regalloc::{self, Regalloc};

pub struct Allocator {
    easy_gc_allocator: easy_gc_alloc::Allocator,
    k_graph: (LinkedList<Reg>, Bitmap), //用来实现弦图优化
                                        // real_interference_graph: HashSet<Reg, HashSet<Reg>>, //真冲突图,
}

impl Allocator {
    pub fn new() -> Allocator {
        Allocator {
            easy_gc_allocator: easy_gc_alloc::Allocator::new(),
            k_graph: (LinkedList::new(), Bitmap::new()),
        }
    }

    // 根据当前已分配情况优化地计算分配代价
    pub fn opt_count_spill_cost(&self) {}

    // 更新弦图,从待作色寄存器中取出悬点
    pub fn refresh_k_graph(&mut self) {
        // TODO
        let old_regs = &mut self.easy_gc_allocator.regs;
        let new_regs: LinkedList<Reg> = LinkedList::new();

        let k_graph = &mut self.k_graph;
        while !old_regs.is_empty() {
            // 判断这个点是否是悬点,如果是,放到悬图中
        }
    }

    // 最终着色剩余悬点
    pub fn color_k_graph(&mut self) {}

    pub fn opt_color(&mut self) {
        // // 开始着色,着色直到无法再找到可着色的点为止,如果全部点都可以着色,返回true,否则返回false
        // // 如果一个点是spill或者是悬点或者是dstr,则取出待着色列表
        // let mut out = true;
        // while !self.regs.is_empty() {
        //     let reg = *self.regs.front().unwrap();
        //     if !reg.is_virtual()
        //         || self.spillings.contains(&reg.get_id())
        //         || self.colors.contains_key(&reg.get_id())
        //     {
        //         self.regs.pop_front();
        //         continue;
        //     }
        //     let ok = self.color_one(reg);
        //     if ok {
        //         self.regs.pop_front();
        //     } else {
        //         out = false;
        //         Allocator::debug("interef", [&reg].into_iter().collect());
        //         self.interference_regs.insert(reg);
        //         break;
        //     }
        // }

        // out
    }

    pub fn opt_spill_one(&mut self) {}

    pub fn opt_simplify_one(&mut self) {}
}
impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::instrs::Func) -> FuncAllocStat {
        self.easy_gc_allocator.build_interference_graph(func);

        self.easy_gc_allocator.count_spill_costs(func);

        // TODO,加入化简后单步合并检查 以及 分配完成后合并检查
        while !self.easy_gc_allocator.color() {
            if self.easy_gc_allocator.simplify() {
                continue;
            }
            self.easy_gc_allocator.spill();
        }

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
        let max_times = 4;
        for i in 0..max_times {
            let ok = regalloc::merge_alloc(
                func,
                &mut out.dstr,
                &mut out.spillings,
                &self.easy_gc_allocator.ends_index_bb,
                &mut self.easy_gc_allocator.nums_neighbor_color,
                &mut self.easy_gc_allocator.availables,
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
