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
    SCEVGEPExpr,
    SCEVAddExpr,
    SCEVSubExpr,
    SCEVMulExpr,
    SCEVRecExpr,
    SCEVGEPRecExpr,
    SCEVAddRecExpr,
    SCEVSubRecExpr,
    SCEVMulRecExpr,
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
    pub fn get_lhs(&self) -> ObjPtr<Inst> {
        self.get_operands()[0]
    }
    pub fn get_rhs(&self) -> ObjPtr<Inst> {
        self.get_operands()[1]
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
    pub fn is_scev_gep_rec_expr(&self) -> bool {
        SCEVExpKind::SCEVGEPRecExpr == self.kind
    }
    pub fn is_scev_gep_expr(&self) -> bool {
        SCEVExpKind::SCEVGEPExpr == self.kind
    }
    pub fn set_operands(&mut self, operands: Vec<ObjPtr<Inst>>) {
        self.operands = operands;
    }
    pub fn get_scev_rec_start(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.kind, SCEVExpKind::SCEVRecExpr);
        self.get_operands()[0]
    }
    pub fn get_scev_rec_step(&self) -> ObjPtr<Inst> {
        debug_assert_eq!(self.kind, SCEVExpKind::SCEVRecExpr);
        self.get_operands()[1]
    }
    pub fn get_scev_rec_end_cond(&self, mut loop_info: ObjPtr<LoopInfo>) -> Vec<ObjPtr<Inst>> {
        debug_assert_eq!(self.get_kind(), SCEVExpKind::SCEVRecExpr);
        loop_info
            .get_exit_blocks()
            .iter()
            .map(|bb| bb.get_tail_inst().get_br_cond())
            .collect()
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

    pub fn make_scev_add_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVAddExpr,
            operands: bond_inst.get_operands().clone(),
            bond_inst,
        })
    }

    pub fn make_scev_sub_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVSubExpr,
            operands: bond_inst.get_operands().clone(),
            bond_inst,
        })
    }

    pub fn make_scev_mul_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVMulExpr,
            operands: bond_inst.get_operands().clone(),
            bond_inst,
        })
    }

    pub fn make_scev_mul_rec_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVMulRecExpr,
            operands: bond_inst.get_operands().clone(),
            bond_inst,
        })
    }

    pub fn make_scev_add_rec_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVAddRecExpr,
            operands: bond_inst.get_operands().clone(),
            bond_inst,
        })
    }

    pub fn make_scev_sub_rec_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVSubRecExpr,
            operands: bond_inst.get_operands().clone(),
            bond_inst,
        })
    }

    pub fn make_scev_gep_rec_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        let mut operands = bond_inst.get_operands().clone();
        operands.pop();
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVGEPRecExpr,
            operands,
            bond_inst,
        })
    }

    pub fn make_scev_gep_expr(&mut self, bond_inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        let mut operands = bond_inst.get_operands().clone();
        operands.pop();
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVGEPExpr,
            operands,
            bond_inst,
        })
    }

    pub fn make_scev_rec_expr(
        &mut self,
        bond_inst: ObjPtr<Inst>,
        start: ObjPtr<Inst>,
        step: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        let mut operands = bond_inst.get_operands().clone();
        operands.pop();
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVRecExpr,
            operands: vec![start, step],
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
