//! src/ir/ir_type/mod.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrType {
    Void,
    Int,
    IntPtr,
    Float,
    FloatPtr,
    Function,
    BBlock,
    Parameter,
}
