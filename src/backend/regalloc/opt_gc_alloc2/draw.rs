use super::*;
impl Allocator {
    #[inline]
    pub fn draw_dstr_spillings(&mut self) -> (HashMap<i32, i32>, HashSet<i32>) {
        // TODO,把to rescue中的内容交回spillings
        let dstr = self.info.as_ref().unwrap().colors.to_owned();
        let spillings = self.info.as_ref().unwrap().spillings.to_owned();
        (dstr, spillings)
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
}
