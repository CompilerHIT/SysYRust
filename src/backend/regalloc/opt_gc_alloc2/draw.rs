use super::*;
impl Allocator {
    /// 该函数只应该在获取最终结果的时候调用一次 (返回dstr和spilling形式(从Allocator手中复制))
    #[inline]
    pub fn draw_dstr_spillings(&mut self) -> (HashMap<i32, i32>, HashSet<i32>) {
        // TODO,把to rescue中的内容交回spillings
        let dstr = self.info.as_ref().unwrap().colors.to_owned();
        let spillings = self.info.as_ref().unwrap().spillings.to_owned();
        (dstr, spillings)
    }

    ///根据总冲突图获取可着色状态
    #[inline]
    pub fn draw_available(&self, reg: &Reg) -> RegUsedStat {
        let mut available = RegUsedStat::new();
        self.get_all_neighbors(reg).iter().for_each(|neighbor| {
            if neighbor.is_physic() {
                available.use_reg(neighbor.get_color());
            } else if self.if_has_been_colored(neighbor) {
                available.use_reg(*self.get_color(neighbor).unwrap());
            }
        });
        available
    }

    #[inline]
    /// 根据总冲突图刷新并返回regusestat和num neighbor color
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
            if self.is_last_colored(neighbor) {
                continue;
            }
            if info.spillings.contains(&neighbor.get_id()) {
                continue;
            }
            let mut color: Option<i32> = None;
            if neighbor.is_physic() {
                color = Some(neighbor.get_color());
            } else {
                color = Some(*self.get_color(neighbor).unwrap());
            }
            let color = color.unwrap();
            available.use_reg(color);
            let new_num = nnc.get(&color).unwrap_or(&0) + 1;
            nnc.insert(color, new_num);
        }
        (available, nnc)
    }

    ///绘制item, 绘制(reg,spill_cost/num_live_neigbhor) item
    #[inline]
    pub fn draw_spill_div_nln_item(&self, reg: &Reg) -> OperItem {
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

    /// 绘制以spill cost为代价的item
    pub fn draw_spill_cost_item(&self, reg: &Reg) -> OperItem {
        let cost = self.get_spill_cost(reg);
        OperItem {
            reg: *reg,
            cost: cost,
        }
    }

    /// 绘制以 num_available_color(可选择的着色的数量) / num_live_neigbhor (周围未spill的寄存器的数量)为代价的item,<br>
    ///
    pub fn draw_na_div_nln_item(&self, reg: &Reg) -> OperItem {
        let na = self.draw_available(reg).num_available_regs(reg.get_type()) as f32;
        let nln = self.get_live_neighbors_bitmap(reg).len() as f32;
        // 考虑到nln可能为0，可能会/0错误，所以加1
        // TODO, 衡量+1的影响，考虑是否有其他函数可以处理这个问题
        OperItem {
            reg: *reg,
            cost: na / (nln + 0.01),
        }
    }

    /// 绘制 以 周围活跃寄存器数量/自身着色可用寄存器数量 为cost的 item
    ///
    pub fn draw_nln_div_na_item(&self, reg: &Reg) -> OperItem {
        let na = self.draw_available(reg).num_available_regs(reg.get_type()) as f32;
        let nln = self.get_live_neighbors_bitmap(reg).len() as f32;
        // 考虑到nln可能为0，可能会/0错误，所以加1
        // TODO, 衡量+1的影响，考虑是否有其他函数可以处理这个问题
        OperItem {
            reg: *reg,
            cost: nln / (na + 0.01),
        }
    }

    /// 绘制以 num_live_neighbors为cost的item
    pub fn draw_nln_item(&self, reg: &Reg) -> OperItem {
        let nln = self.get_live_neighbors_bitmap(reg).len() as f32;
        OperItem {
            reg: *reg,
            cost: nln,
        }
    }
    ///绘制live neighbor图
    pub fn draw_live_neighbors(&self, reg: &Reg) -> (LinkedList<Reg>, Bitmap) {
        let mut live_neighbors = LinkedList::new();
        let mut live_neighbors_bitmap = Bitmap::with_cap(300);
        for neighbor in self.get_all_neighbors(reg) {
            if neighbor.is_physic() {
                continue;
            }
            if self.is_last_colored(reg) {
                continue;
            }
            if self.if_has_been_spilled(neighbor) {
                continue;
            }
            live_neighbors.push_back(*neighbor);
            live_neighbors_bitmap.insert(neighbor.bit_code() as usize);
        }
        (live_neighbors, live_neighbors_bitmap)
    }
}
