///! 此文件为Inst中关于内存分配指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 申请一个int类型的数组
    ///
    /// # Arguments
    /// * 'length' - 要申请的数组的长度
    /// # Returns
    /// 构造好的数组指令，其值为Intptr
    pub fn make_int_array(&mut self, length: ObjPtr<Inst>) -> ObjPtr<Inst> {
        self.put(Inst::new(
            crate::ir::ir_type::IrType::IntPtr,
            InstKind::Alloca,
            vec![length],
        ))
    }

    /// 申请一个double类型的数组
    ///
    /// # Arguments
    /// * 'length' - 要申请的数组的长度
    /// # Returns
    /// 构造好的数组指令，其值为Floatptr
    pub fn make_double_array(&mut self, length: ObjPtr<Inst>) -> ObjPtr<Inst> {
        self.put(Inst::new(
            crate::ir::ir_type::IrType::FloatPtr,
            InstKind::Alloca,
            vec![length],
        ))
    }
}

impl Inst {
    /// 获得数组长度
    pub fn get_array_length(&self) -> ObjPtr<Inst> {
        self.user.get_operand(0)
    }

    /// 设置数组长度
    pub fn set_array_length(&mut self, length: ObjPtr<Inst>) {
        self.user.set_operand(0, length);
    }
}
