///! 本文件为phi指令的实现
use super::*;
impl ObjPool<Inst> {
    /// 创建指定类型的phi
    pub fn make_phi(&mut self, irtype: IrType) -> ObjPtr<Inst> {
        self.put(Inst::new(irtype, InstKind::Phi, vec![]))
    }

    /// 创建int类型的phi指令
    pub fn make_int_phi(&mut self) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::Int, InstKind::Phi, vec![]))
    }

    /// 创建float类型的phi指令
    pub fn make_float_phi(&mut self) -> ObjPtr<Inst> {
        self.put(Inst::new(IrType::Float, InstKind::Phi, vec![]))
    }

    /// 创建指定类型的phi指令
    pub fn make_phi_with_operands(
        &mut self,
        irtype: IrType,
        operands: Vec<ObjPtr<Inst>>,
    ) -> ObjPtr<Inst> {
        let inst = self.put(Inst::new(irtype, InstKind::Phi, operands));
        for op in inst.get_operands() {
            op.as_mut().add_user(inst.as_ref());
        }
        inst
    }
}
impl Inst {
    /// 判断是否为phi指令
    pub fn is_phi(&self) -> bool {
        self.get_kind() == InstKind::Phi
    }

    /// 向phi指令中添加一个操作数
    pub fn add_operand(&mut self, mut operand: ObjPtr<Inst>) {
        // 正确性检查
        debug_assert_eq!(self.get_ir_type(), operand.get_ir_type());

        self.user.push_operand(operand);

        // 更新操作数的使用者
        operand.add_user(self)
    }

    /// 获得phi指令的操作数列表
    pub fn get_operands(&self) -> &Vec<ObjPtr<Inst>> {
        self.user.get_operands()
    }

    /// 获得指定下标的操作数
    pub fn get_operand(&self, index: usize) -> ObjPtr<Inst> {
        self.user.get_operand(index)
    }

    /// 设置phi指令的操作数
    /// # Arguments
    /// * 'operand' - 操作数
    /// * 'index' - 操作数的索引
    pub fn set_operand(&mut self, mut operand: ObjPtr<Inst>, index: usize) {
        // 正确性检查
        debug_assert_eq!(
            self.get_operands()[index].get_ir_type(),
            operand.get_ir_type()
        );

        // 修改use list
        self.user.get_operand(index).remove_user(self);
        operand.add_user(self);

        // 更新操作数的使用者
        self.user.set_operand(index, operand);
    }

    /// 获得操作数的索引，如果有重复的，会给出第一个
    pub fn get_operand_index(&self, operand: ObjPtr<Inst>) -> usize {
        self.get_operands()
            .iter()
            .position(|inst| inst.clone() == operand)
            .unwrap()
    }

    /// 这个函数非常危险，因为其将原来的operends设置为新的operands,但并没有修改use_list
    /// # Arguments
    /// * 'operands' - 操作数列表
    pub fn set_operands(&mut self, operands: Vec<ObjPtr<Inst>>) {
        self.user.set_operands(operands);
    }

    /// 这个函数非常危险，因为其将原来的operends设置为新的operands,但并没有修改use_list
    /// # Arguments
    /// * 'users' - 使用者列表
    pub fn set_users(&mut self, users: Vec<ObjPtr<Inst>>) {
        self.user.set_users(users);
    }

    /// 获得phi指令的操作数对应的前继基本快
    /// 使用时确保该操作数是phi指令的操作数
    pub fn get_phi_predecessor(&self, index: usize) -> ObjPtr<BasicBlock> {
        self.get_parent_bb().get_up_bb()[index]
    }

    /// 删除一个操作数
    /// # Arguments
    /// * 'operand' - 被删除的操作数
    pub fn remove_operand(&mut self, index: usize) {
        self.user.get_operand(index).remove_user(self);
        self.user.remove_operand(index);
    }

    pub fn remove_operand_by_index(&mut self, index: usize) {
        self.user.get_operand(index).remove_user(self);
        self.user.remove_operand(index);
    }
}
