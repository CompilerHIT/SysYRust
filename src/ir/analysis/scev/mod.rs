use std::collections::HashMap;

use crate::{
    ir::instruction::{BinOp, Inst, InstKind, UnOp},
    utility::{ObjPool, ObjPtr},
};

use self::scevexp::{SCEVExp, SCEVExpKind};

use super::loop_tree::LoopList;

pub mod scevexp;

pub struct SCEVAnalyzer<'a> {
    scevexp_pool: ObjPool<SCEVExp>,
    map: HashMap<ObjPtr<Inst>, ObjPtr<SCEVExp>>,
    loops: Option<&'a LoopList>,
}

impl<'a> SCEVAnalyzer<'a> {
    pub fn new() -> Self {
        Self {
            scevexp_pool: ObjPool::new(),
            map: HashMap::new(),
            loops: None,
        }
    }

    pub fn set_loops(&mut self, loops: &'a LoopList) {
        self.loops = Some(loops);
    }

    pub fn clear(&mut self) {
        self.scevexp_pool.free_all();
        self.map.clear();
    }

    pub fn analyze(&mut self, inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if let Some(scev) = self.map.get(&inst) {
            return *scev;
        }

        let mut scev = self.scevexp_pool.make_scev_unknown(inst);
        self.map.insert(inst, scev);

        match inst.get_kind() {
            InstKind::Binary(BinOp::Add) => {
                scev = self.analyze_binary_add(inst);
                self.map.insert(inst, scev);
            }
            InstKind::Binary(BinOp::Sub) => {
                scev = self.analyze_binary_sub(inst);
            }
            InstKind::Binary(BinOp::Mul) => {
                scev = self.analyze_binary_mul(inst);
                self.map.insert(inst, scev);
            }
            InstKind::Unary(UnOp::Neg) => {
                scev = self.analyze_unary_neg(inst);
                self.map.insert(inst, scev);
            }
            InstKind::Gep => {
                scev = self.analyze_gep(inst);
                self.map.insert(inst, scev);
            }
            InstKind::ConstInt(_)
            | InstKind::ConstFloat(_)
            | InstKind::GlobalConstInt(_)
            | InstKind::GlobalConstFloat(_) => {
                scev = self.scevexp_pool.make_scev_constant(inst);
                self.map.insert(inst, scev);
            }
            InstKind::Phi => {
                scev = self.analyze_phi(inst);
                self.map.insert(inst, scev);
            }
            _ => {}
        }

        scev
    }

    fn analyze_binary_add(&mut self, inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.analyze(inst.get_lhs());
        self.analyze(inst.get_rhs());
        self.scevexp_pool
            .make_scev_add_expr(inst.get_operands().clone(), inst)
    }

    fn analyze_binary_sub(&mut self, inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.analyze(inst.get_lhs());
        self.analyze(inst.get_rhs());
        self.scevexp_pool
            .make_scev_sub_expr(inst.get_operands().clone(), inst)
    }

    fn analyze_binary_mul(&mut self, inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        let lhs = self.analyze(inst.get_lhs());
        let rhs = self.analyze(inst.get_rhs());
        self.scevexp_pool
            .make_scev_mul_expr(vec![lhs.get_bond_inst(), rhs.get_bond_inst()], inst)
    }

    fn analyze_gep(&mut self, inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        let lhs = self.analyze(inst.get_gep_ptr());
        let rhs = self.analyze(inst.get_gep_offset());
        self.scevexp_pool
            .make_scev_add_expr(vec![lhs.get_bond_inst(), rhs.get_bond_inst()], inst)
    }

    fn analyze_unary_neg(&mut self, inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        self.analyze(inst.get_unary_operand())
    }

    fn analyze_phi(&mut self, inst: ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if let None = self.loops {
            return self.scevexp_pool.make_scev_unknown(inst);
        }

        if inst.get_operands().len() > 2 {
            return self.scevexp_pool.make_scev_unknown(inst);
        }

        let loop_list = self.loops.unwrap();
        let bb = inst.get_parent_bb();
        if let Some(loop_info) = loop_list.get_loop_list().iter().find(|&l| l.is_header(bb)) {
            let op1 = self.analyze(inst.get_operands()[0]);
            let op2 = self.analyze(inst.get_operands()[1]);

            let mut match_rec_expr = |op1: ObjPtr<SCEVExp>,
                                      op2: ObjPtr<SCEVExp>|
             -> ObjPtr<SCEVExp> {
                if let SCEVExpKind::SCEVAddExpr = op1.get_kind() {
                    let adder = op1.get_operands()[0];
                    let addee = op1.get_operands()[1];
                    if adder == inst && addee.is_const() {
                        return self
                            .scevexp_pool
                            .make_scev_add_rec_expr(vec![op2.get_bond_inst(), addee], inst);
                    } else if addee == inst && adder.is_const() {
                        return self
                            .scevexp_pool
                            .make_scev_add_rec_expr(vec![op2.get_bond_inst(), adder], inst);
                    }
                } else if let SCEVExpKind::SCEVSubExpr = op1.get_kind() {
                    let minuend = op1.get_operands()[0];
                    let subtrahend = op1.get_operands()[1];
                    if minuend == inst && subtrahend.is_const() {
                        return self
                            .scevexp_pool
                            .make_scev_sub_rec_expr(vec![op2.get_bond_inst(), subtrahend], inst);
                    }
                }
                return self.scevexp_pool.make_scev_unknown(inst);
            };

            if op2.get_bond_inst().is_global_var_or_param()
                || (loop_info.is_in_current_loop(&op1.get_bond_inst().get_parent_bb())
                    && !loop_info.is_in_current_loop(&op2.get_bond_inst().get_parent_bb()))
            {
                return match_rec_expr(op1, op2);
            } else if !loop_info.is_in_loop(&op1.get_bond_inst().get_parent_bb())
                && loop_info.is_in_loop(&op2.get_bond_inst().get_parent_bb())
            {
                return match_rec_expr(op2, op1);
            }
        }

        self.scevexp_pool.make_scev_unknown(inst)
    }
}
