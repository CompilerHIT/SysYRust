///! 此文件为Inst中关于内存分配指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 申请一个int类型的数组
    ///
    /// # Arguments
    /// * 'length' - 要申请的数组的长度
    /// # Returns
    /// 构造好的数组指令，其值为Intptr
    pub fn make_int_array(
        &mut self,
        length: i32,
        is_init: bool,
        init_array: Vec<(bool, i32)>,
    ) -> ObjPtr<Inst> {
        // 正确性检查
        // 数组长度必须为整数

        let mut inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::IntPtr,
            InstKind::Alloca(length),
            vec![],
        ));
        inst.set_int_init(is_init, init_array);

        // 设置use list
        inst
    }

    /// 申请一个float类型的数组
    ///
    /// # Arguments
    /// * 'length' - 要申请的数组的长度
    /// # Returns
    /// 构造好的数组指令，其值为Floatptr
    pub fn make_float_array(
        &mut self,
        length: i32,
        is_init: bool,
        init_array: Vec<(bool, f32)>,
    ) -> ObjPtr<Inst> {
        let mut inst = self.put(Inst::new(
            crate::ir::ir_type::IrType::FloatPtr,
            InstKind::Alloca(length),
            vec![],
        ));
        inst.set_float_init(is_init, init_array);

        // 设置use list
        inst
    }
}

impl Inst {
    /// 设置整型数组的初始值
    pub fn set_int_init(&mut self, is_init: bool, init: Vec<(bool, i32)>) {
        if let InstKind::Alloca(_) = self.get_kind() {
            if let IrType::IntPtr = self.get_ir_type() {
            } else {
                unreachable!("Inst::set_int_init")
            }
        } else {
            unreachable!("Inst::set_int_init")
        }

        // 设置use list
        self.init = ((is_init, init), (false, vec![]));
    }

    /// 设置浮点型数组的初始值,只允许在初始化的时候调用
    pub fn set_float_init(&mut self, is_init: bool, init: Vec<(bool, f32)>) {
        if let InstKind::Alloca(_) = self.get_kind() {
            if let IrType::FloatPtr = self.get_ir_type() {
            } else {
                unreachable!("Inst::set_float_init")
            }
        } else {
            unreachable!("Inst::set_float_init")
        }

        // 设置use list
        self.init = ((false, vec![]), (is_init, init));
    }

    /// 获得整型数组的初始值
    /// 第一个bool为false时, 如果当前数组长度为0，则是未初始化的
    /// 第二个bool为true时，如果当前i32值为0，那么这个地方其实是被一个变量初始化的
    pub fn get_int_init(&self) -> &(bool, Vec<(bool, i32)>) {
        // 正确性检查
        if let InstKind::Alloca(_) = self.get_kind() {
            debug_assert!(self.get_ir_type() == IrType::IntPtr);
        } else {
            unreachable!("Inst::get_int_init")
        }

        &self.init.0
    }

    /// 获得浮点型数组的初始值
    pub fn get_float_init(&self) -> &(bool, Vec<(bool, f32)>) {
        // 正确性检查
        if let InstKind::Alloca(_) = self.get_kind() {
            debug_assert!(self.get_ir_type() == IrType::FloatPtr);
        } else {
            unreachable!("Inst::get_float_init")
        }

        &self.init.1
    }

    /// 获得数组长度
    pub fn get_array_length(&self) -> i32 {
        // 正确性检查
        if let InstKind::Alloca(length) = self.get_kind() {
            debug_assert!(self.user.get_operands_size() == 0);
            length
        } else {
            unreachable!("Inst::get_array_length")
        }
    }

    /// 判断当前是否为数组
    /// # Return
    /// 如果是返回true，否则返回false
    pub fn is_array(&self) -> bool {
        self.get_kind() == InstKind::Alloca(0)
    }

    /// 判断当前是否为全局数组
    /// # Return
    /// 如果是返回true，否则返回false
    pub fn is_global_array(&self) -> bool {
        self.is_array() && self.parent_bb.is_none()
    }

    /// 判断当前是否为局部数组
    /// # Return
    /// 如果是返回true，否则返回false
    pub fn is_local_array(&self) -> bool {
        self.is_array() && self.parent_bb.is_some()
    }
}
