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
