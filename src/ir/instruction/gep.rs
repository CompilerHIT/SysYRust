///! 此模块为 GEP 指令提供了实现
use super::*;
use crate::utility::ObjPool;

impl ObjPool<Inst> {
    /// 构造一个 GEP 指令
    ///
    /// # Arguments
    /// * 'ptr' - 要计算偏移的指针
    /// * 'offset' - 偏移量
    /// # Returns
    /// 构造好的 GEP 指令
    pub fn make_gep(&mut self, mut ptr: ObjPtr<Inst>, mut offset: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        match ptr.get_ir_type() {
            IrType::IntPtr | IrType::FloatPtr => match offset.get_ir_type() {
                IrType::Int => {}
                _ => unreachable!("ObjPool::make_gep: offset must be a int"),
            },
            _ => unreachable!("ObjPool::make_gep: ptr must be a pointer"),
        };

        let inst = self.put(Inst::new(
            ptr.get_ir_type(),
            InstKind::Gep,
            vec![ptr, offset],
        ));

        // 设置use_list
        ptr.add_user(inst.as_ref());
        offset.add_user(inst.as_ref());

        inst
    }
}

impl Inst {
    /// 获得 GEP 指令的指针
    pub fn get_gep_ptr(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::Gep = self.get_kind() {
        } else {
            unreachable!("Inst::get_gep_ptr")
        }

        self.user.get_operand(0)
    }

    /// 设置 GEP 指令的指针
    pub fn set_gep_ptr(&mut self, mut ptr: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Gep = self.get_kind() {
            match ptr.get_ir_type() {
                IrType::IntPtr | IrType::FloatPtr => {}
                _ => unreachable!("Inst::set_gep_ptr: ptr must be a pointer"),
            };
        } else {
            unreachable!("Inst::set_gep_ptr")
        }

        // 设置use_list
        self.user.get_operand(0).as_mut().remove_user(self);
        ptr.add_user(self);

        self.user.set_operand(0, ptr);
    }

    /// 获得 GEP 指令的偏移量
    pub fn get_gep_offset(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::Gep = self.get_kind() {
        } else {
            unreachable!("Inst::get_gep_offset")
        }

        self.user.get_operand(1)
    }

    /// 设置 GEP 指令的偏移量
    pub fn set_gep_offset(&mut self, mut offset: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Gep = self.get_kind() {
            match offset.get_ir_type() {
                IrType::Int => {}
                _ => unreachable!("Inst::set_gep_offset: offset must be a int"),
            };
        } else {
            unreachable!("Inst::set_gep_offset")
        }

        // 设置use_list
        self.user.get_operand(1).as_mut().remove_user(self);
        offset.add_user(self);

        self.user.set_operand(1, offset);
    }
}
