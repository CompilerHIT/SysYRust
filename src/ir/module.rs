use super::{function::Function, instruction::Inst};
use crate::utility::ObjPtr;
use std::collections::HashMap;
pub struct Module {
    pub global_variable: HashMap<String, ObjPtr<Inst>>,
    pub function: HashMap<String, ObjPtr<Function>>,
}
impl Module {
    /// 构造一个新的module
    pub fn new() -> Module {
        Module {
            global_variable: HashMap::new(),
            function: HashMap::new(),
        }
    }

    /// 将申请的全局变量放入module中
    pub fn push_var(&mut self, name: String, variable: ObjPtr<Inst>) {
        if let None = self.global_variable.get(&name) {
            self.global_variable.insert(name, variable);
        } else {
            debug_assert!(false);
        }
    }

    /// 将新建的函数加入module中
    pub fn push_function(&mut self, name: String, function: ObjPtr<Function>) {
        if let None = self.function.get(&name) {
            self.function.insert(name, function);
        } else {
            debug_assert!(false);
        }
    }

    /// 根据名字查找变量
    /// 默认先前已经放入module中
    pub fn get_var(&self, name: &str) -> ObjPtr<Inst> {
        if let Some(var) = self.global_variable.get(name) {
            var.clone()
        } else {
            panic!("在定义变量前就使用变量")
        }
    }

    /// 根据名字查找函数
    /// 默认先前已经放入module中
    pub fn get_function(&self, name: &str) -> ObjPtr<Function> {
        if let Some(func) = self.function.get(name) {
            func.clone()
        } else {
            panic!("在定义函数前就使用函数")
        }
    }

    /// 用于遍历所有全局变量
    pub fn get_all_var(&self) -> Vec<(&String, ObjPtr<Inst>)> {
        self.global_variable
            .iter()
            .map(|(name, var)| (name, var.clone()))
            .collect()
    }

    /// 用于遍历所有函数
    pub fn get_all_func(&self) -> Vec<(&String, ObjPtr<Function>)> {
        self.function
            .iter()
            .map(|(name, func)| (name, func.clone()))
            .collect()
    }
}
