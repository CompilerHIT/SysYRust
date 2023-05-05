use super::ir_type_cfg::CfgIrType;

pub struct CfgValue {
    ir_type: CfgIrType,
}

impl CfgValue {
    pub fn make_value(ir_type: CfgIrType) -> CfgValue {
        CfgValue { ir_type }
    }
}
