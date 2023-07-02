use super::*;

impl Allocator {
    #[inline]
    pub fn if_has_been_spilled(&self, reg: &Reg) -> bool {
        self.info
            .as_ref()
            .unwrap()
            .spillings
            .contains(&reg.get_id())
    }
    #[inline]
    pub fn if_has_been_colored(&self, reg: &Reg) -> bool {
        self.info
            .as_ref()
            .unwrap()
            .colors
            .contains_key(&reg.get_id())
    }

    ///判断是否已经加入到k-graph
    #[inline]
    pub fn if_has_been_added_to_k_graph(&self, reg: &Reg) -> bool {
        self.info
            .as_ref()
            .unwrap()
            .k_graph
            .1
            .contains(reg.bit_code() as usize)
    }

    #[inline]
    pub fn if_swapable_for_color(&self, reg1: &Reg, reg2: &Reg) -> bool {
        // 判断两个寄存器的颜色是否能够发生交换
        if !self.if_has_been_colored(reg1) || !self.if_has_been_colored(reg2) {
            return false;
        }
        // 判断
        let color1 = *self.get_color(reg1).unwrap();
        let color2 = *self.get_color(reg2).unwrap();
        let nncs = &self.info.as_ref().unwrap().nums_neighbor_color;
        let color2_times_around_reg1 = nncs.get(reg1).unwrap().get(&color2).unwrap_or(&0);
        let color1_times_arount_reg2 = nncs.get(reg2).unwrap().get(&color1).unwrap_or(&0);
        if self
            .info
            .as_ref()
            .unwrap()
            .all_live_neigbhors_bitmap
            .get(reg1)
            .unwrap()
            .contains(reg2.bit_code() as usize)
        {
            if *color2_times_around_reg1 == 1 && *color1_times_arount_reg2 == 1 {
                return true;
            }
            return false;
        }
        if color1_times_arount_reg2 == &0 || color2_times_around_reg1 == &0 {
            return true;
        }
        false
    }
    #[inline]
    pub fn is_last_colored(&self, reg: &Reg) -> bool {
        self.info.as_ref().unwrap().last_colors.contains(reg)
    }
}
