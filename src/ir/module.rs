use super::{function::Function, instruction::Inst};
use crate::utility::ObjPtr;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Module {
    global_variable: HashMap<String, ObjPtr<Inst>>,
    function: Vec<(String, ObjPtr<Function>)>,
}
impl Module {
    /// 构造一个新的module
    pub fn new() -> Module {
        Module {
            global_variable: HashMap::new(),
            function: Vec::new(),
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
        debug_assert_ne!(self.function.iter().position(|(n, _)| n == &name), None);
        self.function.push((name, function));
    }

    /// 删除函数
    pub fn delete_function(&mut self, name: &str) {
        let index = self.function.iter().position(|(n, _)| n == name).unwrap();
        self.function.remove(index);
    }

    /// 删除全局变量
    pub fn delete_var(&mut self, name: &str) {
        self.global_variable.remove(name);
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
        self.function
            .iter()
            .find(|(n, _)| n == &name)
            .unwrap()
            .1
            .clone()
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

    /// 用于删除全局变量
    pub fn remove_var(&mut self, name: &str) {
        self.global_variable.remove(name);
    }
}
