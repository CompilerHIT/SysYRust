///! 此文件为常量和全局变量指令的实现，包括整型和浮点型、局部和全局变量。
use super::*;
use crate::utility::ObjPool;

impl ObjPool<Inst> {
    /// 创建一个整型常量指令
    /// # Arguments
    /// * 'bonding' - 常量的值
    pub fn make_int_const(&mut self, bonding: i32) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::Int, InstKind::ConstInt(bonding), vec![]))
    }

    /// 创建一个浮点型常量指令
    /// # Arguments
    /// * 'bonding' - 常量的值
    pub fn make_float_const(&mut self, bonding: f32) -> ObjPtr<Inst> {
        self.put(Inst::new(
            IrType::Float,
            InstKind::ConstFloat(bonding),
            vec![],
        ))
    }

    /// 创建一个全局整型常量指令
    /// # Arguments
    /// * 'bonding' - 常量的值
    pub fn make_global_int_const(&mut self, bonding: i32) -> ObjPtr<Inst> {
        self.put(Inst::new(
            IrType::Int,
            InstKind::GlobalConstInt(bonding),
            vec![],
        ))
    }

    /// 创建一个全局浮点型常量指令
    /// # Arguments
    /// * 'bonding' - 常量的值
    pub fn make_global_float_const(&mut self, bonding: f32) -> ObjPtr<Inst> {
        self.put(Inst::new(
            IrType::Float,
            InstKind::GlobalConstFloat(bonding),
            vec![],
        ))
    }

    /// 创建一个全局整型变量指令
    /// # Arguments
    /// * 'bonding' - 初始值
    pub fn make_global_int(&mut self, bonding: i32) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::Int, InstKind::GlobalInt(bonding), vec![]))
    }

    /// 创建一个全局浮点型变量指令
    /// # Arguments
    /// * 'bonding' - 初始值
    pub fn make_global_float(&mut self, bonding: f32) -> ObjPtr<Inst> {
        self.put(Inst::new(
            IrType::Float,
            InstKind::GlobalFloat(bonding),
            vec![],
        ))
    }
}

impl Inst {
    /// 获得Int类型绑定的值
    pub fn get_int_bond(&self) -> i32 {
        match self.get_kind() {
            InstKind::GlobalInt(i) => i,
            InstKind::ConstInt(i) => i,
            InstKind::GlobalConstInt(i) => i,
            _ => unreachable!("Inst::get_int_bond: not a int type"),
        }
    }

    /// 获得Float类型绑定的值
    pub fn get_float_bond(&self) -> f32 {
        match self.get_kind() {
            InstKind::GlobalFloat(f) => f,
            InstKind::ConstFloat(f) => f,
            InstKind::GlobalConstFloat(f) => f,
            _ => unreachable!("Inst::get_float_bond: not a float type"),
        }
    }
}