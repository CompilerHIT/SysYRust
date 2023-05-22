use super::*;

///! 此文件为二元运算指令的实现

impl ObjPool<Inst> {
    /// 创建加法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_add(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        // 操作数类型要相互对应
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Add),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建减法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_sub(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Sub),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }
    /// 创建乘法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_mul(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Mul),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建除法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_div(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Div),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建求余指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_rem(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Rem),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建逻辑与指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_and(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, false);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::And),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建逻辑或指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_or(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, false);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Or),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建小于等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_le(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Le),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建小于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_lt(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Lt),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建大于等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_ge(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Ge),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建大于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_gt(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Gt),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }
    /// 创建等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_eq(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Eq),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }

    /// 创建不等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_ne(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.as_ref().get_ir_type(),
            InstKind::Binary(BinOp::Ne),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.as_mut().add_user(inst.as_ref());
        rhs.as_mut().add_user(inst.as_ref());
        inst
    }
}

impl Inst {
    /// 获得左操作数
    pub fn get_lhs(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::Binary(_) = self.kind {
        } else {
            unreachable!("Inst::get_lhs")
        }

        self.user.get_operand(0)
    }

    /// 获得右操作数
    pub fn get_rhs(&self) -> ObjPtr<Inst> {
        // 正确性检查
        if let InstKind::Binary(_) = self.kind {
        } else {
            unreachable!("Inst::get_rhs")
        }

        self.user.get_operand(1)
    }

    /// 修改左操作数
    pub fn set_lhs(&mut self, lhs: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Binary(_) = self.kind {
            check_arith(lhs, self.get_rhs(), is_arith(self));
        } else {
            unreachable!("Inst::set_lhs")
        }

        // 修改use list
        let old_lhs = self.user.get_operand(0);
        old_lhs.as_mut().remove_user(self);
        lhs.as_mut().add_user(self);

        self.user.set_operand(0, lhs);
    }

    /// 修改右操作数
    pub fn set_rhs(&mut self, rhs: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Binary(_) = self.kind {
            check_arith(self.get_lhs(), rhs, is_arith(self));
        } else {
            unreachable!("Inst::set_rhs")
        }

        // 修改use list
        let old_rhs = self.user.get_operand(1);
        old_rhs.as_mut().remove_user(self);
        rhs.as_mut().add_user(self);

        self.user.set_operand(1, rhs);
    }
}

/// 四则运算指令正确性检查
/// # Arguments
/// * `lhs` - 左操作数
/// * `rhs` - 右操作数
/// * `check_float` - 是否检查浮点数
/// 操作数类型要相互对应
/// (Const)Int op (Const)Int
/// (Const)Float op (Const)Float
fn check_arith(lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>, check_float: bool) {
    match lhs.as_ref().get_ir_type() {
        IrType::Int | IrType::ConstInt => debug_assert!(
            rhs.as_ref().get_ir_type() == IrType::Int
                || rhs.as_ref().get_ir_type() == IrType::ConstInt
        ),
        IrType::Float | IrType::ConstFloat if check_float => debug_assert!(
            rhs.as_ref().get_ir_type() == IrType::Float
                || rhs.as_ref().get_ir_type() == IrType::ConstFloat
        ),
        _ => unreachable!("check_arith"),
    }
}

/// 检测是否为算术指令
/// # Arguments
/// * `inst` - 待检测指令
/// # Return
/// * `true` - 是算术指令
fn is_arith(inst: &Inst) -> bool {
    if let InstKind::Binary(op) = inst.get_kind() {
        match op {
            BinOp::And | BinOp::Or => false,
            _ => true,
        }
    } else {
        unreachable!("is_arith")
    }
}
