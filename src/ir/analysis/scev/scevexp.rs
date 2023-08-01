use std::fmt::Debug;

use crate::ir::analysis::loop_tree::LoopInfo;

use super::*;

pub struct SCEVExp {
    kind: SCEVExpKind,
    operands: Vec<ObjPtr<SCEVExp>>,
    scev_const: i32,
    bond_inst: Option<ObjPtr<Inst>>,
    in_loop: Option<ObjPtr<LoopInfo>>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SCEVExpKind {
    SCEVConstant,
    SCEVUnknown,
    SCEVAddExpr,
    SCEVSubExpr,
    SCEVMulExpr,
    SCEVRecExpr,
    SCEVAddRecExpr,
    SCEVSubRecExpr,
    SCEVMulRecExpr,
}

impl SCEVExp {
    pub fn new(
        kind: SCEVExpKind,
        operands: Vec<ObjPtr<SCEVExp>>,
        scev_const: i32,
        bond_inst: Option<ObjPtr<Inst>>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> Self {
        SCEVExp {
            kind,
            operands,
            scev_const,
            bond_inst,
            in_loop,
        }
    }

    pub fn get_scev_const(&self) -> i32 {
        self.scev_const
    }

    pub fn get_in_loop(&self) -> Option<ObjPtr<LoopInfo>> {
        self.in_loop
    }
    pub fn get_kind(&self) -> SCEVExpKind {
        self.kind
    }
    pub fn get_operands(&self) -> Vec<ObjPtr<SCEVExp>> {
        self.operands.clone()
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
    pub fn is_scev_mul_rec_expr(&self) -> bool {
        SCEVExpKind::SCEVMulRecExpr == self.kind
    }
    pub fn is_scev_rec_expr(&self) -> bool {
        SCEVExpKind::SCEVRecExpr == self.kind
    }
    pub fn is_scev_rec(&self) -> bool {
        match self.kind {
            SCEVExpKind::SCEVMulRecExpr
            | SCEVExpKind::SCEVSubRecExpr
            | SCEVExpKind::SCEVAddRecExpr
            | SCEVExpKind::SCEVRecExpr => true,
            _ => false,
        }
    }
    pub fn set_operands(&mut self, operands: Vec<ObjPtr<SCEVExp>>) {
        self.operands = operands;
    }
    pub fn get_bond_inst(&self) -> ObjPtr<Inst> {
        debug_assert!(self.is_scev_rec_expr() || self.is_scev_unknown());
        self.bond_inst.unwrap()
    }
}

impl ObjPool<SCEVExp> {
    pub fn make_scev_constant(&mut self, scev_const: i32) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVConstant,
            vec![],
            scev_const,
            None,
            None,
        ))
    }

    pub fn make_scev_unknown(
        &mut self,
        bond_inst: Option<ObjPtr<Inst>>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVUnknown,
            vec![],
            0,
            bond_inst,
            in_loop,
        ))
    }

    pub fn make_scev_add_expr(
        &mut self,
        lhs: ObjPtr<SCEVExp>,
        rhs: ObjPtr<SCEVExp>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVAddExpr,
            vec![lhs, rhs],
            0,
            None,
            in_loop,
        ))
    }

    pub fn make_scev_sub_expr(
        &mut self,
        lhs: ObjPtr<SCEVExp>,
        rhs: ObjPtr<SCEVExp>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVSubExpr,
            vec![lhs, rhs],
            0,
            None,
            in_loop,
        ))
    }

    pub fn make_scev_mul_expr(
        &mut self,
        lhs: ObjPtr<SCEVExp>,
        rhs: ObjPtr<SCEVExp>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVMulExpr,
            vec![lhs, rhs],
            0,
            None,
            in_loop,
        ))
    }

    pub fn make_scev_mul_rec_expr(
        &mut self,
        operands: Vec<ObjPtr<SCEVExp>>,
        bond_inst: Option<ObjPtr<Inst>>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVMulRecExpr,
            operands,
            0,
            bond_inst,
            in_loop,
        ))
    }

    pub fn make_scev_add_rec_expr(
        &mut self,
        operands: Vec<ObjPtr<SCEVExp>>,
        bond_inst: Option<ObjPtr<Inst>>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVAddRecExpr,
            operands,
            0,
            bond_inst,
            in_loop,
        ))
    }

    pub fn make_scev_sub_rec_expr(
        &mut self,
        operands: Vec<ObjPtr<SCEVExp>>,
        bond_inst: Option<ObjPtr<Inst>>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVSubRecExpr,
            operands,
            0,
            bond_inst,
            in_loop,
        ))
    }

    pub fn make_scev_rec_expr(
        &mut self,
        operands: Vec<ObjPtr<SCEVExp>>,
        bond_inst: Option<ObjPtr<Inst>>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp::new(
            SCEVExpKind::SCEVRecExpr,
            operands,
            0,
            bond_inst,
            in_loop,
        ))
    }
}

impl Debug for SCEVExp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        s += &format!("{:?}: ", self.kind);
        for operand in self.operands.iter() {
            match operand.kind {
                SCEVExpKind::SCEVConstant => {
                    s += &format!("{}, ", operand.scev_const);
                }
                SCEVExpKind::SCEVUnknown => {
                    s += &format!("{:?}, ", operand.bond_inst);
                }
                _ => {
                    s += &format!("{:?}, ", operand.kind);
                }
            }
        }
        write!(f, "{}", s)
    }
}
