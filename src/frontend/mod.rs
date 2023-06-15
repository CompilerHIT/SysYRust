pub mod ast;
pub mod context;
pub mod error;
pub mod irgen;
pub mod kit;
pub mod typesearch;
use crate::{ir::basicblock::BasicBlock, utility::ObjPtr};

#[derive(Clone, Copy)]
pub enum InfuncChoice {
    InFunc(ObjPtr<BasicBlock>),
    NInFunc(),
}

#[derive(Debug, Clone, Copy)]
pub enum ExpValue {
    Float(f32),
    Int(i32),
    // True,
    // False,
    None,
}

pub enum RetInitVec {
    Float(Vec<f32>),
    Int(Vec<i32>),
}

pub fn init_padding_int(vec: &mut Vec<i32>, dimension_now: Vec<i32>) {
    let mut total = 1;
    let now = vec.len();
    for dm in dimension_now {
        total = total * dm;
    }

    // // println!("total:{:?},now{:?}",total,now);
    let need = total as usize - now;
    for i in 0..need {
        vec.push(0);
    }
}

pub fn init_padding_float(vec: &mut Vec<f32>, dimension_now: Vec<i32>) {
    let mut total = 1;
    let now = vec.len();
    for dm in dimension_now {
        total = total * dm;
    }

    let need = total as usize - now;
    for i in 0..need {
        vec.push(0.0);
    }
}
