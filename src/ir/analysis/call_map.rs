use core::fmt;
use std::collections::{HashMap, HashSet};

use crate::{
    ir::{
        basicblock::BasicBlock,
        instruction::InstKind,
        module::Module,
        tools::{bfs_inst_process, func_process},
    },
    utility::ObjPtr,
};

pub struct CallMap {
    call_map: HashMap<String, HashSet<String>>,
}

impl CallMap {
    pub fn get_succs(&self, func_name: &str) -> &HashSet<String> {
        self.call_map.get(func_name).unwrap()
    }

    pub fn delete_edge(&mut self, caller: &str, callee: &str) {
        self.call_map.get_mut(caller).unwrap().remove(callee);
    }

    pub fn add_edge(&mut self, caller: &str, callee: &str) {
        self.call_map
            .get_mut(caller)
            .unwrap()
            .insert(callee.to_string());
    }

    pub fn delete_func(&mut self, func_name: &str) {
        self.call_map.remove(func_name);
    }

    pub fn find_predecessors(&self, func_name: &str) -> Vec<String> {
        let mut predecessors = Vec::new();
        for (caller, callees) in self.call_map.iter() {
            if callees.contains(func_name) {
                predecessors.push(caller.clone());
            }
        }
        predecessors
    }

    pub fn contains_edge(&self, caller: &str, callee: &str) -> bool {
        self.call_map
            .get(caller)
            .unwrap()
            .contains(&callee.to_string())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &HashSet<String>)> {
        self.call_map.iter()
    }

    pub fn get_all_func(&self) -> Vec<String> {
        self.call_map.keys().cloned().collect()
    }
}

impl fmt::Display for CallMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (caller, callees) in &self.call_map {
            write!(f, "{} -> ", caller)?;
            for callee in callees {
                write!(f, "{} ", callee)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub fn call_map_gen(module: &mut Module) -> CallMap {
    let mut call_map = HashMap::new();

    // 先找出外部函数
    let mut extern_func = HashSet::new();
    for (name, func) in module.get_all_func() {
        if func.is_empty_bb() {
            extern_func.insert(name.clone());
        }
    }

    // 对每个非外部函数，构造调用边
    func_process(module, |name, func| {
        let call_set = call_set_gen(&extern_func, func.get_head());
        call_map.insert(name.clone(), call_set);
    });
    CallMap { call_map }
}

fn call_set_gen(extern_func: &HashSet<String>, head_bb: ObjPtr<BasicBlock>) -> HashSet<String> {
    let mut call_set = HashSet::new();
    bfs_inst_process(head_bb, |inst| {
        if let InstKind::Call(callee) = inst.get_kind() {
            if !extern_func.contains(&callee) {
                call_set.insert(callee);
            }
        }
    });

    call_set
}
