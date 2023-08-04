use crate::{
    ir::instruction::{Inst, InstKind},
    utility::ObjPtr,
};

use super::{loop_tree::LoopInfo, scev::SCEVAnalyzer};

pub fn dependency_check(gep: [ObjPtr<Inst>; 2], vector: Vec<(ObjPtr<Inst>, [i32; 2])>) -> bool {
    debug_assert_eq!(gep[0].get_ptr(), gep[1].get_ptr());

    if gep[0].get_gep_offset() == gep[1].get_gep_offset() {
        return false;
    }

    let inst_vec = vector.iter().map(|x| x.0.clone()).collect::<Vec<_>>();

    let matrix_1 = parse(gep[0].get_gep_offset(), &inst_vec);
    if matrix_1.is_empty() {
        return true;
    }

    let matrix_2 = parse(gep[1].get_gep_offset(), &inst_vec);
    if matrix_2.is_empty() {
        return true;
    }

    todo!()
}

fn parse(offset: ObjPtr<Inst>, vector: &Vec<ObjPtr<Inst>>) -> Vec<i32> {
    let mut result = Vec::new();
    result.resize(vector.len() + 1, 0);
    debug_assert_eq!(result.len(), vector.len() + 1);
    match offset.get_kind() {
        InstKind::Binary(crate::ir::instruction::BinOp::Add) => {
            parse_add(offset.get_lhs(), offset.get_rhs(), &mut result, vector)
        }
        InstKind::Binary(crate::ir::instruction::BinOp::Mul) => {
            parse_mul(offset.get_lhs(), offset.get_rhs(), &mut result, vector)
        }
        InstKind::Phi => parse_phi(offset, &mut result, vector),
        _ => Vec::new(),
    }
}

fn parse_add(
    lhs: ObjPtr<Inst>,
    rhs: ObjPtr<Inst>,
    result: &mut [i32],
    vector: &Vec<ObjPtr<Inst>>,
) -> Vec<i32> {
    todo!()
}

fn parse_mul(
    lhs: ObjPtr<Inst>,
    rhs: ObjPtr<Inst>,
    result: &mut [i32],
    vector: &Vec<ObjPtr<Inst>>,
) -> Vec<i32> {
    todo!()
}

fn parse_phi(phi: ObjPtr<Inst>, result: &mut [i32], vector: &Vec<ObjPtr<Inst>>) -> Vec<i32> {
    todo!()
}
