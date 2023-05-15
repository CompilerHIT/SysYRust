///! 此文件为 load 指令的实现
use super::{Inst, InstKind};

impl Inst {
    /// 加载一个int值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_int_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::Int,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 加载一个全局int值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_int_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::Int,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 加载一个int数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_int_array_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::Int,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 加载一个全局int数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_int_array_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::IntPtr,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 加载一个float值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_float_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::Float,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 加载一个全局float值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_float_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::Float,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 加载一个float数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_float_array_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::Float,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 加载一个全局float数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_float_array_load(ptr: &Inst, offset: &Inst) -> Self {
        Self::new(
            crate::ir::ir_type::IrType::FloatPtr,
            InstKind::Load,
            vec![ptr, offset],
        )
    }

    /// 获得指针
    /// # Return
    /// 返回指针的引用
    pub fn get_ptr(&self) -> &Inst {
        &self.user.get_operand(0)
    }

    /// 获得偏移量
    /// # Return
    /// 返回偏移量的引用
    pub fn get_offset(&self) -> &Inst {
        &self.user.get_operand(1)
    }

    /// 修改指针
    /// # Arguments
    /// * 'ptr' - 新的指针
    pub fn set_ptr(&mut self, ptr: &Inst) {
        self.user.set_operand(0, ptr);
    }

    /// 修改偏移量
    /// # Arguments
    /// * 'offset' - 新的偏移量
    pub fn set_offset(&mut self, offset: &Inst) {
        self.user.set_operand(1, offset);
    }
}
