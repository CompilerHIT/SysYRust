use super::*;

impl Allocator {
    pub fn new() -> Allocator {
        Allocator { info: None }
    }

    pub fn init(&mut self, func: &Func) {
        let num_estimate_regs = func.num_regs();
        let ends_index_bb = regalloc::build_ends_index_bb(func);
        let all_neighbors = regalloc::build_interference_into_lst(func, &ends_index_bb);
        let nums_neighbor_color = regalloc::build_nums_neighbor_color(func, &ends_index_bb);
        let availables = regalloc::build_availables(func, &ends_index_bb);
        let spill_cost = regalloc::estimate_spill_cost(func);
        let mut to_color: BiHeap<OperItem> = BiHeap::new();

        let (last_colors, last_colors_lst, all_live_neighbors_bitmap, all_live_neigbhors) =
            Allocator::build_live_graph_and_last_colors(&all_neighbors, &availables);
        //
        // todo!("分离悬点集合构建的逻辑,完成初始化纯净干涉图操作");

        // 初始化last_colors  (last colors不包括物理寄存器,不包括不一定能够着色的寄存器)
        // for (reg, neighbors) in &all_neighbors {
        //     // 判断它是否是一个last colors,如果virtual neighbors< availables
        //     if reg.is_physic() {
        //         continue;
        //     }
        //     let num = availables
        //         .get(reg)
        //         .unwrap()
        //         .num_available_regs(reg.get_type());
        //     let mut num_v_neighbors = 0;
        //     for neighbor in neighbors {
        //         if neighbor.is_physic() {
        //             continue;
        //         }
        //         num_v_neighbors += 1;
        //     }
        //     if num_v_neighbors < num {
        //         last_colors.insert(*reg);
        //     }
        // }

        // TOCHECK,根据live_graph初始化to_color
        for (reg, live_neighbors) in &all_live_neigbhors {
            to_color.push(OperItem::new(
                reg,
                &(*spill_cost.get(reg).unwrap() / (live_neighbors.len() as f32)),
            ));
        }
        //
        // for (reg, neighbors) in &all_neighbors {
        //     if reg.is_physic() || last_colors.contains(reg) {
        //         continue;
        //     }
        //     let mut live_neighbors = LinkedList::new();
        //     let mut live_neigbhors_bitmap = Bitmap::with_cap(neighbors.len() / 8);
        //     for neighbor in neighbors {
        //         if neighbor.is_physic() || last_colors.contains(neighbor) {
        //             continue;
        //         }
        //         live_neighbors.push_back(*neighbor);
        //         live_neigbhors_bitmap.insert(neighbor.bit_code() as usize);
        //     }
        //     to_color.push(OperItem::new(
        //         reg,
        //         &(*spill_cost.get(reg).unwrap() / (live_neighbors.len() as f32)),
        //     ));
        //     all_live_neigbhors.insert(*reg, live_neighbors);
        //     all_live_neighbors_bitmap.insert(*reg, live_neigbhors_bitmap);
        // }

        let info = AllocatorInfo {
            to_color: to_color,
            to_simplify: BiHeap::new(),
            to_spill: BiHeap::new(),
            colored: BiHeap::new(),
            k_graph: (BiHeap::new(), Bitmap::with_cap(num_estimate_regs / 8 + 1)),
            spill_cost: spill_cost,
            all_neighbors,
            nums_neighbor_color: nums_neighbor_color,
            availables: availables,
            colors: HashMap::new(),
            spillings: HashSet::new(),
            all_live_neighbors: all_live_neigbhors,
            last_colors: last_colors,
            last_colors_lst: last_colors_lst,
            all_live_neigbhors_bitmap: all_live_neighbors_bitmap,
        };
        self.info = Some(info);
    }

    /// build last colors 和livegraph,
    /// 传入的参数为all_neighbors,返回的参数为last_colos_set,last_colors_lst,live_graph,live_graph_bitmap
    fn build_live_graph_and_last_colors(
        all_neighbors: &HashMap<Reg, LinkedList<Reg>>,
        availables: &HashMap<Reg, RegUsedStat>,
    ) -> (
        Bitmap,
        LinkedList<Reg>,
        HashMap<Reg, Bitmap>,
        HashMap<Reg, LinkedList<Reg>>,
    ) {
        //
        // let mut last_colors
        let mut last_colors_bitmap = Bitmap::with_cap(all_neighbors.len() / 16);
        let mut last_colors_lst = LinkedList::new();
        let mut all_live_neighbors = HashMap::new();
        let mut all_live_neighbors_bitmap = HashMap::new();
        let mut num_availables = HashMap::new();
        // 对所有寄存器建立na表
        for (reg, reg_use_stat) in availables {
            num_availables.insert(*reg, reg_use_stat.num_available_regs(reg.get_type()));
        }

        // 第一次初始化last colors
        for (reg, neighbors) in all_neighbors {
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
                last_colors_bitmap.insert(reg.bit_code() as usize);
                last_colors_lst.push_back(*reg);
            }
        }

        // 第一次初始化live_graph
        for (reg, neighbors) in all_neighbors {
            if reg.is_physic() || last_colors_bitmap.contains(reg.bit_code() as usize) {
                continue;
            }
            let mut live_neighbors = LinkedList::new();
            let mut live_neigbhors_bitmap = Bitmap::with_cap(neighbors.len() / 8);
            for neighbor in neighbors {
                if neighbor.is_physic() || last_colors_bitmap.contains(neighbor.bit_code() as usize)
                {
                    continue;
                }
                live_neighbors.push_back(*neighbor);
                live_neigbhors_bitmap.insert(neighbor.bit_code() as usize);
            }
            all_live_neighbors.insert(*reg, live_neighbors);
            all_live_neighbors_bitmap.insert(*reg, live_neigbhors_bitmap);
        }

        // 循环地对live_graph进行处理,不断取出其中的悬点直到无悬点可取为止
        loop {
            let mut ifFinish = true;

            // 取出悬点加入last colors
            for (reg, live_neighbors) in &all_live_neighbors {
                let na = *num_availables.get(reg).unwrap();
                let nln = live_neighbors.len();
                if nln < na {
                    last_colors_bitmap.insert(reg.bit_code() as usize);
                    last_colors_lst.push_back(*reg);
                    ifFinish = false;
                }
            }
            // live_graph中移除悬点
            let mut to_remove_keys = Vec::new();
            for (reg, live_neighbors) in &mut all_live_neighbors {
                if last_colors_bitmap.contains(reg.bit_code() as usize) {
                    to_remove_keys.push(*reg);
                    continue;
                }
                let mut num = live_neighbors.len();
                while num > 0 {
                    num -= 1;
                    let neighbor = live_neighbors.pop_front().unwrap();
                    if last_colors_bitmap.contains(neighbor.bit_code() as usize) {
                        // 移除点
                        continue;
                    }
                    live_neighbors.push_back(neighbor);
                }
            }
            for reg in &to_remove_keys {
                all_live_neighbors.remove(reg);
            }
            if ifFinish {
                break;
            }
        }
        // 最后更新一下,last neighbors bitmap
        for (reg, live_neighbors) in &all_live_neighbors {
            let mut bitmap = Bitmap::new();
            for neighbor in live_neighbors {
                bitmap.insert(neighbor.bit_code() as usize);
            }
            all_live_neighbors_bitmap.insert(*reg, bitmap);
        }
        (
            last_colors_bitmap,
            last_colors_lst,
            all_live_neighbors_bitmap,
            all_live_neighbors,
        )
    }
}
