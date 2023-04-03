use super::ir_type::IrType;
pub struct Value {
    ir_type: IrType,
    name: String,
}

impl Value {
    pub fn make_value(name: String, ir_type: IrType) -> Value {
        Value { ir_type, name }
    }
}
