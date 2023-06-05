pub mod ast;
pub mod context;
pub mod error;
pub mod irgen;

#[derive(Debug, Clone, Copy)]
pub enum ExpValue {
    Float(f32),
    Int(i32),
    None,
}

pub fn init_padding_int(now:i32,num_of_sons:i32,dimension_now:&Vec<i32>) ->Vec<i32>{
    let mut vec = vec![];
    let mut total = 1;
    for dm in dimension_now{
        total = total*dm;
    }
    if dimension_now.len()==1&&num_of_sons!=0{
        unreachable!()
    }
    let son_has = num_of_sons*(total/dimension_now[0]);
    let need = total-now-son_has;
    for i in 0..need{
        vec.push(0);
    }
    vec
}

pub fn init_padding_float(now:i32,num_of_sons:i32,dimension_now:&Vec<i32>) ->Vec<f32>{
    let mut vec = vec![];
    let mut total = 1;
    for dm in dimension_now{
        total = total*dm;
    }
    if dimension_now.len()==1&&num_of_sons!=0{
        unreachable!()
    }
    let son_has = num_of_sons*(total/dimension_now[0]);
    let need = total-now-son_has;
    for i in 0..need{
        vec.push(0.0);
    }
    vec
}