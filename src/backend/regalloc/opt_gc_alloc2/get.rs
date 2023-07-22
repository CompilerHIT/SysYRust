use super::*;
impl Allocator {
    ///获取寄存器的一些属性
    /// * 周围已有的各色物理寄存器数量
    /// * 自身剩余可着色空间
    /// * 自身是否已经着色
    /// * 自身是否已经spill
    #[inline]
    pub fn get_colors(&self) -> &HashMap<i32, i32> {
        &self.info.as_ref().unwrap().colors
    }

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
    pub fn get_all_neighbors(&self, reg: &Reg) -> &LinkedList<Reg> {
        self.info.as_ref().unwrap().all_neighbors.get(reg).unwrap()
    }
    #[inline]
    pub fn get_mut_all_neighbors(&mut self, reg: &Reg) -> &mut LinkedList<Reg> {
        self.info
            .as_mut()
            .unwrap()
            .all_neighbors
            .get_mut(reg)
            .unwrap()
    }

    #[inline]
    pub fn get_spill_cost(&self, reg: &Reg) -> f32 {
        *self.info.as_ref().unwrap().spill_cost.get(reg).unwrap()
    }

    #[inline]
    pub fn get_color(&self, reg: &Reg) -> Option<&i32> {
        if reg.is_physic() {
            unreachable!();
        }
        self.info.as_ref().unwrap().colors.get(&reg.get_id())
    }

    #[inline]
    pub fn get_available(&self, reg: &Reg) -> &RegUsedStat {
        self.info.as_ref().unwrap().availables.get(reg).unwrap()
    }
    #[inline]
    pub fn get_mut_available(&mut self, reg: &Reg) -> &mut RegUsedStat {
        self.info.as_mut().unwrap().availables.get_mut(reg).unwrap()
    }
    #[inline]
    pub fn get_num_neighbor_color(&self, reg: &Reg) -> &HashMap<i32, i32> {
        self.info
            .as_ref()
            .unwrap()
            .nums_neighbor_color
            .get(reg)
            .unwrap()
    }
    #[inline]
    pub fn get_mut_num_neighbor_color(&mut self, reg: &Reg) -> &mut HashMap<i32, i32> {
        self.info
            .as_mut()
            .unwrap()
            .nums_neighbor_color
            .get_mut(reg)
            .unwrap()
    }
    #[inline]
    pub fn get_mut_live_neighbors(&mut self, reg: &Reg) -> &mut LinkedList<Reg> {
        self.info
            .as_mut()
            .unwrap()
            .all_live_neighbors
            .get_mut(reg)
            .unwrap()
    }
    #[inline]
    pub fn get_live_neighbors(&self, reg: &Reg) -> &LinkedList<Reg> {
        self.info
            .as_ref()
            .unwrap()
            .all_live_neighbors
            .get(reg)
            .unwrap()
    }
    #[inline]
    pub fn get_live_neighbors_bitmap(&self, reg: &Reg) -> &Bitmap {
        self.info
            .as_ref()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get(reg)
            .unwrap()
    }
    #[inline]
    pub fn get_mut_live_neigbhors_bitmap(&mut self, reg: &Reg) -> &mut Bitmap {
        self.info
            .as_mut()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get_mut(reg)
            .unwrap()
    }

    #[inline]
    pub fn get_mut_colors(&mut self) -> &mut HashMap<i32, i32> {
        &mut self.info.as_mut().unwrap().colors
    }

    #[inline]
    pub fn get_num_of_live_neighbors(&self, _reg: &Reg) -> usize {
        self.info.as_ref().unwrap().all_live_neigbhors_bitmap.len()
    }

    // 获取的可用颜色以及周围的活邻居数量
    #[inline]
    pub fn get_num_available_and_num_live_neighbor(&self, reg: &Reg) -> (i32, i32) {
        let na = self.get_available(reg).num_available_regs(reg.get_type());
        let nn = self.get_num_of_live_neighbors(reg);
        (na as i32, nn as i32)
    }
}

/// 获取自身多种寄存器集合
impl Allocator {
    pub fn get_tocolor(&self) -> &BiHeap<OperItem> {
        &self.info.as_ref().unwrap().to_color
    }
    pub fn get_mut_tocolor(&mut self) -> &mut BiHeap<OperItem> {
        &mut self.info.as_mut().unwrap().to_color
    }
    pub fn get_mut_tosimplify(&mut self) -> &mut BiHeap<OperItem> {
        &mut self.info.as_mut().unwrap().to_simplify
    }
    pub fn get_mut_tospill(&mut self) -> &mut BiHeap<OperItem> {
        &mut self.info.as_mut().unwrap().to_spill
    }
    pub fn get_mut_torescue(&mut self) -> &mut BiHeap<OperItem> {
        &mut self.info.as_mut().unwrap().to_rescue
    }
}

///last colors相关处理
impl Allocator {
    pub fn get_mut_last_colors_lst(&mut self) -> &mut LinkedList<Reg> {
        &mut self.info.as_mut().unwrap().last_colors_lst
    }
    pub fn get_last_colors_lst(&self) -> &LinkedList<Reg> {
        &self.info.as_ref().unwrap().last_colors_lst
    }
}
