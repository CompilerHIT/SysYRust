use super::{function::Function, instruction::Inst};
use std::collections::HashMap;
pub struct Module {
    pub global_variable: HashMap<&'static str, &'static Inst>,
    pub function: HashMap<&'static str, &'static Function>,
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
    pub fn push_var(&mut self, name: &str, variable: &'static Inst) {
        if let None = self.global_variable.get(name) {
            self.global_variable.insert(name, variable);
        } else {
            debug_assert!(false);
        }
    }

    /// 将新建的函数加入module中
    pub fn push_function(&mut self, name: &str, function: &'static Function) {
        if let None = self.function.get(name) {
            self.function.insert(name, function);
        } else {
            debug_assert!(false);
        }
    }

    /// 根据名字查找变量
    /// 默认先前已经放入module中
    pub fn get_var(&self, name: &str) -> &Inst {
        if let Some(var) = self.global_variable.get(name) {
            var
        } else {
            panic!("在定义变量前就使用变量")
        }
    }

    /// 根据名字查找函数
    /// 默认先前已经放入module中
    pub fn get_function(&self, name: &str) -> &Function {
        if let Some(func) = self.function.get(name) {
            func
        } else {
            panic!("在定义函数前就使用函数")
        }
    }

    /// 用于遍历所有函数
    pub fn get_all_func(&self) -> Vec<(&str, &Function)> {
        self.function.into_iter().collect()
    }
}
