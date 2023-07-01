use super::*;
impl Allocator {
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
    pub fn get_color(&self, reg: &Reg) -> Option<&i32> {
        if reg.is_physic() {
            unreachable!();
        }
        self.info.as_ref().unwrap().colors.get(&reg.get_id())
    }

    #[inline]
    pub fn get_available(&self, reg: &Reg) -> RegUsedStat {
        *self.info.as_ref().unwrap().availables.get(reg).unwrap()
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
    pub fn get_num_of_live_neighbors(&self, reg: &Reg) -> usize {
        self.info
            .as_ref()
            .unwrap()
            .all_live_neighbors
            .get(reg)
            .unwrap()
            .len()
    }

    // 获取的可用颜色以及周围的活邻居数量
    pub fn get_num_available_and_num_live_neighbor(&self, reg: &Reg) -> (i32, i32) {
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
