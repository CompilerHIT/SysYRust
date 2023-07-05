use core::panic;

use super::*;
impl Allocator {
    /// 溢出
    /// * 从待溢出列表中选择一个最优溢出项目进行溢出处理
    /// * 如果溢出列表为空,返回Finish
    /// * 溢出成功返回Success  (溢出是肯定能够成功的)
    /// * 溢出失败返回Fail (比如to_spill对象已经过期,被着色了/被spill了 )
    pub fn spill(&mut self) -> ActionResult {
        // sill 直到没有tospill或者直到出现新的可color的节点
        // spill先从 spillcost较小的,邻居度较大的开始
        if self.info.as_ref().unwrap().to_spill.is_empty() {
            return ActionResult::Finish;
        }
        // 试图拯救to_rescue中spill代价最大的节点
        // 如果spill后能够出现可以着色的节点,则算spill成功,先结束这次spill
        let item = self.get_mut_tospill().pop_min().unwrap();
        //判断是否已经被拯救,
        let reg = item.reg;
        if self.if_has_been_colored(&reg) || self.if_has_been_spilled(&reg) {
            return ActionResult::Fail;
        }
        //
        let tospill = self.choose_spill(&reg);
        if tospill != reg {
            // 如果要溢出的寄存器不等于选择的寄存器,需要把选择的寄存器再加入to_color中
            self.push_to_tocolor(&reg);
        }
        // 溢出操作一定成功
        if self.if_has_been_colored(&tospill) {
            self.decolor_one(&tospill);
        }
        self.spill_one(tospill);
        ActionResult::Success
    }

    #[inline]
    // 如果spill过程救活了一些节点,则返回true,否则返回false
    pub fn spill_one(&mut self, reg: Reg) {
        self.dump_action("spill", &reg);
        // spill reg本身或者周围的某个有色寄存器,选择一个结果好的,判断丢弃寄存器后是否产生新的好处
        // spill reg本身,
        if self.if_has_been_spilled(&reg) {
            panic!("u");
        }
        if self.if_has_been_colored(&reg) {
            unreachable!();
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
            if self.if_has_been_spilled(&neighbor) {
                continue;
            }
            // 对于邻居非spilling的情况
            let info = &mut self.info.as_mut().unwrap();
            // 首先把节点放回live_neigbhors
            info.all_live_neigbhors_bitmap
                .get_mut(&neighbor)
                .unwrap()
                .remove(reg.bit_code() as usize);
        }
    }

    #[inline]
    pub fn despill_one(&mut self, reg: &Reg) {
        self.dump_action("despill", reg);
        // 从spill中取东西回来要把东西加回live negibhores中
        // 需要修改live_neigbhors,用到allneighbors,spillings,
        if !self.if_has_been_spilled(reg) || self.if_has_been_colored(reg) {
            panic!("gg");
        }
        // 首先从spill移除
        self.info.as_mut().unwrap().spillings.remove(&reg.get_id());

        //然后刷新available和 nums_neighbor_color
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
        self.push_to_tocolor(reg);
    }
}
