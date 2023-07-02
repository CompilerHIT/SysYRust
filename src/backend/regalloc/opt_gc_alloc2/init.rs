use super::*;

impl Allocator {
    pub fn new() -> Allocator {
        Allocator { info: None }
    }

    pub fn init(&mut self, func: &Func) {
        let num_estimate_regs = func.num_regs();
        let ends_index_bb = regalloc::ends_index_bb(func);
        let allneighbors = regalloc::build_interference_into_lst(func, &ends_index_bb);
        let nums_neighbor_color = regalloc::build_nums_neighbor_color(func, &ends_index_bb);
        let availables = regalloc::build_availables(func, &ends_index_bb);
        let spill_cost = regalloc::estimate_spill_cost(func);
        let mut all_live_neigbhors: HashMap<Reg, LinkedList<Reg>> = HashMap::new();
        let mut all_live_neighbors_bitmap: HashMap<Reg, Bitmap> = HashMap::new();
        let mut last_colors: HashSet<Reg> = HashSet::new();
        let mut to_color: BiHeap<OperItem> = BiHeap::new();

        // 初始化last_colors  (last colors不包括物理寄存器,不包括不一定能够着色的寄存器)
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

        // 初始化to colors和live neighbors
        for (reg, neighbors) in &allneighbors {
            if reg.is_physic() || last_colors.contains(reg) {
                continue;
            }
            let mut live_neighbors = LinkedList::new();
            let mut live_neigbhors_bitmap = Bitmap::with_cap(10);
            for neighbor in neighbors {
                if neighbor.is_physic() || last_colors.contains(neighbor) {
                    continue;
                }
                live_neighbors.push_back(*neighbor);
                live_neigbhors_bitmap.insert(neighbor.bit_code() as usize);
            }
            to_color.push(OperItem::new(
                reg,
                &(*spill_cost.get(reg).unwrap() / (live_neighbors.len() as f32)),
            ));
            all_live_neigbhors.insert(*reg, live_neighbors);
            all_live_neighbors_bitmap.insert(*reg, live_neigbhors_bitmap);
        }

        let info = AllocatorInfo {
            to_color: to_color,
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
    //
}
