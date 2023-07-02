use super::*;

impl Allocator {
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
                self.push_to_tosimpilfy(&reg);
            }
            break;
        }
        out
    }

    // 把一个寄存器加入tocolor
    pub fn push_to_tocolor(&mut self, reg: &Reg) {
        let item = self.draw_na_div_nln_item(reg);
        self.info.as_mut().unwrap().to_color.push(item);
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

    // 给某个虚拟寄存器挑选可以用来作色的颜色
    #[inline]
    pub fn choose_color(&mut self, reg: &Reg) -> Option<i32> {
        //TOCHECK
        // return match self.get_available(reg).get_available_reg(reg.get_type()) {
        //     None => None,
        //     Some(color) => Some(color),
        //     _ => panic!("gg"),
        // };
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
                let color = self.get_color(nn).unwrap();
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
        let mut num_all_neighbors = self.get_all_neighbors(reg).len();
        while num_all_neighbors > 0 {
            num_all_neighbors -= 1;
            let neighbors = self.get_mut_all_neighbors(reg);
            let neighbor = neighbors.pop_front().unwrap();
            neighbors.push_back(neighbor);
            if neighbor.is_physic() || self.is_last_colored(&neighbor) {
                continue;
            }
            let nums_neighbor_color = self.get_mut_num_neighbor_color(&neighbor);
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
            let neighbor = self.get_mut_live_neighbors(reg).pop_front().unwrap();
            if self
                .get_live_neighbors_bitmap(reg)
                .contains(neighbor.bit_code() as usize)
            {
                continue;
            }

            self.get_mut_live_neighbors(&neighbor).push_back(neighbor);
            self.get_mut_available(&neighbor).use_reg(color);
            let nums_neighbor_color = self.get_mut_num_neighbor_color(&neighbor);
            nums_neighbor_color.insert(color, nums_neighbor_color.get(&color).unwrap_or(&0) + 1);
            // 判断这个寄存器是否能够被加入到to simpilfy
            if !self
                .get_available(&neighbor)
                .is_available(neighbor.get_type())
            {
                // 如果这个寄存器失效了,把它加入待spill列表中

                self.push_to_tosimpilfy(&neighbor);
            }
            // 判断这个虚拟寄存器是否已经存在
            // tocheck("判断是否要从 k_graph中移除");
            if self.is_k_graph_node(&neighbor) {
                let num_available = self
                    .get_available(&neighbor)
                    .num_available_regs(neighbor.get_type());
                let num_live_neigbhors = self.get_live_neighbors_bitmap(&neighbor).len();
                if num_available >= num_live_neigbhors {
                    self.remove_from_k_graph(&neighbor);
                }
            }
        }
    }
}
