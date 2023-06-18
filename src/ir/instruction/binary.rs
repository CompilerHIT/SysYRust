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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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

    /// 创建小于等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_le(&mut self, mut lhs: ObjPtr<Inst>, mut rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
        // 正确性检查
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
        debug_assert_eq!(lhs.get_ir_type(), rhs.get_ir_type());

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
            debug_assert_eq!(lhs.get_ir_type(), self.get_lhs().get_ir_type());
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
            debug_assert_eq!(self.get_rhs().get_ir_type(), rhs.get_ir_type());
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
