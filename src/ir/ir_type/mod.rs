//! src/ir/ir_type/mod.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrType {
    Void,
    Int,
    ConstInt,
    IntPtr,
    Float,
    FloatPtr,
    ConstFloat,
    Function,
    BBlock,
    Parameter,
}
