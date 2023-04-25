//! src/ir/ir_type/mod.rs

#[derive(Clone, Copy)]
pub enum IrType {
    Void,
    Bool,
    Int,
    ConstInt,
    Float,
    ConstFloat,
    Pointer,
    Array,
    Function,
    BBlock,
    Parameter,
}

impl IrType {
    pub fn is_void(&self) -> bool {
        match self {
            IrType::Void => true,
            __ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            IrType::Bool => true,
            __ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            IrType::Int => true,
            __ => false,
        }
    }
    pub fn is_float(&self) -> bool {
        match self {
            IrType::Float => true,
            __ => false,
        }
    }
    pub fn is_pointer(&self) -> bool {
        match self {
            IrType::Pointer => true,
            __ => false,
        }
    }
    pub fn is_array(&self) -> bool {
        match self {
            IrType::Array => true,
            __ => false,
        }
    }
    pub fn is_function(&self) -> bool {
        match self {
            IrType::Function => true,
            __ => false,
        }
    }
    pub fn is_bblock(&self) -> bool {
        match self {
            IrType::BBlock => true,
            __ => false,
        }
    }
    pub fn is_parameter(&self) -> bool {
        match self {
            IrType::Parameter => true,
            __ => false,
        }
    }
}

#[test]
fn is_void_test() {
    let tested = IrType::Void;
    assert!(tested.is_void());
}
