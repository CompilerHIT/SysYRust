use super::*;

impl Allocator {
    #[inline]
    pub fn rescue(&mut self) -> ActionResult {
        // 记录原本的代价,记录下动作序列
        // 如果rescue后的收获大于代价,则rescue采用,否则回退
        // 在torescue中选择旁边的节点数最大的一个寄存器(to rescue寄存器中的寄存器一定是已经被spill了的),将它despill
        // 首先despill该虚拟寄存器
        // 然后把所有decolor的虚拟寄存器以及所有despill了的虚拟寄存器加入到我们的tocolor中
        // 然后进行color,spill流程,
        // 以spill/nln为cost,优先color cost最大的节点
        // 计算这个过程产生的收获，
        // color到的虚拟寄存器的spill cost之和就是收获,收获越大越好,
        let out = ActionResult::Finish;
        loop {
            let mut old_val: f32 = 0.0; //记录旧的价值
            let mut new_val: f32 = 0.0; //新的价值
            let mut old_colored: LinkedList<Reg> = LinkedList::new();
            let mut old_spilled: LinkedList<Reg> = LinkedList::new();
            let mut new_colored: LinkedList<Reg> = LinkedList::new();
            if self.get_mut_torescue().len() == 0 {
                break;
            }
            let torescue_one = self.info.as_mut().unwrap().to_rescue.pop_max().unwrap();
            let reg = torescue_one.reg;
            if !self.if_has_been_spilled(&reg) {
                continue;
            }
            self.despill_one(&reg);
            old_spilled.push_back(reg);

            for neighbor in self.get_all_neighbors(&reg) {
                //
                if self.if_has_been_colored(neighbor) {
                    old_colored.push_back(*neighbor);
                    old_val += self.get_spill_cost(neighbor);
                } else {
                    old_spilled.push_back(*neighbor);
                }
            }
            // let mut passed = Bitmap::new();
            // 把所有影响到的节点加入寄存器
            for torescue in &old_colored {
                self.decolor_one(torescue);
                self.push_to_tocolor_for_rescue(torescue);
                // 判断这个寄存器周围是否有没有颜色而且没有在tocolor for rescue中的节点
            }
            for torescue in &old_spilled {
                self.despill_one(torescue);
                self.push_to_tocolor_for_rescue(torescue);
            }
            // 然后对tocolor中的内容进行着色
            loop {
                let tocolors = self.get_mut_tocolor();
                if tocolors.len() == 0 {
                    break;
                }
                let item = tocolors.pop_max().unwrap();
                if self.color_one(&item.reg) || self.simpilfy_one(item.reg) {
                    new_val += self.get_spill_cost(&item.reg);
                    new_colored.push_back(item.reg);
                    continue;
                }
                self.spill_one(item.reg);
            }
            if new_val > old_val {
                return ActionResult::Success;
            }
            // 否则化简失败,恢复开始情况
            for reg in new_colored {
                self.decolor_one(&reg);
            }
            if self.if_has_been_colored(&torescue_one.reg) {
                self.decolor_one(&torescue_one.reg);
            }
            //
            for reg in old_colored {
                if self.if_has_been_spilled(&reg) {
                    self.despill_one(&reg);
                }
                self.push_to_k_graph(&reg);
            }
            self.color_k_graph();
            return ActionResult::Fail;
        }
        out
    }
}
