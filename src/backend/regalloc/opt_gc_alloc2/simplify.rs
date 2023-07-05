use crate::log;

use super::*;
impl Allocator {
    #[inline]
    pub fn push_to_tosimpilfy(&mut self, reg: &Reg) {
        self.dump_action("tosimplify", reg);
        // 把一个节点加入待着色列表中
        let item = self.draw_spill_div_nln_item(reg);
        self.info.as_mut().unwrap().to_simplify.push(item);
    }

    #[inline]
    pub fn simpilfy(&mut self) -> ActionResult {
        // 此处的simplify是简化color中color到的颜色
        // simpilfy,选择spill cost最大的一个
        if self.info.as_ref().unwrap().to_simplify.is_empty() {
            return ActionResult::Finish;
        }
        // 试图拯救to_rescue中spill代价最大的节点
        // 试图simplify来拯救当前节点
        let item = self.info.as_mut().unwrap().to_simplify.pop_max().unwrap();
        // // TODO ,如果化简成功
        if self.simpilfy_one(item.reg) {
            return ActionResult::Success;
        }
        return ActionResult::Fail;
    }
    #[inline]
    pub fn simpilfy_one(&mut self, reg: Reg) -> bool {
        self.dump_action("simplify", &reg);

        if self.if_has_been_colored(&reg) || self.if_has_been_spilled(&reg) {
            panic!("");
            return false;
        }

        //简化成功,该实例可以使用颜色,则化简成功,否则化简失败(但是化简失败也可能让别的spill能够恢复可着色状态)
        // 首先获取其nnc,从颜色最少的节点开始尝试,判断是否周围的节点能够与其他的地方交换颜色从而化简
        let nnc = self.get_num_neighbor_color(&reg);
        // 对nnc进行堆排序找到一个可以开始的节点,并对节点进行尝试
        let mut order: Vec<i32> = Vec::with_capacity(32);
        // 获取颜色排序
        let sort = crate::backend::regalloc::utils::sort;
        sort(nnc, &mut order); //按照颜色在邻居节点出现数量数量从小到大升序排序
        let tmp_regusestat = RegUsedStat::init_for_reg(reg.get_type());
        // 判断是否能够化简成功,如果能够化简成功,返回交换队列以及产生的代价,以及是否能够成功 (如果化简失败回回退自己的化简操作)
        let try_simplify = |allocator: &mut Allocator,
                            color: i32,
                            times_to_remove: i32,
                            reg: &Reg|
         -> (Vec<(Reg, Reg)>, f32, bool) {
            // 模拟simplify过程,如果模拟成功了,则进行spimlify
            // 遍历所有邻居,找到所有颜色为color的节点,然后判断是否它与附近的颜色有可以交换的
            // 如果可以,则进行交换,并记录在交换表中,
            // 一直交换下去直到交换完成,返回是否交换成功
            let mut num_live_neigbhors = allocator.get_live_neighbors(reg).len();
            let mut simpilfy_cost: f32 = 0.0;
            let mut remove_num_neibor_this_color = 0; //移除的周围的该颜色的数量
            let mut swap_list: Vec<(Reg, Reg)> = Vec::new();

            log!(
                "num_live_neighbors:{},{:?}\n\n",
                num_live_neigbhors,
                allocator.get_live_neighbors(reg)
            );

            while num_live_neigbhors > 0 {
                num_live_neigbhors -= 1;
                let neighbor = allocator.get_mut_live_neighbors(reg).pop_front().unwrap();
                if !allocator
                    .get_live_neighbors_bitmap(reg)
                    .contains(neighbor.bit_code() as usize)
                {
                    continue;
                }
                allocator.get_mut_live_neighbors(reg).push_back(neighbor);

                //
                if !allocator.if_has_been_colored(&neighbor) {
                    continue;
                }
                let neighbor_color = *allocator.get_color(&neighbor).unwrap();
                if neighbor_color != color {
                    continue;
                }
                // 判断是否和周围存在寄存器可以交换颜色
                // 选中第一个可以交换颜色的寄存器
                let mut neighbor_to_swap_to: Option<Reg> = None;
                for neighbor_to_neighbor in allocator.get_live_neighbors(&neighbor) {
                    if !allocator.if_has_been_colored(neighbor_to_neighbor) {
                        continue;
                    }
                    if allocator
                        .get_live_neighbors_bitmap(reg)
                        .contains(neighbor_to_neighbor.bit_code() as usize)
                    {
                        // 如果nn和n都是reg地邻接点,则它们交换颜色不会减少reg身边的指定颜色color的数量
                        continue;
                    }
                    if allocator.if_swapable_for_color(neighbor_to_neighbor, &neighbor) {
                        neighbor_to_swap_to = Some(*neighbor_to_neighbor);
                        break;
                    }
                }
                if let Some(neighbor_to_swap_to) = neighbor_to_swap_to {
                    // 如果可以交换颜色,获取交换颜色造成的代价
                    simpilfy_cost += allocator.eval_swap(&neighbor, &neighbor_to_swap_to);
                    // if reg.get_id() == 71 {
                    //     // println!("swap:{")
                    //     println!(
                    //         "{}\n{:?}",
                    //         allocator.get_available(reg),
                    //         allocator.get_num_neighbor_color(reg)
                    //     );
                    // }
                    allocator.swap_color(neighbor, neighbor_to_swap_to);
                    // if reg.get_id() == 71 {
                    //     println!(
                    //         "{}\n{:?}",
                    //         allocator.get_available(reg),
                    //         allocator.get_num_neighbor_color(reg)
                    //     );
                    // }

                    swap_list.push((neighbor, neighbor_to_swap_to));
                    remove_num_neibor_this_color += 1;
                } else {
                    for (reg1, reg2) in swap_list.iter().rev() {
                        allocator.swap_color(*reg1, *reg2);
                    }
                    return (swap_list, simpilfy_cost, false);
                }
            }

            if remove_num_neibor_this_color > times_to_remove {
                // log!(
                //     "num_live_neighbors:{},{:?}",
                //     num_live_neigbhors,
                //     allocator.get_live_neighbors(reg)
                // );
                unreachable!();
            }
            // 如果移除的数量少于原本数量,则交易失败
            if remove_num_neibor_this_color != times_to_remove {
                for (reg1, reg2) in swap_list.iter().rev() {
                    allocator.swap_color(*reg1, *reg2);
                }
                return (swap_list, simpilfy_cost, false);
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
            if i >= order.len() {
                break;
            }
            let color = *order.get(i).unwrap();
            let times_to_remove = *self.get_num_neighbor_color(&reg).get(&color).unwrap();
            // 判断这个颜色是否是合理的颜色
            if !tmp_regusestat.is_available_reg(color) {
                continue;
            }
            let (swap_list, simpilfy_cost, ok) = try_simplify(self, color, times_to_remove, &reg);
            if !ok {
                continue;
            } else if simpilfy_cost > spill_cost {
                // 如果可以分配,但是分配代价高昂,回退
                undo_simpilify(self, swap_list);
                continue;
            } else {
                //TOCHECK,化简成功,而且代价合适,则着色当前寄存器
                self.color_one(&reg);
                return true;
            }
        }
        self.push_to_tospill(&reg);
        false
    }

    #[inline]
    pub fn eval_swap(&self, reg1: &Reg, reg2: &Reg) -> f32 {
        // TODO,检查这里的值
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
        self.dump_swap_color(&reg1, &reg2);
        let info = self.info.as_ref().unwrap();
        let color1 = *info.colors.get(&reg1.get_id()).unwrap();
        let color2 = *info.colors.get(&reg2.get_id()).unwrap();
        self.decolor_one(&reg1);
        self.decolor_one(&reg2);
        self.color_one_with_certain_color(&reg1, color2);
        self.color_one_with_certain_color(&reg2, color1);
    }
}
