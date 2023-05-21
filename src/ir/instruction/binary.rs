use super::*;

///! 此文件为二元运算指令的实现

impl ObjPool<Inst> {
    /// 创建加法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_add(&mut self, lhs: ObjPtr<Inst>, rhs: ObjPtr<Inst>) -> ObjPtr<Inst> {
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
        self.user.get_operand(0)
    }

    /// 获得右操作数
    pub fn get_rhs(&self) -> ObjPtr<Inst> {
        self.user.get_operand(1)
    }

    /// 修改左操作数
    pub fn set_lhs(&mut self, lhs: ObjPtr<Inst>) {
        // 修改use list
        let old_lhs = self.user.get_operand(0);
        old_lhs.as_mut().remove_user(self);
        lhs.as_mut().add_user(self);

        self.user.set_operand(0, lhs);
    }

    /// 修改右操作数
    pub fn set_rhs(&mut self, rhs: ObjPtr<Inst>) {
        // 修改use list
        let old_rhs = self.user.get_operand(1);
        old_rhs.as_mut().remove_user(self);
        rhs.as_mut().add_user(self);

        self.user.set_operand(1, rhs);
    }
}
