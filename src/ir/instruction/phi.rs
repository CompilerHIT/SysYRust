///! 本文件为phi指令的实现
use super::*;

impl Inst {
    /// 创建int类型的phi指令
    pub fn make_int_phi() -> Inst {
        Self::new(IrType::Int, InstKind::Phi, vec![])
    }

    /// 创建float类型的phi指令
    pub fn make_float_phi() -> Inst {
        Self::new(IrType::Float, InstKind::Phi, vec![])
    }

    /// 创建intptr类型的phi指令
    pub fn make_intptr_phi() -> Inst {
        Self::new(IrType::IntPtr, InstKind::Phi, vec![])
    }

    /// 创建floatptr类型的phi指令
    pub fn make_floatptr_phi() -> Inst {
        Self::new(IrType::FloatPtr, InstKind::Phi, vec![])
    }

    /// 向phi指令中添加一个操作数
    pub fn add_operand(&mut self, operand: &Inst) {
        self.user.push_operand(operand);
    }
}
