//! src/ir/ir_type/mod.rs

#[derive(Debug, Clone, Copy)]
pub enum IrType {
    Void,
    Bool,
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
