///! 此文件为 load 指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 加载一个int值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_int_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            IntPtr => {}
            _ => unreachable!("ObjPool::make_int_load: ptr must be a pointer"),
        }
        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Int,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }

    /// 加载一个全局int值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_int_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            // 全局变量作为指针，但是其值的类型仍为Int
            IrType::Int | IrType::ConstInt => {}
            _ => unreachable!("ObjPool::make_global_int_load: ptr must be a pointer"),
        }

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Int,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }
    /// 加载一个int数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_int_array_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            IntPtr => {}
            _ => unreachable!("ObjPool::make_int_array_load: ptr must be a pointer"),
        }

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Int,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }

    /// 加载一个全局int数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_int_array_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            IrType::IntPtr => {}
            _ => unreachable!("ObjPool::make_global_int_array_load: ptr must be a pointer"),
        }

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::IntPtr,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }
    /// 加载一个float值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_float_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            IrType::FloatPtr => {}
            _ => unreachable!("ObjPool::make_float_load: ptr must be a pointer"),
        }

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Float,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }

    /// 加载一个全局float值
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_float_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            IrType::Float | IrType::ConstFloat => {}
            _ => unreachable!("ObjPool::make_global_float_load: ptr must be a pointer"),
        }

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Float,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }

    /// 加载一个float数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_float_array_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            IrType::FloatPtr => {}
            _ => unreachable!("ObjPool::make_float_array_load: ptr must be a pointer"),
        }

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::Float,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }

    /// 加载一个全局float数组
    /// # Arguments
    /// * 'ptr' - 需要加载的指针
    /// * 'offset' - 偏移量
    /// # Return
    /// 返回一个Inst实例
    pub fn make_global_float_array_load(&mut self, ptr: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.as_ref().get_ir_type() {
            IrType::FloatPtr => {}
            _ => unreachable!("ObjPool::make_global_float_array_load: ptr must be a pointer"),
        }

        let inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::FloatPtr,
            InstKind::Load,
            vec![ptr],
        ));

        // 设置use list
        ptr.as_mut().add_user(inst.as_ref());

        inst
    }
}

impl Inst {
    /// 获得指针
    /// # Return
    /// 返回指针的引用
    pub fn get_ptr(&self) -> ObjPtr<Inst> {
        self.user.get_operand(0)
    }

    /// 修改指针
    /// # Arguments
    /// * 'ptr' - 新的指针
    pub fn set_ptr(&mut self, ptr: ObjPtr<Inst>) {
        // 修改use list
        self.get_ptr().as_mut().remove_user(self);
        ptr.as_mut().add_user(self);

        self.user.set_operand(0, ptr);
    }
}