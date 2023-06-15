///! 本文件为分支指令的实现
use super::*;

impl ObjPool<Inst> {
    /// 创建条件跳转指令
    /// # Arguments
    /// * `cond` - 条件
    /// # Returns
    /// 返回创建的条件跳转指令
    pub fn make_br(&mut self, mut cond: ObjPtr<Inst>) -> ObjPtr<Inst> {
        let ir_type = IrType::Void;
        let kind = InstKind::Branch;
        let operands = vec![cond];
        let inst = Inst::new(ir_type, kind, operands);
        let inst = self.put(inst);

        // 设置use_list
        cond.add_user(inst.as_ref());

        inst
    }

    /// 创建无条件跳转指令
    /// # Returns
    /// 返回创建的无条件跳转指令
    pub fn make_jmp(&mut self) -> ObjPtr<Inst> {
        let ir_type = IrType::Void;
        let kind = InstKind::Branch;
        let operands = vec![];
        self.put(Inst::new(ir_type, kind, operands))
    }
}

impl Inst {
    /// 判断是否为条件跳转指令
    pub fn is_br(&self) -> bool {
        // 正确性检查
        if let InstKind::Branch = self.kind {
            debug_assert!(self.user.get_operands_size() <= 1);
        } else {
            unreachable!("Inst::is_br")
        };

        self.user.get_operands_size() == 1
    }

    /// 判断是否为无条件跳转指令
    pub fn is_jmp(&self) -> bool {
        // 正确性检查
        if let InstKind::Branch = self.kind {
            debug_assert!(self.user.get_operands_size() <= 1);
        } else {
            unreachable!("Inst::is_jmp")
        };

        self.user.get_operands_size() == 0
    }

    /// 获得条件跳转指令的条件
    pub fn get_br_cond(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::Branch = self.kind {
            debug_assert!(self.is_br());
        } else {
            unreachable!("Inst::get_br_cond")
        };

        self.user.get_operand(0)
    }

    /// 设置条件跳转指令的条件
    /// # Arguments
    /// * `cond` - 条件
    pub fn set_br_cond(&mut self, mut cond: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Branch = self.kind {
            debug_assert!(self.is_br());
            debug_assert!(
                cond.as_ref().get_ir_type() == IrType::Int
                    || cond.as_ref().get_ir_type() == IrType::ConstInt
            );
        } else {
            unreachable!("Inst::set_br_cond")
        };

        // 修改use_list
        self.user.get_operand(0).remove_user(self);
        cond.add_user(self);

        self.user.set_operand(0, cond);
    }

    /// 获得条件为true时的bb
    pub fn get_true_bb(&self) -> ObjPtr<BasicBlock> {
        debug_assert!(self.is_br());
        self.get_parent_bb().get_next_bb()[1].clone()
    }

    /// 获得条件为false时的bb
    pub fn get_false_bb(&self) -> ObjPtr<BasicBlock> {
        debug_assert!(self.is_br());
        self.get_parent_bb().get_next_bb()[0].clone()
    }
}
