use std::collections::HashMap;

use crate::{
    ir::instruction::{BinOp, Inst, InstKind},
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
        let in_loop = if inst.is_param() || inst.is_global_var() {
            None
        } else {
            self.loop_list
                .iter()
                .find(|li| li.is_in_current_loop(&inst.get_parent_bb()))
                .cloned()
        };

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

    fn chech_constant_or_no_in_loop(exp: ObjPtr<SCEVExp>, cur_loop: ObjPtr<LoopInfo>) -> bool {
        exp.is_scev_constant()
            || exp.get_in_loop() == None
            || exp.get_in_loop().unwrap() != cur_loop
                && cur_loop.is_a_sub_loop(exp.get_in_loop().unwrap())
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

        match Self::check_rec_available(lhs, rhs, cur_loop) {
            (true, true) => {
                let lhs_op = lhs.get_operands();
                let rhs_op = rhs.get_operands();

                let result = self.parse_add(&lhs_op, &rhs_op, in_loop);
                self.scevexp_pool
                    .make_scev_add_rec_expr(result, Some(*inst), in_loop)
            }
            (true, false) => {
                if Self::chech_constant_or_no_in_loop(rhs, cur_loop) {
                    let result = self.parse_add(&lhs.get_operands(), &[rhs], in_loop);
                    self.scevexp_pool
                        .make_scev_add_rec_expr(result, Some(*inst), in_loop)
                } else {
                    self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
                }
            }
            (false, true) => {
                if Self::chech_constant_or_no_in_loop(lhs, cur_loop) {
                    let result = self.parse_add(&rhs.get_operands(), &[lhs], in_loop);
                    self.scevexp_pool
                        .make_scev_add_rec_expr(result, Some(*inst), in_loop)
                } else {
                    self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
                }
            }
            (false, false) => self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop),
        }
    }

    fn analyze_binary_sub(
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

        match Self::check_rec_available(lhs, rhs, cur_loop) {
            (true, true) => {
                let lhs_op = lhs.get_operands();
                let rhs_op = rhs.get_operands();

                let result = self.parse_sub(&lhs_op, &rhs_op, in_loop);
                self.scevexp_pool
                    .make_scev_sub_rec_expr(result, Some(*inst), in_loop)
            }
            (true, false) => {
                let lhs_op = lhs.get_operands();
                if Self::chech_constant_or_no_in_loop(rhs, cur_loop) {
                    let result = self.parse_sub(&lhs_op, &[rhs], in_loop);
                    self.scevexp_pool
                        .make_scev_sub_rec_expr(result, Some(*inst), in_loop)
                } else {
                    self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
                }
            }
            (false, true) => {
                let rhs_op = rhs.get_operands();
                if Self::chech_constant_or_no_in_loop(lhs, cur_loop) {
                    let result = self.parse_sub(&rhs_op, &[lhs], in_loop);
                    self.scevexp_pool
                        .make_scev_sub_rec_expr(result, Some(*inst), in_loop)
                } else {
                    self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
                }
            }
            (false, false) => self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop),
        }
    }

    fn analyze_binary_mul(
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

        match Self::check_rec_available(lhs, rhs, cur_loop) {
            (true, true) => {
                let lhs_op = lhs.get_operands();
                let rhs_op = rhs.get_operands();

                let result = self.parse_mul(&lhs_op, &rhs_op, in_loop);
                self.scevexp_pool
                    .make_scev_mul_rec_expr(result, Some(*inst), in_loop)
            }
            (true, false) => {
                let lhs_op = lhs.get_operands();

                if Self::chech_constant_or_no_in_loop(rhs, cur_loop) {
                    let result = self.parse_mul(&lhs_op, &[rhs], in_loop);
                    self.scevexp_pool
                        .make_scev_mul_rec_expr(result, Some(*inst), in_loop)
                } else {
                    self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
                }
            }
            (false, true) => {
                let rhs_op = rhs.get_operands();

                if Self::chech_constant_or_no_in_loop(lhs, cur_loop) {
                    let result = self.parse_mul(&[lhs], &rhs_op, in_loop);
                    self.scevexp_pool
                        .make_scev_mul_rec_expr(result, Some(*inst), in_loop)
                } else {
                    self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
                }
            }
            (false, false) => self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop),
        }
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

        if inst.get_operands().len() != 2 {
            return self.scevexp_pool.make_scev_unknown(Some(*inst), None);
        }

        if let Some(index) = inst
            .get_operands()
            .iter()
            .position(|x| x.get_operands().contains(inst))
        {
            let op = inst.get_operand(index);
            let mut parse = |parsee: ObjPtr<Inst>| -> ObjPtr<SCEVExp> {
                if parsee.is_int_const() {
                    self.scevexp_pool.make_scev_constant(parsee.get_int_bond())
                } else if parsee.is_global_var()
                    || parsee.is_param()
                    || cur_loop.is_in_current_loop(&parsee.get_parent_bb())
                {
                    self.scevexp_pool.make_scev_unknown(Some(parsee), in_loop)
                } else {
                    self.analyze(&parsee)
                }
            };

            let check_inst_avaliable = |inst: ObjPtr<Inst>| -> bool {
                inst.is_const()
                    || inst.is_global_var()
                    || inst.is_param()
                    || !cur_loop.is_in_loop(&inst.get_parent_bb())
            };

            match op.get_kind() {
                InstKind::Binary(BinOp::Add) => {
                    let step = op.get_operand((index + 1) % 2);
                    if check_inst_avaliable(step) {
                        let start = inst
                            .get_operands()
                            .iter()
                            .find(|x| check_inst_avaliable(**x))
                            .unwrap();

                        let result = vec![parse(*start), parse(step)];
                        return self
                            .scevexp_pool
                            .make_scev_rec_expr(result, Some(*inst), in_loop);
                    }
                }

                InstKind::Binary(BinOp::Sub) => {
                    let step = op.get_operand(1);
                    if index == 0 && check_inst_avaliable(step) {
                        let start = inst
                            .get_operands()
                            .iter()
                            .find(|x| check_inst_avaliable(**x))
                            .unwrap();
                        let result = vec![parse(*start), parse(step)];
                        return self
                            .scevexp_pool
                            .make_scev_rec_expr(result, Some(*inst), in_loop);
                    }
                }
                _ => {}
            }
        }
        self.scevexp_pool.make_scev_unknown(Some(*inst), in_loop)
    }

    fn parse_add(
        &mut self,
        l_slice: &[ObjPtr<SCEVExp>],
        r_slice: &[ObjPtr<SCEVExp>],
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> Vec<ObjPtr<SCEVExp>> {
        let mut op_vec = Vec::new();
        debug_assert_ne!(l_slice.len(), 0);
        debug_assert_ne!(r_slice.len(), 0);

        let max_len = if l_slice.len() > r_slice.len() {
            l_slice.len()
        } else {
            r_slice.len()
        };

        for i in 0..max_len {
            if i < l_slice.len() && i < r_slice.len() {
                if l_slice[i].is_scev_constant() && r_slice[i].is_scev_constant() {
                    op_vec.push(self.scevexp_pool.make_scev_constant(
                        l_slice[i].get_scev_const() + r_slice[i].get_scev_const(),
                    ));
                } else {
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_add_expr(l_slice[i], r_slice[i], in_loop),
                    );
                }
            } else if i < l_slice.len() {
                op_vec.push(l_slice[i]);
            } else {
                op_vec.push(r_slice[i]);
            }
        }

        op_vec
    }

    fn parse_sub(
        &mut self,
        l_slice: &[ObjPtr<SCEVExp>],
        r_slice: &[ObjPtr<SCEVExp>],
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> Vec<ObjPtr<SCEVExp>> {
        let mut op_vec = Vec::new();
        debug_assert_ne!(l_slice.len(), 0);
        debug_assert_ne!(r_slice.len(), 0);

        let max_len = if l_slice.len() > r_slice.len() {
            l_slice.len()
        } else {
            r_slice.len()
        };

        for i in 0..max_len {
            if i < l_slice.len() && i < r_slice.len() {
                if l_slice[i].is_scev_constant() && r_slice[i].is_scev_constant() {
                    op_vec.push(self.scevexp_pool.make_scev_constant(
                        l_slice[i].get_scev_const() - r_slice[i].get_scev_const(),
                    ));
                } else {
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_sub_expr(l_slice[i], r_slice[i], in_loop),
                    );
                }
            } else if i < l_slice.len() {
                op_vec.push(l_slice[i]);
            } else {
                if r_slice[i].is_scev_constant() {
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_constant(-r_slice[i].get_scev_const()),
                    );
                } else {
                    let result = self.scevexp_pool.make_scev_constant(0);
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_sub_expr(result, r_slice[i], in_loop),
                    );
                }
            }
        }

        op_vec
    }

    fn parse_mul(
        &mut self,
        l_slice: &[ObjPtr<SCEVExp>],
        r_slice: &[ObjPtr<SCEVExp>],
        in_loop: Option<ObjPtr<LoopInfo>>,
    ) -> Vec<ObjPtr<SCEVExp>> {
        let mut op_vec = Vec::new();
        debug_assert_ne!(l_slice.len(), 0);
        debug_assert_ne!(r_slice.len(), 0);
        if l_slice.len() == 1 {
            for op in r_slice {
                if l_slice[0].is_scev_constant() && op.is_scev_constant() {
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_constant(l_slice[0].get_scev_const() * op.get_scev_const()),
                    );
                } else {
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_mul_expr(l_slice[0], *op, in_loop),
                    );
                }
            }
        } else if r_slice.len() == 1 {
            for op in l_slice {
                if op.is_scev_constant() && r_slice[0].is_scev_constant() {
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_constant(op.get_scev_const() * r_slice[0].get_scev_const()),
                    );
                } else {
                    op_vec.push(
                        self.scevexp_pool
                            .make_scev_mul_expr(*op, r_slice[0], in_loop),
                    );
                }
            }
        } else {
            let l_0 = l_slice[0];
            let r_0 = r_slice[0];
            let l_left = &l_slice[1..];
            let r_left = &r_slice[1..];

            let l_res = self.parse_mul(l_left, r_slice, in_loop);
            let r_res = self.parse_mul(l_slice, r_left, in_loop);
            let res = self.parse_mul(l_left, r_left, in_loop);

            op_vec.extend(self.parse_add(&[l_0], &[r_0], in_loop));

            let result = self.parse_add(&l_res, &r_res, in_loop);
            op_vec.extend(self.parse_add(&result, &res, in_loop));
        }

        op_vec
    }
}
