use crate::utility::{ObjPool, ObjPtr};

use super::{basicblock::BasicBlock, instruction::Inst, ir_type::IrType, value::Value};
use std::collections::HashMap;

pub struct Function {
    value: Value,
    return_type: IrType,
    parameters: HashMap<String, ObjPtr<Inst>>,
    index: Vec<ObjPtr<Inst>>,
    head_block: Option<ObjPtr<BasicBlock>>,
}

impl ObjPool<Function> {
    /// 创建一个新的函数
    pub fn new_function(&mut self) -> ObjPtr<Function> {
        self.put(Function::new())
    }
}

impl Function {
    /// 构造一个没有bb和参数的函数
    pub fn new() -> Function {
        Function {
            value: Value::new(IrType::Function),
            return_type: IrType::Void,
            parameters: HashMap::new(),
            index: Vec::new(),
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
        debug_assert_eq!(self.head_block.is_none(), true, "函数中已经有了BB");
        self.head_block = Some(bb);
    }

    /// 获得第一个块，默认为非空块
    pub fn get_head(&self) -> ObjPtr<BasicBlock> {
        debug_assert_ne!(self.head_block.is_none(), true, "函数中没有BB");
        match self.head_block {
            Some(bb) => bb,
            None => panic!("尝试获得一个空的BB"),
        }
    }

    /// 为函数增加参数
    pub fn set_parameter(&mut self, name: String, parameter: ObjPtr<Inst>) {
        self.parameters.insert(name, parameter);
        self.index.push(parameter);
    }

    /// 获得按顺序排列的参数
    /// 默认参数存在
    pub fn get_parameter_list(&self) -> &Vec<ObjPtr<Inst>> {
        &self.index
    }

    /// 获得参数
    /// 默认参数存在
    pub fn get_parameter(&self, name: &String) -> ObjPtr<Inst> {
        match self.parameters.get(name) {
            Some(p) => *p,
            None => panic!("尝试获得不存在的参数"),
        }
    }

    pub fn get_params(&self) -> &HashMap<String, ObjPtr<Inst>> {
        &self.parameters
    }

    pub fn get_ir_type(&self) -> IrType {
        self.value.get_ir_type()
    }
}
