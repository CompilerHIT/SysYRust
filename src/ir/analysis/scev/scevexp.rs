use std::fmt::Debug;

use super::*;

pub struct SCEVExp {
    kind: SCEVExpKind,
    operands: Vec<ObjPtr<SCEVExp>>,
    bond_inst: ObjPtr<Inst>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SCEVExpKind {
    SCEVConstant,
    SCEVUnknown,
    SCEVAddExpr,
    SCEVMulExpr,
    SCEVAddRecExpr,
}

impl SCEVExp {
    pub fn get_kind(&self) -> SCEVExpKind {
        self.kind
    }
    pub fn get_operands(&self) -> &Vec<ObjPtr<SCEVExp>> {
        &self.operands
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
        self.kind == SCEVExpKind::SCEVAddRecExpr
    }
    pub fn set_operands(&mut self, operands: Vec<ObjPtr<SCEVExp>>) {
        self.operands = operands;
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
        operands: Vec<ObjPtr<SCEVExp>>,
        bond_inst: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVAddExpr,
            operands,
            bond_inst,
        })
    }

    pub fn make_scev_mul_expr(
        &mut self,
        operands: Vec<ObjPtr<SCEVExp>>,
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
        operands: Vec<ObjPtr<SCEVExp>>,
        bond_inst: ObjPtr<Inst>,
    ) -> ObjPtr<SCEVExp> {
        self.put(SCEVExp {
            kind: SCEVExpKind::SCEVAddRecExpr,
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
