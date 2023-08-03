use std::{
    collections::{HashMap, HashSet, LinkedList},
    hash::Hash,
};

use regex::internal::Exec;

use crate::{
    backend::{
        instrs::{AsmBuilder, BinaryOp, CmpOp, Func, InstrsType, LIRInst, Operand, SingleOp, BB},
        operand::Reg,
        BackendPool,
    },
    ir::{instruction::Inst, CallMap},
    utility::{ObjPool, ObjPtr, ScalarType},
};

///复杂值类型 (实际实现的时候需要)
pub struct ComplexValue {
    add: HashMap<ObjPtr<LIRInst>, usize>,
    minus: HashMap<ObjPtr<LIRInst>, usize>,
}

///通用值类型
#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    Inst(ObjPtr<LIRInst>),
    IImm(i64),
    FImm(String),
    Addr((String, i64)),
}
//值类型
#[derive(Clone, PartialEq, Eq)]
pub enum ValueType {
    Inst,
    IImm,
    FImm,
    Addr,
}

impl Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Value::Addr(addr) => {
                addr.hash(state);
            }
            Value::FImm(fimm) => {
                fimm.hash(state);
            }
            Value::Inst(inst) => {
                inst.hash(state);
            }
            Value::IImm(iimm) => {
                iimm.hash(state);
            }
            _ => (),
        }
        core::mem::discriminant(self).hash(state);
    }
}

impl Value {
    #[inline]
    pub fn get_type(&self) -> ValueType {
        match self {
            Value::Addr(_) => ValueType::Addr,
            Value::IImm(_) => ValueType::IImm,
            Value::FImm(_) => ValueType::FImm,
            Value::Inst(_) => ValueType::Inst,
            _ => unreachable!(),
        }
    }
    #[inline]
    pub fn get_imm(&self) -> Option<&i64> {
        match self {
            Value::IImm(val) => Some(val),
            _ => None,
        }
    }
    #[inline]
    pub fn get_fimm(&self) -> Option<&String> {
        match self {
            Value::FImm(val) => Some(val),
            _ => None,
        }
    }
    #[inline]
    pub fn get_addr(&self) -> Option<&(String, i64)> {
        match self {
            Value::Addr(val) => Some(val),
            _ => None,
        }
    }
}

impl PartialOrd for Value {
    fn lt(&self, other: &Self) -> bool {
        //判断是否小于(仅仅当两个数都是数字(整数)/都是对同一个数组下标的访问的时候)
        if self.get_type() != other.get_type() {
            false
        } else if let Value::IImm(lhs) = self {
            let rhs = other.get_imm().unwrap();
            lhs < rhs
        } else if let Value::Addr(addr) = self {
            let r_addr = other.get_addr().unwrap();
            if addr.0 != r_addr.0 {
                false
            } else {
                addr.1 < r_addr.1
            }
        } else {
            false
        }
    }
    fn le(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            false
        } else if let Value::IImm(lhs) = self {
            let rhs = other.get_imm().unwrap();
            lhs <= rhs
        } else if let Value::Addr(addr) = self {
            let r_addr = other.get_addr().unwrap();
            if addr.0 != r_addr.0 {
                false
            } else {
                addr.1 <= r_addr.1
            }
        } else {
            false
        }
    }

    fn gt(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            false
        } else if let Value::IImm(lhs) = self {
            let rhs = other.get_imm().unwrap();
            lhs > rhs
        } else if let Value::Addr(addr) = self {
            let r_addr = other.get_addr().unwrap();
            if addr.0 != r_addr.0 {
                false
            } else {
                addr.1 > r_addr.1
            }
        } else {
            false
        }
    }

    fn ge(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            false
        } else if let Value::IImm(lhs) = self {
            let rhs = other.get_imm().unwrap();
            lhs >= rhs
        } else if let Value::Addr(addr) = self {
            let r_addr = other.get_addr().unwrap();
            if addr.0 != r_addr.0 {
                false
            } else {
                addr.1 >= r_addr.1
            }
        } else {
            false
        }
    }
    ///value并不都能排序
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        todo!()
    }
}

///实现一些对value的运算
impl Value {
    //只有同类型的值才可以进行加减运算
    pub fn add_another(&mut self, another: &Value) {}
    pub fn minus_another(&mut self, another: &Value) {}

    ///如果运算成功,返回值,如果失败,返回None
    pub fn add(one: &Value, another: &Value) -> Option<Value> {
        if one.get_type() == another.get_type() && one.get_type() == ValueType::IImm {
            let v1 = one.get_imm().unwrap();
            let v2 = one.get_imm().unwrap();
            return Some(Value::IImm(v1 + v2));
        } else if let Some(one) = one.get_addr() {
            if let Some(imm) = another.get_imm() {
                let mut new_v = one.clone();
                new_v.1 += imm;
                return Some(Value::Addr(new_v));
            }
        } else if let Some(another) = another.get_addr() {
            if let Some(imm) = one.get_imm() {
                let mut new_v = another.clone();
                new_v.1 += imm;
                return Some(Value::Addr(new_v));
            }
        }
        None
    }

    ///如果运算成功,返回值,如果失败,返回None
    pub fn minus(one: &Value, another: &Value) -> Option<Value> {
        todo!()
    }
}

///内置函数 (比如一些io函数)
pub struct BuiltInFunc {}
