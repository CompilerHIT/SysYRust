use std::collections::HashMap;

use crate::{
    ir::instruction::{BinOp, Inst, InstKind},
    utility::{ObjPool, ObjPtr},
};

use self::scevexp::SCEVExp;

use super::dominator_tree::DominatorTree;

pub mod scevexp;

pub struct SCEVAnalyzer {
    scevexp_pool: ObjPool<SCEVExp>,
    map: HashMap<ObjPtr<Inst>, ObjPtr<SCEVExp>>,
    dominator_tree: Option<ObjPtr<DominatorTree>>,
}

impl SCEVAnalyzer {
    pub fn new() -> Self {
        Self {
            scevexp_pool: ObjPool::new(),
            map: HashMap::new(),
            dominator_tree: None,
        }
    }

    pub fn set_dominator_tree(&mut self, dominator_tree: ObjPtr<DominatorTree>) {
        self.dominator_tree = Some(dominator_tree);
    }

    pub fn clear(&mut self) {
        self.scevexp_pool.free_all();
        self.map.clear();
    }

    pub fn analyze(&mut self, inst: &ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if let Some(scev) = self.map.get(inst) {
            return *scev;
        }

        let scev;

        match inst.get_kind() {
            InstKind::Binary(BinOp::Add) => {
                scev = self.analyze_binary_add(inst);
                self.map.insert(*inst, scev);
            }
            InstKind::Binary(BinOp::Sub) => {
                scev = self.analyze_binary_sub(inst);
            }
            InstKind::Binary(BinOp::Mul) => {
                scev = self.analyze_binary_mul(inst);
                self.map.insert(*inst, scev);
            }
            InstKind::Gep => {
                scev = self.analyze_gep(inst);
                self.map.insert(*inst, scev);
            }
            InstKind::ConstInt(_)
            | InstKind::ConstFloat(_)
            | InstKind::GlobalConstInt(_)
            | InstKind::GlobalConstFloat(_) => {
                scev = self.scevexp_pool.make_scev_constant(*inst);
                self.map.insert(*inst, scev);
            }
            InstKind::Phi => {
                scev = self.analyze_phi(inst);
                self.map.insert(*inst, scev);
            }
            _ => {
                scev = self.scevexp_pool.make_scev_unknown(*inst);
                self.map.insert(*inst, scev);
            }
        }

        scev
    }

    fn check_rec(inst: &ObjPtr<Inst>, analyzer: &mut Self) -> bool {
        inst.get_operands()
            .iter()
            .any(|op| op.is_phi() && analyzer.analyze(op).is_scev_rec_expr())
    }

    fn analyze_binary_add(&mut self, inst: &ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if Self::check_rec(inst, self) {
            self.scevexp_pool.make_scev_add_rec_expr(*inst)
        } else {
            self.scevexp_pool.make_scev_add_expr(*inst)
        }
    }

    fn analyze_binary_sub(&mut self, inst: &ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if Self::check_rec(inst, self) {
            self.scevexp_pool.make_scev_sub_rec_expr(*inst)
        } else {
            self.scevexp_pool.make_scev_sub_expr(*inst)
        }
    }

    fn analyze_binary_mul(&mut self, inst: &ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if Self::check_rec(inst, self) {
            self.scevexp_pool.make_scev_mul_rec_expr(*inst)
        } else {
            self.scevexp_pool.make_scev_mul_expr(*inst)
        }
    }

    fn analyze_gep(&mut self, inst: &ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if Self::check_rec(inst, self) {
            self.scevexp_pool.make_scev_gep_rec_expr(*inst)
        } else {
            self.scevexp_pool.make_scev_gep_expr(*inst)
        }
    }

    fn analyze_phi(&mut self, inst: &ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if let Some(index) = inst
            .get_operands()
            .iter()
            .position(|op| op.get_operands().contains(&inst))
        {
            let dominator_tree = if let Some(dominator_tree) = self.dominator_tree {
                dominator_tree
            } else {
                return self.scevexp_pool.make_scev_unknown(*inst);
            };

            if inst.get_operand(index).get_kind() == InstKind::Binary(BinOp::Add)
                && dominator_tree.is_dominate(
                    &inst.get_parent_bb(),
                    &inst.get_operands()[index].get_parent_bb(),
                )
            {
                let start: Vec<_> = inst
                    .get_operands()
                    .iter()
                    .enumerate()
                    .filter(|(x, _)| *x != index)
                    .map(|(_, op)| *op)
                    .collect();
                debug_assert_eq!(start.len(), 1);
                let start = start[0];

                let step = inst.get_operands()[index]
                    .get_operands()
                    .iter()
                    .find(|x| *x != inst)
                    .cloned()
                    .unwrap();

                self.scevexp_pool.make_scev_rec_expr(*inst, start, step)
            } else {
                self.scevexp_pool.make_scev_unknown(*inst)
            }
        } else {
            self.scevexp_pool.make_scev_unknown(*inst)
        }
    }
}
