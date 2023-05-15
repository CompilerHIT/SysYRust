use super::*;

///! 此文件为二元运算指令的实现

impl Inst {
    /// 创建加法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_add(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Add),
            vec![lhs, rhs],
        )
    }

    /// 创建减法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_sub(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Sub),
            vec![lhs, rhs],
        )
    }

    /// 创建乘法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_mul(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Mul),
            vec![lhs, rhs],
        )
    }

    /// 创建除法指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_div(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Div),
            vec![lhs, rhs],
        )
    }

    /// 创建求余指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_rem(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Rem),
            vec![lhs, rhs],
        )
    }

    /// 创建逻辑与指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_and(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::And),
            vec![lhs, rhs],
        )
    }

    /// 创建逻辑或指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_or(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Or),
            vec![lhs, rhs],
        )
    }

    /// 创建小于等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_le(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Le),
            vec![lhs, rhs],
        )
    }

    /// 创建小于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_lt(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Lt),
            vec![lhs, rhs],
        )
    }

    /// 创建大于等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_ge(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Ge),
            vec![lhs, rhs],
        )
    }

    /// 创建大于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_gt(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Gt),
            vec![lhs, rhs],
        )
    }

    /// 创建等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_eq(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Eq),
            vec![lhs, rhs],
        )
    }

    /// 创建不等于指令
    /// # Arguments
    /// * `lhs` - 左操作数
    /// * `rhs` - 右操作数
    pub fn make_ne(lhs: &Inst, rhs: &Inst) -> Self {
        Self::new(
            lhs.get_ir_type(),
            InstKind::Binary(BinOp::Ne),
            vec![lhs, rhs],
        )
    }

    /// 获得左操作数
    pub fn get_lhs(&self) -> &Inst {
        self.user.get_operand(0)
    }

    /// 获得右操作数
    pub fn get_rhs(&self) -> &Inst {
        self.user.get_operand(1)
    }

    /// 修改左操作数
    pub fn set_lhs(&mut self, lhs: &Inst) {
        self.user.set_operand(0, lhs);
    }

    /// 修改右操作数
    pub fn set_rhs(&mut self, rhs: &Inst) {
        self.user.set_operand(1, rhs);
    }
}
