use super::*;

impl Allocator {
    #[inline]
    pub fn color_last(&mut self) {
        // 着色最后的节点
        let last_colors = &self.info.as_ref().unwrap().last_colors;
        let mut tocolors = Vec::with_capacity(last_colors.len());
        for reg in last_colors {
            tocolors.push(*reg);
        }
        for reg in tocolors {
            let available = self.draw_available(&reg);
            let color = available.get_available_reg(reg.get_type()).unwrap();
            self.get_mut_colors().insert(reg.get_id(), color);
        }
    }

    // 把一个虚拟寄存器加入 k_graph
    pub fn push_to_k_graph(&mut self, reg: &Reg) {
        // 加入虚拟寄存器的k_graph item 以 num_available/num live neighbor为权重
        // 检查的时候优先检查权重小的
        // 这样可以优先检查到不在k-graph的节点
        let item = self.draw_na_div_nln_item(reg);
        self.info.as_mut().unwrap().k_graph.0.push(item);
        self.info
            .as_mut()
            .unwrap()
            .k_graph
            .1
            .insert(reg.bit_code() as usize);
    }

    // 检查是否当前k_graph中的节点都已经是合理的节点
    // 如果k_graph中的节点不是已经
    pub fn check_k_graph(&mut self) -> ActionResult {
        // 检查是否k_graph里面的值全部为真
        let mut out = ActionResult::Success;
        let mut new_biheap: BiHeap<OperItem> = BiHeap::new();
        loop {
            if self.info.as_ref().unwrap().k_graph.0.len() == 0 {
                break;
            }
            let item = self.info.as_mut().unwrap().k_graph.0.pop_min().unwrap();
            let reg = item.reg;
            if self.if_has_been_colored(&reg) || self.if_has_been_spilled(&reg) {
                // unreachable!();
                continue;
            }
            if !self
                .info
                .as_ref()
                .unwrap()
                .k_graph
                .1
                .contains(reg.bit_code() as usize)
            {
                continue;
            }
            let (na, nln) = self.get_num_available_and_num_live_neighbor(&reg);
            if na <= nln {
                self.info
                    .as_mut()
                    .unwrap()
                    .k_graph
                    .1
                    .remove(reg.bit_code() as usize);
                out = ActionResult::Fail;
                self.push_to_tocolor(&reg);
                break;
            }
            new_biheap.push(item);
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
        assert!(true);
        loop {
            let k_graph = &mut self.info.as_mut().unwrap().k_graph;
            if k_graph.0.is_empty() {
                break;
            }
            // println!("{}", k_graph.0.len());
            let item = k_graph.0.pop_min().unwrap();
            let reg = item.reg;
            // println!("{}", reg);
            let available = self.draw_available(&reg);
            let color = available.get_available_reg(reg.get_type()).unwrap();
            self.get_mut_colors().insert(reg.get_id(), color);
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
