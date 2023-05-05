//! src/ir/ir_type/mod.rs

pub enum CfgIrType {
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

impl CfgIrType {
    pub fn is_void(&self) -> bool {
        match self {
            CfgIrType::Void => true,
            __ => false,
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            CfgIrType::Bool => true,
            __ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            CfgIrType::Int => true,
            __ => false,
        }
    }
    pub fn is_float(&self) -> bool {
        match self {
            CfgIrType::Float => true,
            __ => false,
        }
    }
    pub fn is_pointer(&self) -> bool {
        match self {
            CfgIrType::Pointer => true,
            __ => false,
        }
    }
    pub fn is_array(&self) -> bool {
        match self {
            CfgIrType::Array => true,
            __ => false,
        }
    }
    pub fn is_function(&self) -> bool {
        match self {
            CfgIrType::Function => true,
            __ => false,
        }
    }
    pub fn is_bblock(&self) -> bool {
        match self {
            CfgIrType::BBlock => true,
            __ => false,
        }
    }
    pub fn is_parameter(&self) -> bool {
        match self {
            CfgIrType::Parameter => true,
            __ => false,
        }
    }
}

#[test]
fn is_void_test() {
    let tested = CfgIrType::Void;
    assert!(tested.is_void());
}

// #[test]
// fn global_variable_test() {
//     let result = 2 + 2;
//     // sysy::CompUnit::new().parse("22").is_ok();
//     assert_eq!(result, 4);
//     // "int".to_string();
// }
