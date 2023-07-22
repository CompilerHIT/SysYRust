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

impl IrType {
    pub fn is_pointer(&self) -> bool {
        match self {
            IrType::IntPtr | IrType::FloatPtr => true,
            _ => false,
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            IrType::Int => true,
            _ => false,
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            IrType::Float => true,
            _ => false,
        }
    }

    pub fn is_void(&self) -> bool {
        match self {
            IrType::Void => true,
            _ => false,
        }
    }

    pub fn is_int_ptr(&self) -> bool {
        match self {
            IrType::IntPtr => true,
            _ => false,
        }
    }

    pub fn is_float_ptr(&self) -> bool {
        match self {
            IrType::FloatPtr => true,
            _ => false,
        }
    }

    pub fn is_parameter(&self) -> bool {
        match self {
            IrType::Parameter => true,
            _ => false,
        }
    }
}
