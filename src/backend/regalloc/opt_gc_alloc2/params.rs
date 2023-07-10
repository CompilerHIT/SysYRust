use std::collections::btree_map::IterMut;

use super::*;

// 调参池
impl Allocator {
    /// 把一个寄存器加入tocolor
    /// 以(reg,cost)作为操作单元
    /// tocolor每次会选择cost最大的单元进行try color操作
    /// 当前以 num_live_neighbors/num_available_colors作为参数
    /// 时间复杂度O(logn)
    pub fn push_to_tocolor(&mut self, reg: &Reg) {
        self.dump_action("tocolor", reg);
        // let item = self.draw_spill_div_nln_item(reg);
        let item = self.draw_nln_div_na_item(reg);
        // let item = self.draw_na_div_nln_item(reg);
        // let item = self.draw_nln_div_na_item(reg);
        // let item = self.draw_nln_item(reg);
        self.info.as_mut().unwrap().to_color.push(item);
    }

    pub fn push_to_tocolor_for_rescue(&mut self, reg: &Reg) {
        self.dump_action("tocolor", reg);
        let item = self.draw_spill_div_nln_item(reg);
        self.info.as_mut().unwrap().to_color.push(item);
    }

    /// 把(reg,cost)加入tosimplify
    /// tosimplify每次会选择costs最大的节点进行操作
    /// time:O(logn)
    #[inline]
    pub fn push_to_tosimpilfy(&mut self, reg: &Reg) {
        self.dump_action("tosimplify", reg);
        // 把一个节点加入待着色列表中
        let item = self.draw_nln_item(reg);
        // let item = self.draw_spill_div_nln_item(reg);
        self.info.as_mut().unwrap().to_simplify.push(item);
    }

    ///把(reg,cost)加入tospill
    /// tospill每次会选择一个cost最小的单元先进行操作
    /// 当前以 spill_cost/num_live_neighbors作为 cost
    pub fn push_to_tospill(&mut self, reg: &Reg) {
        self.dump_action("tospill", reg);
        // TOCHECK,修改tospill权重为spillcost
        let item = self.draw_spill_cost_item(reg);
        // let item = self.draw_spill_div_nln_item(reg);
        self.get_mut_tospill().push(item);
    }

    // 把一个虚拟寄存器加入 k_graph
    pub fn push_to_k_graph(&mut self, reg: &Reg) {
        // 加入虚拟寄存器的k_graph item 以 num_available/num live neighbor为权重
        // 检查的时候优先检查权重小的
        // 这样可以优先检查到不在k-graph的节点
        let item = self.draw_nln_item(reg);
        self.info.as_mut().unwrap().k_graph.0.push(item);
        self.info
            .as_mut()
            .unwrap()
            .k_graph
            .1
            .insert(reg.bit_code() as usize);
    }

    /// 选择spill节点
    /// 在reg和reg邻居节点中选择一个最适合spill的节点
    ///
    #[inline]
    pub fn choose_spill(&self, reg: &Reg) -> Reg {
        //在该节点和该节点的周围节点中选择一个最适合spill的节点
        // 最适合spill的节点就是spill代价最小的节点
        // spill代价计算:  活邻居越多,spill代价越小,spill_cost越大,spill代价越大,
        // 能够救回的节点的代价越大,spiLl代价越小
        // val[reg]=reg.spill_cost/num_live_neighbor[reg] - sum(rescue.spill_cost/num_live_neighbor[reg])
        let val = |allocator: &Allocator, reg: &Reg| -> f32 {
            // 计算价值,首先,获取当前节点本身的spill cost(简单地使用spill cost来计算节省地内容)
            let mut out_val = self.get_spill_cost_div_lnn(reg);
            // 如果当前节点在colors里面,则spill cost还要减去消去它的颜色后能够救回的spill cost
            // 对该节点地邻居进行一次遍历(如果该节点有颜色的话)
            let color = self.get_color(reg);
            if color.is_none() {
                return out_val;
            }
            // TODO, 考虑边迹效应,遇到能够拯救多个节点的情况,调整下增加/减少权重的系数
            let color = *color.unwrap();
            for neighbor in self.info.as_ref().unwrap().all_neighbors.get(reg).unwrap() {
                if neighbor.is_physic()
                    || self.if_has_been_colored(neighbor)
                    || self.is_last_colored(neighbor)
                {
                    continue;
                }
                let nnc = self
                    .info
                    .as_ref()
                    .unwrap()
                    .nums_neighbor_color
                    .get(neighbor)
                    .unwrap();
                if *nnc.get(&color).unwrap_or(&0) == 1 {
                    out_val -= self.get_spill_cost_div_lnn2(neighbor);
                }
            }
            out_val
        };
        // 遍历节点reg和它周围节点
        let mut tospill = *reg;
        let info = self.info.as_ref().unwrap().to_owned();
        let all_live_neigbhors = &info.all_live_neighbors;
        let all_live_neigbors_bitmap = &info.all_live_neigbhors_bitmap;
        let mut tospill_val = val(self, reg);
        let bitmap = all_live_neigbors_bitmap.get(reg).unwrap();
        // 只在活着的节点(也就是没有被spill的节点中选择)
        // TODO,
        // 改进这里的选择
        // 在周围没有作色的节点和自己中选择要spill的对象
        // 如果节点有颜色，而且spill掉节点后能够让自己作色,且收益更高，则选择节点
        for neighbor in all_live_neigbhors.get(reg).unwrap() {
            let neigbor = *neighbor;
            // TOCHECK,只选择已经 colored的节点
            if !self.if_has_been_colored(&neigbor) {
                continue;
            }
            // TOCHEK,排除spill后不能够救自己的邻居节点
            let color = self.get_color(&neigbor).unwrap();
            if self.get_num_neighbor_color(&reg).get(color).unwrap() != &1 {
                continue;
            }

            if !bitmap.contains(neigbor.bit_code() as usize) {
                continue;
            }
            //
            // if self.if_has_been_colored(&neigbor) {
            //     let color = self.get_color(&neigbor).unwrap();
            //     if self.get_num_neighbor_color(reg).get(color).unwrap() != &1 {
            //         continue;
            //     }
            // }
            // 获取价值
            let tmp_tospill_val = val(self, &neigbor);
            if tmp_tospill_val < tospill_val {
                tospill = neigbor;
                tospill_val = tmp_tospill_val;
            }
        }
        tospill
    }
}
