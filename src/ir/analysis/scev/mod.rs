use std::collections::HashMap;

use crate::{
    ir::{
        instruction::{BinOp, Inst, InstKind},
        ir_type::IrType,
    },
    utility::{ObjPool, ObjPtr},
};

use self::scevexp::SCEVExp;

use super::loop_tree::LoopInfo;

pub mod scevexp;

pub struct SCEVAnalyzer {
    scevexp_pool: ObjPool<SCEVExp>,
    map: HashMap<ObjPtr<Inst>, ObjPtr<SCEVExp>>,
    loop_list: Vec<ObjPtr<LoopInfo>>,
}

impl SCEVAnalyzer {
    pub fn new() -> Self {
        Self {
            scevexp_pool: ObjPool::new(),
            map: HashMap::new(),
            loop_list: Vec::new(),
        }
    }

    pub fn set_loop_list(&mut self, loop_list: Vec<ObjPtr<LoopInfo>>) {
        self.loop_list = loop_list;
    }

    pub fn clear(&mut self) {
        self.scevexp_pool.free_all();
        self.map.clear();
    }

    pub fn analyze(&mut self, inst: &ObjPtr<Inst>) -> ObjPtr<SCEVExp> {
        if let Some(scev) = self.map.get(inst) {
            return *scev;
        }
        let in_loop = self
            .loop_list
            .iter()
            .find(|li| li.is_in_current_loop(&inst.get_parent_bb()))
            .cloned();

        let scev;

        match inst.get_kind() {
            InstKind::Binary(BinOp::Add) => {
                scev = self.analyze_binary_add(inst, in_loop);
            }
            InstKind::Binary(BinOp::Sub) => {
                scev = self.analyze_binary_sub(inst, in_loop);
            }
            InstKind::Binary(BinOp::Mul) => {
                scev = self.analyze_binary_mul(inst, in_loop);
            }
            InstKind::ConstInt(value) | InstKind::GlobalConstInt(value) => {
                scev = self.scevexp_pool.make_scev_constant(value);
            }
            InstKind::Phi => {
                scev = self.analyze_phi(inst, in_loop);
            }
            _ => {
                scev = self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop);
            }
        }

