use super::ir_type::IrType;

pub struct Value {
    ir_type: IrType,
}

impl Value {
    pub fn new(ir_type: IrType) -> Value {
        Value { ir_type }
    }

    pub fn get_ir_type(&self) -> IrType {
        self.ir_type
    }
}
