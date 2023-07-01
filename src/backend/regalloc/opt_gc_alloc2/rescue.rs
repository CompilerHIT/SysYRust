use super::*;

impl Allocator {
    #[inline]
    pub fn rescue(&mut self) -> ActionResult {
        // 从已经spill的寄存器旁边,根据spill cost删掉几个周围的寄存器,然后把脱离color的寄存器加入
        // 删除的操作可以局限与一轮,也可以局限于2轮
        // 在局部节点中判断是否能够产生优化操作
        todo!()
    }
}
