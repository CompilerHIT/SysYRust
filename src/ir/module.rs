use super::{function::Function, instruction::Inst};
use crate::utility::ObjPtr;

#[derive(Clone)]
pub struct Module {
    global_variable: Vec<(String, ObjPtr<Inst>)>,
    function: Vec<(String, ObjPtr<Function>)>,
}
impl Module {
    /// 构造一个新的module
    pub fn new() -> Module {
        Module {
            global_variable: Vec::new(),
            function: Vec::new(),
        }
    }

    /// 将申请的全局变量放入module中
    pub fn push_var(&mut self, name: String, variable: ObjPtr<Inst>) {
        debug_assert_eq!(
            self.global_variable.iter().position(|(n, _)| n == &name),
            None
        );
        self.global_variable.push((name, variable));
    }

    /// 将新建的函数加入module中
    pub fn push_function(&mut self, name: String, function: ObjPtr<Function>) {
        debug_assert_eq!(self.function.iter().position(|(n, _)| n == &name), None);
        self.function.push((name, function));
    }

    /// 删除函数
    pub fn delete_function(&mut self, name: &str) {
        let index = self.function.iter().position(|(n, _)| n == name).unwrap();
        self.function.remove(index);
    }

    /// 删除全局变量
    pub fn delete_var(&mut self, name: &str) {
        let index = self
            .global_variable
            .iter()
            .position(|(n, _)| n == name)
            .unwrap();
        self.global_variable.remove(index);
    }

    /// 根据名字查找变量
    /// 默认先前已经放入module中
    pub fn get_var(&self, name: &str) -> ObjPtr<Inst> {
        self.global_variable
            .iter()
            .find(|(n, _)| n == &name)
            .unwrap()
            .1
            .clone()
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
}
