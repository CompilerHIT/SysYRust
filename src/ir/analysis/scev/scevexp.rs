use std::fmt::Debug;

use crate::ir::analysis::loop_tree::LoopInfo;

use super::*;

pub struct SCEVExp {
    kind: SCEVExpKind,
    operands: Vec<ObjPtr<Inst>>,
    bond_inst: ObjPtr<Inst>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SCEVExpKind {
    SCEVConstant,
    SCEVUnknown,
    SCEVAddExpr,
    SCEVSubExpr,
    SCEVMulExpr,
    SCEVAddRecExpr,
    SCEVSubRecExpr,
}

impl SCEVExp {
    pub fn get_kind(&self) -> SCEVExpKind {
        self.kind
    }
    pub fn get_operands(&self) -> Vec<ObjPtr<Inst>> {
        self.operands.clone()
    }
    pub fn get_bond_inst(&self) -> ObjPtr<Inst> {
        self.bond_inst
    }
    pub fn is_scev_constant(&self) -> bool {
        self.kind == SCEVExpKind::SCEVConstant
    }
    pub fn is_scev_unknown(&self) -> bool {
        self.kind == SCEVExpKind::SCEVUnknown
    }
    pub fn is_scev_add_expr(&self) -> bool {
        self.kind == SCEVExpKind::SCEVAddExpr
    }
    pub fn is_scev_mul_expr(&self) -> bool {
        self.kind == SCEVExpKind::SCEVMulExpr
    }
    pub fn is_scev_add_rec_expr(&self) -> bool {
        SCEVExpKind::SCEVAddRecExpr == self.kind
    }
    pub fn is_scev_sub_rec_expr(&self) -> bool {
        SCEVExpKind::SCEVSubRecExpr == self.kind
    }
    pub fn is_scev_sub_expr(&self) -> bool {
        SCEVExpKind::SCEVSubExpr == self.kind
    }
    pub fn set_operands(&mut self, operands: Vec<ObjPtr<Inst>>) {
        self.operands = operands;
    }

    /// 获得SCEVAddRecExpr的start值
    pub fn get_add_rec_start(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.get_kind(), SCEVExpKind::SCEVAddRecExpr);
        self.get_operands()[0]
    }

    /// 获得SCEVAddRecExpr的step值
    pub fn get_add_rec_step(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.get_kind(), SCEVExpKind::SCEVAddRecExpr);
        self.get_operands()[1]
    }

    /// 获得SCEVAddRecExpr的end条件
    /// 动态计算
    pub fn get_add_rec_end_cond(&self, mut loop_info: ObjPtr<LoopInfo>) -> Vec<ObjPtr<Inst>> {
        debug_assert_eq!(self.get_kind(), SCEVExpKind::SCEVAddRecExpr);
        let exit_block = loop_info.get_exit_blocks();
        loop_info
            .get_exit_blocks()
            .iter()
            .map(|bb| bb.get_tail_inst().get_br_cond())
            .collect()
    }

    /// 获得SCEVSubRecExpr的start值
    pub fn get_sub_rec_start(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.get_kind(), SCEVExpKind::SCEVSubRecExpr);
        self.get_operands()[0]
    }

    /// 获得SCEVSubRecExpr的step值，注意，此处给出的是被减数，所以实际上的step应该取相反数
    pub fn get_sub_rec_step(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.get_kind(), SCEVExpKind::SCEVSubRecExpr);
        self.get_operands()[1]
    }

    /// 获得SCEVSubRecExpr的end值
    pub fn get_sub_rec_end(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.get_kind(), SCEVExpKind::SCEVSubRecExpr);
        self.get_operands()[2]
    }
}

impl ObjPool<SCEVExp> {
    pub fn make_scev_constant(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVConstant,
            operands: vec![],
            bond_inst,
        })
    }

    pub fn make_scev_unknown(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVUnknown,
            operands: vec![],
            bond_inst,
        })
    }

    pub fn make_scev_add_expr(
        &mut self,
        operands: Vec<ObjPtr<Inst>>,
        bond_inst: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVAddExpr,
            operands,
            bond_inst,
        })
    }

    pub fn make_scev_sub_expr(
        &mut self,
        operands: Vec<ObjPtr<Inst>>,
        bond_inst: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVSubExpr,
            operands,
            bond_inst,
        })
    }

    pub fn make_scev_mul_expr(
        &mut self,
        operands: Vec<ObjPtr<Inst>>,
        bond_inst: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVMulExpr,
            operands,
            bond_inst,
        })
    }

    pub fn make_scev_add_rec_expr(
        &mut self,
        operands: Vec<ObjPtr<Inst>>,
        bond_inst: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVAddRecExpr,
            operands,
            bond_inst,
        })
    }

    pub fn make_scev_sub_rec_expr(
        &mut self,
        operands: Vec<ObjPtr<Inst>>,
        bond_inst: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVSubRecExpr,
            operands,
            bond_inst,
        })
    }
}

impl Debug for SCEVExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s += &format!("{:?}: ", self.kind);
        for operand in self.operands.iter() {
            s += &format!("{:?}, ", operand);
        }
        write!(f, "{}", s)
    }
}
