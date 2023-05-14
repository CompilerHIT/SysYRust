use crate::cfgir::{instruction_cfg::*, ir_type_cfg::CfgIrType, user_cfg::CfgUser};
use crate::utility::Pointer;
//todo:是否需要把指针区域删去
pub enum CfgOperator {
    Add,
    Sub,
    Mul,
    Div,
}

pub struct CfgBinaryOpInst {
    user: CfgUser,
    operator: CfgOperator,
    namea:String,
    nameb:String,
    namec:String,
}

impl CfgBinaryOpInst {
    fn make_binary_op_inst(
        ir_type: CfgIrType,
        operator: CfgOperator,
        lhs: Pointer<Box<dyn CfgInstruction>>,
        rhs: Pointer<Box<dyn CfgInstruction>>,
        namea:String,
        nameb:String,
        namec:String,
    ) -> Pointer<Box<dyn CfgInstruction>> {
        let user = CfgUser::make_user(ir_type, vec![lhs, rhs]);
        let inst = CfgBinaryOpInst { user, operator ,namea,nameb,namec};
        Pointer::new(Box::new(inst))
    }

    /// 构造一个加指令
    pub fn make_add_inst(
        lhs: Pointer<Box<dyn CfgInstruction>>,
        rhs: Pointer<Box<dyn CfgInstruction>>,
        namea:String,
        nameb:String,
        namec:String,
    ) -> Pointer<Box<dyn CfgInstruction>> {
        Self::make_binary_op_inst(CfgIrType::Int, CfgOperator::Add, lhs, rhs,namea,nameb,namec)
    }

    /// 构造一个减指令
    pub fn make_sub_inst(
        lhs: Pointer<Box<dyn CfgInstruction>>,
        rhs: Pointer<Box<dyn CfgInstruction>>,
        namea:String,
        nameb:String,
        namec:String,
    ) -> Pointer<Box<dyn CfgInstruction>> {
        Self::make_binary_op_inst(CfgIrType::Int, CfgOperator::Sub, lhs, rhs,namea,nameb,namec)
    }

    /// 构造一个乘指令
    pub fn make_mul_inst(
        lhs: Pointer<Box<dyn CfgInstruction>>,
        rhs: Pointer<Box<dyn CfgInstruction>>,
        namea:String,
        nameb:String,
        namec:String,
    ) -> Pointer<Box<dyn CfgInstruction>> {
        Self::make_binary_op_inst(CfgIrType::Int, CfgOperator::Mul, lhs, rhs,namea,nameb,namec)
    }

    /// 构造一个除指令
    pub fn make_div_inst(
        lhs: Pointer<Box<dyn CfgInstruction>>,
        rhs: Pointer<Box<dyn CfgInstruction>>,
        namea:String,
        nameb:String,
        namec:String,
    ) -> Pointer<Box<dyn CfgInstruction>> {
        Self::make_binary_op_inst(CfgIrType::Int, CfgOperator::Div, lhs, rhs,namea,nameb,namec)
    }

    // 获得操作符
    pub fn get_operator(&self) -> &CfgOperator {
        &self.operator
    }

    // 获得左操作数
    // # Panics
    // 左操作数不存在，是空指针
    pub fn get_lhs(&self) -> Pointer<Box<dyn CfgInstruction>> {
        self.user.get_operand(0)
    }

    // 获得右操作数
    //
    // # Panics
    // 右操作数不存在，是空指针
    pub fn get_rhs(&self) -> Pointer<Box<dyn CfgInstruction>> {
        self.user.get_operand(1)
    }
}

impl CfgInstruction for CfgBinaryOpInst {
    fn get_type(&self) -> CfgInstructionType {
        CfgInstructionType::IBinaryOpInst
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
