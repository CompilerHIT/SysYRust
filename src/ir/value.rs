use super::ir_type::IrType;

#[derive(Debug, Clone)]
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

    pub fn set_ir_type(&mut self, ir_type: IrType) {
        self.ir_type = ir_type
    }
}