        self.map.insert(*inst, scev);
        scev
    }

    fn check_rec_available(
        lhs: ObjPtr<SCEVExp>,
        rhs: ObjPtr<SCEVExp>,
        cur_loop: ObjPtr<LoopInfo>,
    ) -> (bool, bool) {
        (
            lhs.is_scev_rec() && lhs.get_in_loop().unwrap() == cur_loop,
            rhs.is_scev_rec() && rhs.get_in_loop().unwrap() == cur_loop,
        )
    }

    fn analyze_binary_add(
        &mut self,
        inst: &ObjPtr<Inst>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        if let None = in_loop {
            return self.scevexp_pool.make_scev_unknown(Some(*inst), None);
        }

        let cur_loop = in_loop.unwrap();
        let lhs = self.analyze(&inst.get_lhs());
        let rhs = self.analyze(&inst.get_rhs());

        let mut lhs_rhs_op = |lhs: ObjPtr<SCEVExp>, rhs: ObjPtr<SCEVExp>| -> ObjPtr<SCEVExp> {
            let mut new_op: Vec<ObjPtr<SCEVExp>> = Vec::new();
            if rhs.is_scev_constant() {
                new_op = lhs
                    .get_operands()
                    .iter()
                    .map(|op| {
                        if op.is_scev_constant() {
                            self.scevexp_pool
                                .make_scev_constant(op.get_scev_const() + rhs.get_scev_const())
                        } else {
                            self.scevexp_pool.make_scev_add_expr(*op, rhs, in_loop)
                        }
                    })
                    .collect();
            } else if rhs.get_in_loop().unwrap() != cur_loop
                && cur_loop.is_a_sub_loop(rhs.get_in_loop().unwrap())
            {
                new_op = lhs
                    .get_operands()
                    .iter()
                    .map(|op| self.scevexp_pool.make_scev_add_expr(*op, rhs, in_loop))
                    .collect();
            } else {
                return self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop);
            }
            self.scevexp_pool
                .make_scev_add_rec_expr(new_op, Some(*inst), in_loop)
        };

        match Self::check_rec_available(lhs, rhs, cur_loop) {
            (true, true) => {
                let lhs_op = lhs.get_operands();
                let rhs_op = rhs.get_operands();

                let max_index = if lhs_op.len() > rhs_op.len() {
                    lhs_op.len()
                } else {
                    rhs_op.len()
                };

                let mut new_op = Vec::new();

                for index in 0..max_index {
                    if index >= lhs_op.len() {
                        new_op.push(rhs_op[index]);
                    } else if index >= rhs_op.len() {
                        new_op.push(lhs_op[index]);
                    } else {
                        if lhs_op[index].is_scev_constant() && rhs_op[index].is_scev_constant() {
                            new_op.push(self.scevexp_pool.make_scev_constant(
                                lhs_op[index].get_scev_const() + rhs_op[index].get_scev_const(),
                            ));
                        } else {
                            new_op.push(self.scevexp_pool.make_scev_add_expr(
                                lhs_op[index],
                                rhs_op[index],
                                in_loop,
                            ));
                        }
                    }
                }

                self.scevexp_pool
                    .make_scev_add_rec_expr(new_op, Some(*inst), in_loop)
            }
            (true, false) => lhs_rhs_op(lhs, rhs),
            (false, true) => lhs_rhs_op(rhs, lhs),
            (false, false) => self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop),
        }
    }

    fn analyze_binary_sub(
        &mut self,
        inst: &ObjPtr<Inst>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        todo!()
    }

    fn analyze_binary_mul(
        &mut self,
        inst: &ObjPtr<Inst>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        todo!()
    }

    fn analyze_phi(
        &mut self,
        inst: &ObjPtr<Inst>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        let cur_loop = if let Some(x) = in_loop {
            x
        } else {
            return self.scevexp_pool.make_scev_unknown(Some(*inst), None);
        };

        if let Some(index) = inst
            .get_operands()
            .iter()
            .position(|x| x.get_operands().contains(inst))
        {
            let op = inst.get_operand(index);
            let mut parse = |parsee: ObjPtr<Inst>| -> ObjPtr<SCEVExp> {
                if parsee.is_int_const() {
                    self.scevexp_pool.make_scev_constant(parsee.get_int_bond())
                } else {
                    self.scevexp_pool.make_scev_unknown(Some(parsee), in_loop)
                }
            };
            match op.get_kind() {
                InstKind::Binary(BinOp::Add) => {
                    let step = op.get_operand((index + 1) % 2);
                    if step.is_const() || !cur_loop.is_in_loop(&step.get_parent_bb()) {
                        let start = inst
                            .get_operands()
                            .iter()
                            .find(|x| !cur_loop.is_in_loop(&x.get_parent_bb()))
                            .unwrap();

                        return self.scevexp_pool.make_scev_rec_expr(
                            vec![parse(*start), parse(step)],
                            Some(*inst),
                            in_loop,
                        );
                    }
                }

                InstKind::Binary(BinOp::Sub) => {
                    let step = op.get_operand(1);
                    if index == 0
                        && (step.is_const() || !cur_loop.is_in_loop(&step.get_parent_bb()))
                    {
                        let start = inst
                            .get_operands()
                            .iter()
                            .find(|x| !cur_loop.is_in_loop(&x.get_parent_bb()))
                            .unwrap();
                        return self.scevexp_pool.make_scev_rec_expr(
                            vec![parse(*start), parse(step)],
                            Some(*inst),
                            in_loop,
                        );
                    }
                }
                _ => {}
            }
        }
        self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
    }
}
