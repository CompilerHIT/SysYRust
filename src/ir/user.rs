use super::ir_type::IrType;
use super::value::Value;
pub struct User {
    value: Value,
}

impl User {
    pub fn make_user(name: String, ir_type: IrType) -> User {
        let value = Value::make_value(name, ir_type);
        User { value }
    }
}
