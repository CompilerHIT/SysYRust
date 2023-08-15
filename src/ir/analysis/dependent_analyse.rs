use std::collections::HashSet;

use crate::{
    ir::instruction::{Inst, InstKind},
    utility::ObjPtr,
};

use super::scev::scevexp::SCEVExp;

/// 分析循环内部关于数组操作的依赖关系
/// 根据vector的长度，可以大致分为三种情况：
/// a. vector.len() == 0 为ZIV，两次循环迭代间必定存在依赖关系
/// b. vector.len() == 1 为SIV，需要进行判断
/// c. vector.len() > 1 为MIV，需要进行判断
/// # Arguments
/// * 'gep' - 两条操作同一个数组的GEP指令
/// * 'vector' - 当前循环的IV向量，最外层循环的IV在最前面，内层循环依次往后;
///            - 每个元素包含一个SCEV表达式和一个bound，表示该IV的取值范围。第一个参数为LowBound,第二个为UpBound。
/// # Returns
/// true表示存在依赖关系，false表示不存在依赖关系
pub fn dependency_check(gep: [ObjPtr<Inst>; 2], vector: Vec<(ObjPtr<SCEVExp>, [i32; 2])>) -> bool {
    debug_assert!(if gep[0].get_gep_ptr().get_kind() == InstKind::Load {
        gep[0].get_gep_ptr().get_ptr() == gep[1].get_gep_ptr().get_ptr()
    } else {
        gep[0].get_gep_ptr() == gep[1].get_gep_ptr()
    });

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

    debug_assert_eq!(matrix_1.len(), matrix_2.len());

    let indexs = vector
        .iter()
        .enumerate()
        .filter_map(|(index, x)| {
            if matrix_1[index] != 0 && matrix_2[index] != 0 {
                Some(index)
            } else {
                None
            }
        })
        .collect::<HashSet<_>>();

    let new_vector = vector
        .iter()
        .enumerate()
        .filter_map(|(index, x)| {
            if indexs.contains(&index) {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let matrix_1 = matrix_1
        .iter()
        .enumerate()
        .filter_map(|(index, x)| {
            if indexs.contains(&index) || index == matrix_1.len() - 1 {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let matrix_2 = matrix_2
        .iter()
        .enumerate()
        .filter_map(|(index, x)| {
            if indexs.contains(&index) || index == matrix_2.len() - 1 {
                Some(x.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    if new_vector.len() == 0 {
        // ZIV
        true
    } else if new_vector.len() == 1 {
        // SIV
        siv_test(
            &matrix_1,
            &matrix_2,
            &new_vector.iter().map(|x| x.1).collect::<Vec<_>>(),
        )
    } else {
        // MIV
        miv_test(
            &matrix_1,
            &matrix_2,
            &new_vector.iter().map(|x| x.1).collect::<Vec<_>>(),
        )
    }
}

/// siv_test函数，分为强SIV和弱SIV两种情况
/// 强SIV：两个矩阵的第一列相同，且第二列的差值小于bound
/// 弱SIV：分为弱-0 SIV和弱-交叉 SIV
/// 弱-0 SIV：两个矩阵的第一列相反，且第二列的和小于bound
///
/// # Arguments
/// * 'matrix_1' - 第一个矩阵
/// * 'matrix_2' - 第二个矩阵
/// * 'bound' - 对应iv的上下界，第一个参数为下界，第二个参数为上界
/// # Returns
/// true表示存在依赖关系，false表示不存在依赖关系
fn siv_test(matrix_1: &[i32], matrix_2: &[i32], bound: &[[i32; 2]]) -> bool {
    debug_assert_eq!(matrix_1.len(), 2);
    debug_assert_eq!(matrix_2.len(), 2);
    debug_assert_eq!(bound.len(), 1);

    let matrix_1_64 = matrix_1.iter().map(|x| *x as i64).collect::<Vec<_>>();
    let matrix_2_64 = matrix_2.iter().map(|x| *x as i64).collect::<Vec<_>>();
    let low_bound = bound[0][0] as i64;
    let up_bound = bound[0][1] as i64;

    if matrix_1_64[0] == matrix_2_64[0] {
        // 强SIV

        let dis = (matrix_1_64[1] - matrix_2_64[1]).abs();
        if dis % matrix_1_64[0].abs() == 0 {
            dis < ((up_bound - low_bound) * matrix_1_64[0]).abs()
        } else {
            false
        }
    } else if matrix_1_64[0] == 0 {
        // 弱-0 SIV
        let dis = (matrix_1_64[1] - matrix_2_64[1]).abs();
        if dis % matrix_2_64[0].abs() == 0 {
            dis < ((up_bound - low_bound) * matrix_2_64[0]).abs()
        } else {
            false
        }
    } else if matrix_2_64[0] == 0 {
        // 弱-0 SIV
        let dis = (matrix_1_64[1] - matrix_2_64[1]).abs();
        if dis % matrix_1_64[0].abs() == 0 {
            dis < ((up_bound - low_bound) * matrix_1_64[0]).abs()
        } else {
            false
        }
    } else if matrix_1_64[0] + matrix_2_64[0] == 0 {
        // 弱-交叉 SIV
        let dis = (matrix_1_64[1] - matrix_2_64[1]).abs();
        if dis % (matrix_1_64[0].abs()) == 0 {
            dis < ((up_bound - low_bound) * matrix_1_64[0] * 2).abs()
        } else {
            false
        }
    } else {
        miv_test(matrix_1, matrix_2, bound)
    }
}

/// miv_test函数，分为gcd测试，Banerjee测试和I测试
/// # Arguments
/// * 'matrix_1' - 第一个矩阵
/// * 'matrix_2' - 第二个矩阵
/// * 'bound' - 对应iv的上下界，第一个参数为下界，第二个参数为上界
/// # Returns
/// true表示存在依赖关系，false表示不存在依赖关系
fn miv_test(matrix_1: &[i32], matrix_2: &[i32], bound: &[[i32; 2]]) -> bool {
    gcd_test(matrix_1, matrix_2)
        && banerjee_test(matrix_1, matrix_2, bound)
        && i_test(matrix_1, matrix_2, bound)
}

/// 求两个数的最大公约数
fn gcd(a: i32, b: i32) -> i32 {
    let mut a = a;
    let mut b = b;
    while b != 0 {
        let temp = b;
        b = a % b;
        a = temp;
    }
    a
}

/// 测试Fi + Gj = C的情况，其中F和G为访问矩阵，i和j为IV向量，C为常数，
/// 方程有一个解当且仅当等式左边系数的最大公约数可以整除等式右边的常数项
/// 缺陷：没有考虑边界条件
/// # Arguments
/// * 'matrix_1' - 第一个矩阵
/// * 'matrix_2' - 第二个矩阵
/// # Returns
/// true表示存在依赖关系，false表示不存在依赖关系
fn gcd_test(matrix_1: &[i32], matrix_2: &[i32]) -> bool {
    let matrix = matrix_1[0..(matrix_1.len() - 1)]
        .iter()
        .zip(matrix_2[0..(matrix_2.len() - 1)].iter())
        .map(|(x, y)| *x - *y)
        .collect::<Vec<_>>();

    let gcd = matrix.iter().fold(
        matrix.iter().find(|x| **x != 0).cloned().unwrap_or(1),
        |x, y| {
            if *y != 0 {
                gcd(x, *y)
            } else {
                x
            }
        },
    );

    (matrix_1[matrix_1.len() - 1] - matrix_2[matrix_2.len() - 1]) % gcd == 0
}

/// Banerjee测试，构造非常数项的表达式的一个取值区间，然后判断常数项是否在区间内
/// 缺陷：判断的是有无实数解，没有考虑整数依赖
/// # Arguments
/// * 'matrix_1' - 第一个矩阵
/// * 'matrix_2' - 第二个矩阵
/// * 'bound' - 对应iv的上下界，第一个参数为下界，第二个参数为上界
/// # Returns
/// true表示存在依赖关系，false表示不存在依赖关系
fn banerjee_test(matrix_1: &[i32], matrix_2: &[i32], bound: &[[i32; 2]]) -> bool {
    debug_assert!(matrix_1.len() == matrix_2.len());
    debug_assert!(matrix_1.len() > 1);

    let matrix_1_64 = matrix_1.iter().map(|x| *x as i64).collect::<Vec<_>>();

    let matrix_2_64 = matrix_2.iter().map(|x| *x as i64).collect::<Vec<_>>();

    let bound_64 = bound
        .iter()
        .map(|x| [x[0] as i64, x[1] as i64])
        .collect::<Vec<_>>();

    let c = matrix_2_64[matrix_2_64.len() - 1] - matrix_1_64[matrix_1_64.len() - 1];

    let mut h_k_positve: i64 = 0;
    let mut h_k_negative: i64 = 0;

    let a_positve = |a: i64| -> i64 {
        if a > 0 {
            a
        } else {
            0
        }
    };

    let a_negative = |a: i64| -> i64 {
        if a < 0 {
            -a
        } else {
            0
        }
    };

    debug_assert_eq!(a_positve(-10) - a_negative(-10), -10);
    debug_assert_eq!(a_positve(10) - a_negative(10), 10);

    for i in 0..(matrix_1_64.len() - 1) {
        let a = matrix_1_64[i];
        let b = matrix_2_64[i];
        let l = bound_64[i][0];
        let u = bound_64[i][1];

        debug_assert_eq!(a_positve(a) - a_negative(a), a);
        debug_assert_eq!(a_positve(b) - a_negative(b), b);

        h_k_positve += a_positve(a - b) * u - a_positve(a - b) * l;
        h_k_negative += -a_negative(a - b) * u + a_positve(a - b) * l;
    }

    h_k_negative <= c && c <= h_k_positve
}

/// I测试，
fn i_test(matrix_1: &[i32], matrix_2: &[i32], bound: &[[i32; 2]]) -> bool {
    debug_assert_eq!(matrix_1.len(), matrix_2.len());

    let mut new_matrix_1 = matrix_1.to_vec();
    let mut new_matrix_2 = matrix_2.to_vec();
    let mut new_bound = bound.to_vec();

    while new_matrix_1.len() > 2 {
        let cur_gcd = gcd(
            new_matrix_1.iter().fold(
                new_matrix_1.iter().find(|x| **x != 0).cloned().unwrap_or(1),
                |x, y| if *y != 0 { gcd(x, *y) } else { x },
            ),
            new_matrix_2.iter().fold(
                new_matrix_2.iter().find(|x| **x != 0).cloned().unwrap_or(1),
                |x, y| if *y != 0 { gcd(x, *y) } else { x },
            ),
        );

        if cur_gcd != 1 {
            new_matrix_1.iter_mut().for_each(|x| *x /= cur_gcd);
            new_matrix_2.iter_mut().for_each(|x| *x /= cur_gcd);
        }

        if banerjee_test(&new_matrix_1, &new_matrix_2, &new_bound) {
            new_matrix_1 = new_matrix_1[1..(new_matrix_1.len() - 1)].to_vec();
            new_matrix_2 = new_matrix_2[1..(new_matrix_2.len() - 1)].to_vec();
            new_bound = new_bound[1..(new_bound.len() - 1)].to_vec();
        } else {
            return false;
        }
    }

    gcd_test(&new_matrix_1, &new_matrix_2)
}

/// # Arguments
/// * 'offset' - GEP指令的偏移量
/// * 'vector' - 当前循环的IV向量
/// # Returns
/// 解析出来的矩阵，长度为vector.len() + 1，最后一位为常数项
fn parse(offset: ObjPtr<Inst>, vector: &Vec<ObjPtr<SCEVExp>>) -> Vec<i32> {
    let mut matrix = parse_recursion(offset, vector);

    if !matrix.is_empty() {
        // 考虑step，将矩阵中的每一项乘上step的绝对值
        debug_assert_eq!(vector.len(), matrix.len() - 1);
        vector.iter().enumerate().for_each(|(index, iv)| {
            let step = if iv.get_operands().len() == 2 && iv.get_operands()[1].is_scev_constant() {
                iv.get_operands()[1].get_scev_const().abs()
            } else {
                1
            };

            matrix[index] *= step;
        });
    }

    matrix
}

/// 识别表达式树，将以下变量视为叶节点:
/// a. 常数
/// b. 当前循环和子循环的IV
///
/// 而循环内部的基本运算指令（加减乘除）作为内部节点，进行后序遍历。
/// 当遇到以下变量时停止，分析失败：
/// a. 循环外部变量
/// b. Load指令
/// c. 非IV的Phi
/// d. call指令
/// e. 函数参数
///
/// 解析规则：
/// a. 使用递归函数访问每颗子树，每个递归函数维护一个对应IV向量的系数矩阵
/// b. 递归函数返回一个系数矩阵，如果分析失败则返回空矩阵
/// c. 根据当前内部节点的操作符不同，分别进行不同的处理
///   a). 加法：将左右子树的系数矩阵相加
///   b). 减法：左子树的系数矩阵减去右子树的系数矩阵
///   c). 乘法：左子树的系数矩阵乘上右子树的系数矩阵;
///            其中一颗子树的系数矩阵除常数项外全为0，否则分析失败
///   d). 除法：左子树的系数矩阵除以右子树的系数矩阵中的常数项;
///            其中右子树的系数矩阵除常数项外全为0，否则分析失败
///   e). 取模：左子树的系数矩阵取模右子树的系数矩阵中的常数项;
///            其中右子树的系数矩阵除常数项外全为0，否则分析失败
///   f). 其他：分析失败
fn parse_recursion(operand: ObjPtr<Inst>, vector: &Vec<ObjPtr<SCEVExp>>) -> Vec<i32> {
    let mut result = Vec::new();
    result.resize(vector.len() + 1, 0);

    // 处理叶结点

    if operand.is_const() {
        if operand.get_ir_type().is_int() {
            result[vector.len()] = operand.get_int_bond() as i32;
            return result;
        } else {
            result[vector.len()] = operand.get_float_bond() as i32;
        }
    }

    if operand.is_phi() {
        if let Some(index) = vector.iter().position(|x| x.get_bond_inst() == operand) {
            result[index] = 1;
            return result;
        } else {
            return Vec::new();
        }
    }

    if operand.is_param() {
        return Vec::new();
    }

    if !vector[0]
        .get_in_loop()
        .unwrap()
        .is_in_loop(&operand.get_parent_bb())
    {
        return Vec::new();
    }

    let has_iv = |x: &[i32]| -> bool {
        if let Some(index) = x.iter().position(|y| *y != 0) {
            index != x.len() - 1
        } else {
            false
        }
    };

    // 处理内部节点
    match operand.get_kind() {
        InstKind::Binary(crate::ir::instruction::BinOp::Add) => {
            let lhs = parse_recursion(operand.get_lhs(), vector);
            if lhs.is_empty() {
                return Vec::new();
            }

            let rhs = parse_recursion(operand.get_rhs(), vector);
            if rhs.is_empty() {
                return Vec::new();
            }

            for i in 0..result.len() {
                result[i] = lhs[i] + rhs[i];
            }

            result
        }
        InstKind::Binary(crate::ir::instruction::BinOp::Sub) => {
            let lhs = parse_recursion(operand.get_lhs(), vector);
            if lhs.is_empty() {
                return Vec::new();
            }

            let rhs = parse_recursion(operand.get_rhs(), vector);
            if rhs.is_empty() {
                return Vec::new();
            }

            for i in 0..result.len() {
                result[i] = lhs[i] - rhs[i];
            }

            result
        }
        InstKind::Binary(crate::ir::instruction::BinOp::Mul) => {
            let lhs = parse_recursion(operand.get_lhs(), vector);
            if lhs.is_empty() {
                return Vec::new();
            }

            let rhs = parse_recursion(operand.get_rhs(), vector);
            if rhs.is_empty() {
                return Vec::new();
            }

            debug_assert_eq!(lhs.len(), rhs.len());

            let lhs_has_iv = has_iv(&lhs);
            let rhs_has_iv = has_iv(&rhs);

            if lhs_has_iv && rhs_has_iv {
                return Vec::new();
            }

            if lhs_has_iv {
                for i in 0..result.len() {
                    result[i] = lhs[i] * rhs[rhs.len() - 1];
                }
            } else {
                for i in 0..result.len() {
                    result[i] = rhs[i] * lhs[lhs.len() - 1];
                }
            }
            result
        }
        InstKind::Binary(crate::ir::instruction::BinOp::Div) => {
            let lhs = parse_recursion(operand.get_lhs(), vector);
            if lhs.is_empty() {
                return Vec::new();
            }

            let rhs = parse_recursion(operand.get_rhs(), vector);
            if rhs.is_empty() {
                return Vec::new();
            }

            debug_assert_eq!(lhs.len(), rhs.len());

            if has_iv(&rhs) {
                Vec::new()
            } else {
                debug_assert_ne!(rhs[rhs.len() - 1], 0);
                for i in 0..result.len() {
                    result[i] = lhs[i] / rhs[rhs.len() - 1];
                }
                result
            }
        }
        InstKind::Binary(crate::ir::instruction::BinOp::Rem) => {
            let lhs = parse_recursion(operand.get_lhs(), vector);
            if lhs.is_empty() {
                return Vec::new();
            }

            let rhs = parse_recursion(operand.get_rhs(), vector);
            if rhs.is_empty() {
                return Vec::new();
            }

            debug_assert_eq!(lhs.len(), rhs.len());

            if has_iv(&rhs) {
                Vec::new()
            } else {
                debug_assert_ne!(rhs[rhs.len() - 1], 0);
                for i in 0..result.len() {
                    result[i] = lhs[i] % rhs[rhs.len() - 1];
                }
                result
            }
        }
        _ => Vec::new(),
    }
}
