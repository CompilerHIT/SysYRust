use crate::utility::ObjPtr;

use super::{basicblock::BasicBlock, instruction::Inst, ir_type::IrType, value::Value};
use std::collections::HashMap;

pub struct Function {
    value: Value,
    return_type: IrType,
    parameters: HashMap<&'static str, ObjPtr<Inst>>,
    head_block: Option<ObjPtr<BasicBlock>>,
}

impl Function {
    /// 构造一个没有bb和参数的函数
    pub fn new() -> Function {
        Function {
            value: Value::new(IrType::Function),
            return_type: IrType::Void,
            parameters: HashMap::new(),
            head_block: None,
        }
    }

    /// 设置函数的返回类型
    pub fn set_return_type(&mut self, return_type: IrType) {
        self.return_type = return_type;
    }

    /// 获得函数的返回类型
    pub fn get_return_type(&self) -> IrType {
        self.return_type
    }

    /// 判断函数中是否有bb
    pub fn is_empty_bb(&self) -> bool {
        if let None = self.head_block {
            true
        } else {
            false
        }
    }

    /// 将第一个BB加入到函数中
    pub fn insert_first_bb(&mut self, bb: ObjPtr<BasicBlock>) {
        debug_assert_eq!(self.head_block.is_none(), true);
        self.head_block = Some(bb);
    }

    /// 获得第一个块，默认为非空块
    pub fn get_head(&self) -> ObjPtr<BasicBlock> {
        match self.head_block {
            Some(bb) => bb,
            None => panic!("尝试获得一个空的BB"),
        }
    }

    /// 为函数增加参数
    pub fn set_parameter(&mut self, name: &str, parameter: ObjPtr<Inst>) {
        self.parameters.insert(name, parameter);
    }

    /// 获得参数
    /// 默认参数存在
    pub fn get_parameter(&self, name: &str) -> ObjPtr<Inst> {
        match self.parameters.get(name) {
            Some(p) => *p,
            None => panic!("尝试获得不存在的参数"),
        }
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }
}
