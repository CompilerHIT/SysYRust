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
                scev = self.scevexp_pool.make_scev_int_constant(value);
            }
            InstKind::ConstFloat(value) | InstKind::GlobalConstFloat(value) => {
                scev = self.scevexp_pool.make_scev_float_constant(value);
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

    fn analyze_binary_add(
        &mut self,
        inst: &ObjPtr<Inst>,
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> ObjPtr<SCEVExp> {
        todo!()
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
                    self.scevexp_pool
                        .make_scev_int_constant(parsee.get_int_bond())
                } else if parsee.is_float_const() {
                    self.scevexp_pool
                        .make_scev_float_constant(parsee.get_float_bond())
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
