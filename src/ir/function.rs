use super::{basicblock::BasicBlock, instruction::Inst, ir_type::IrType, value::Value};
use std::collections::HashMap;

pub struct Function {
    value: Value,
    parameters: HashMap<&'static str, &'static Inst>,
    head_block: Option<&'static BasicBlock>,
}

impl Function {
    /// 构造一个没有bb和参数的函数
    pub fn new() -> Function {
        Function {
            value: Value::new(IrType::Function),
            parameters: HashMap::new(),
            head_block: None,
        }
    }

    /// 判断函数中是否有bb
    pub fn is_empty_bb(&self) -> bool {
        if let None = self.head_block {
            true
        } else {
            false
        }
    }

    /// 获得第一个块，默认为非空块
    pub fn get_head(&self) -> &BasicBlock {
        match self.head_block {
            Some(bb) => bb,
            None => panic!("尝试获得一个空的BB"),
        }
    }

    /// 为函数增加参数
    pub fn set_parameter(&mut self, name: &str, parameter: &Inst) {
        self.parameters.insert(name, parameter);
    }

    /// 获得参数
    /// 默认参数存在
    pub fn get_parameter(&self, name: &str) -> &Inst {
        match self.parameters.get(name) {
            Some(p) => p,
            None => panic!("尝试获得不存在的参数"),
        }
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }
}
