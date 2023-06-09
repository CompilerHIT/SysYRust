use super::*;

///! 此文件为二元运算指令的实现

impl ObjPool<Inst> {
    /// 创建加法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_add(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        // 操作数类型要相互对应
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Add),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建减法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_sub(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Sub),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }
    /// 创建乘法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_mul(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Mul),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建除法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_div(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Div),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建求余指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_rem(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Rem),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建逻辑与指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_and(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, false);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::And),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建逻辑或指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_or(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, false);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Or),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建小于等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_le(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Le),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建小于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_lt(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Lt),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建大于等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_ge(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Ge),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建大于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_gt(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Gt),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }
    /// 创建等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_eq(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Eq),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
        inst
    }

    /// 创建不等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_ne(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        check_arith(lhs, rhs, true);

        let inst = self.put(Inst::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Ne),
            vec![lhs, rhs],
        ));

        // 设置use list
        lhs.add_user(inst.as_ref());
        rhs.add_user(inst.as_ref());
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
    pub fn set_lhs(&mut self, mut lhs: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Binary(_) = self.kind {
            check_arith(lhs, self.get_rhs(), is_arith(self));
        } else {
            unreachable!("Inst::set_lhs")
        }

        // 修改use list
        let mut old_lhs = self.user.get_operand(0);
        old_lhs.remove_user(self);
        lhs.add_user(self);

        self.user.set_operand(0, lhs);
    }

    /// 修改右操作数
    pub fn set_rhs(&mut self, mut rhs: ObjPtr<Inst>) {
        // 正确性检查
        if let InstKind::Binary(_) = self.kind {
            check_arith(self.get_lhs(), rhs, is_arith(self));
        } else {
            unreachable!("Inst::set_rhs")
        }

        // 修改use list
        let mut old_rhs = self.user.get_operand(1);
        old_rhs.remove_user(self);
        rhs.add_user(self);

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
    match lhs.get_ir_type() {
        IrType::Int | IrType::ConstInt => {
            debug_assert!(rhs.get_ir_type() == IrType::Int || rhs.get_ir_type() == IrType::ConstInt)
        }
        IrType::Float | IrType::ConstFloat if check_float => debug_assert!(
            rhs.get_ir_type() == IrType::Float || rhs.get_ir_type() == IrType::ConstFloat
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
