use super::*;

impl Allocator {
    #[inline]
    pub fn color_last(&mut self) {
        // 着色最后的节点
        let last_colors = &self.info.as_ref().unwrap().last_colors;
        let spillings = &self.info.as_ref().unwrap().spillings;
        let dstr = &self.info.as_ref().unwrap().colors;
        let mut to_color: Vec<(i32, i32)> =
            Vec::with_capacity(self.info.as_ref().unwrap().last_colors.len());
        let interference_graph = &self.info.as_ref().unwrap().all_neighbors;
        for reg in last_colors {
            // 计算其available
            let mut reg_use_stat = RegUsedStat::new();
            for reg in interference_graph.get(&reg).unwrap() {
                if reg.is_physic() {
                    reg_use_stat.use_reg(reg.get_color());
                } else {
                    if spillings.contains(&reg.get_id()) {
                        continue;
                    }
                    reg_use_stat.use_reg(*dstr.get(&reg.get_id()).unwrap());
                }
            }
            to_color.push((
                reg.get_id(),
                reg_use_stat.get_available_reg(reg.get_type()).unwrap(),
            ));
        }
        let dstr = &mut self.info.as_mut().unwrap().colors;
        for (reg, color) in to_color {
            dstr.insert(reg, color);
        }
    }

    // 检查是否当前k_graph中的节点都已经是合理的节点
    pub fn check_k_graph(&mut self) -> ActionResult {
        // 检查是否k_graph里面的值全部为真
        let mut out = ActionResult::Success;
        let mut new_biheap: BiHeap<OperItem> = BiHeap::new();
        loop {
            if self.info.as_ref().unwrap().k_graph.0.len() == 0 {
                break;
            }
            let item = self.info.as_mut().unwrap().k_graph.0.pop_min().unwrap();
            let map = &self.info.as_ref().unwrap().k_graph.1;
            if !map.contains(item.reg.bit_code() as usize) {
                // 如果不在k graph中了,则继续
                continue;
            }
            let reg = item.reg;
            if !self.is_k_graph_node(&reg) {
                out = ActionResult::Unfinish;
                let new_item = self.draw_spill_div_nlc_item(&reg);
                self.info.as_mut().unwrap().to_color.push(new_item);
                continue;
            }
            let new_item = self.draw_spill_div_nlc_item(&reg);
            new_biheap.push(new_item);
        }
        if self.info.as_ref().unwrap().k_graph.0.len() == 0 {
            self.info.as_mut().unwrap().k_graph.0 = new_biheap;
        } else {
            new_biheap.iter().for_each(|item| {
                self.info.as_mut().unwrap().k_graph.0.push(*item);
            });
        }
        out
    }
    /// 在color_k_graph之前应该check k graph<br>
    ///  给剩余地悬点进行着色  (悬点并未进入spilling中,所以仍然获取到周围地颜色)
    pub fn color_k_graph(&mut self) -> ActionResult {
        // 对最后的k个节点进行着色
        loop {
            let k_graph = &mut self.info.as_mut().unwrap().k_graph;
            if k_graph.0.is_empty() {
                break;
            }
            let item = k_graph.0.pop_min().unwrap();
            let reg = item.reg;
            let available = self.draw_available_and_num_neigbhor_color(&reg);
        }

        ActionResult::Success
    }

    // 判断某个就节点是否是悬点
    #[inline]
    pub fn is_k_graph_node(&mut self, reg: &Reg) -> bool {
        self.get_available(reg).num_available_regs(reg.get_type())
            > self.get_num_of_live_neighbors(reg)
    }

    #[inline]
    pub fn remove_from_k_graph(&mut self, reg: &Reg) {
        self.info
            .as_mut()
            .unwrap()
            .k_graph
            .1
            .remove(reg.bit_code() as usize);
    }
}
