use core::panic;
use std::collections::HashMap;

use crate::{ir::instruction::Inst, utility::ObjPtr};

pub struct ActionScope {
    var_map: HashMap<String, Vec<(String, i64)>>,
    symbol_table: HashMap<String, Symbol>,
    bb_map: HashMap<String, HashMap<String, ObjPtr<Inst>>>,
    index: i64,
    layer: i64,
}

pub enum Type {
    Int,
    Float,
}

pub struct Symbol {
    tp: Type,
    is_array: bool,
    dimension: Vec<i64>,
}

impl ActionScope {
    pub fn new() -> ActionScope {
        ActionScope {
            var_map: HashMap::new(),
            bb_map: HashMap::new(),
            index: 0,
            layer: 0,
            symbol_table: HashMap::new(),
        }
    }
    pub fn get_var(&self, s: &str, bbname: &str) -> Option<(ObjPtr<Inst>, &Symbol)> {
        let mut name_;
        if let Some(vec_temp) = self.var_map.get(s.clone()) {
            if let Some((last_element0, _last_element1)) = vec_temp.last() {
                name_ = last_element0.clone();
            }
        }
        let symbol;
        if let Some(symbol_temp) = self.symbol_table.get(&name_) {
            symbol = symbol_temp.clone();
        }
        if let Some(var_inst_map) = self.bb_map.get(bbname) {
            if let Some(inst) = var_inst_map.get(s.clone()) {
                let ret_inst = inst.clone();
                return Option::Some((ret_inst, symbol));
            }
        }
        Option::None
    }
    // pub fn push_var(
    //     &self,
    //     s: &str,
    //     tp: Type,
    //     is_array: bool,
    //     dimension: Vec<i64>,
    //     bbname: &str,
    //     inst: ObjPtr<Inst>,
    // ) -> bool {
    //     let s1 = s.clone();
    //     if (self.has_var_now(s1)) {
    //         return false;
    //     }
    //     if self.var_map.contains_key(s) {
    //         if let Some(vec) = self.var_map.get_mut(s) {
    //             let temps = self.add_prefix(s.to_string()).as_str();
    //             vec.push((temps.to_string(), self.layer));
    //             self.symbol_table.insert(
    //                 temps.clone().to_string(),
    //                 Symbol {
    //                     tp,
    //                     is_array,
    //                     dimension,
    //                 },
    //             );
    //             if let Some(inst_map) = self.bb_map.get_mut(bbname) {
    //                 inst_map.insert(temps.to_string(), inst);
    //             } else {
    //                 let mut map = HashMap::new();
    //                 map.insert(temps.to_string(), inst);
    //                 self.bb_map.insert(bbname.to_string(), map);
    //             }
    //         }
    //     } else {
    //         let mut v = vec![];
    //         let temps = self.add_prefix(s.to_string());
    //         v.push((temps, self.layer));
    //         self.var_map.insert(s.to_string(), v);
    //         self.symbol_table.insert(
    //             temps.clone(),
    //             Symbol {
    //                 tp,
    //                 is_array,
    //                 dimension,
    //             },
    //         );
    //         if let Some(inst_map) = self.bb_map.get(bbname) {
    //             inst_map.insert(temps.to_string(), inst);
    //         } else {
    //             let mut map = HashMap::new();
    //             map.insert(temps, inst);
    //             self.bb_map.insert(bbname.to_string(), map);
    //         }
    //     }
    //     true
    // }

    pub fn update_var(&self, s: &str, bbname: &str, inst: ObjPtr<Inst>) -> bool {
        if self.var_map.contains_key(s) {
            if let Some(vec) = self.var_map.get_mut(s) {
                // let temps = self.add_prefix(s.to_string()).as_str()
                if let Some(tempvar) = vec.last() {
                    let temps = tempvar.0.clone();
                    if let Some(inst_map) = self.bb_map.get_mut(bbname) {
                        inst_map.insert(temps, inst);
                    } else {
                        let mut map = HashMap::new();
                        map.insert(temps, inst);
                        self.bb_map.insert(bbname.to_string(), map);
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn add_var(&self, s: &str, tp: Type, is_array: bool, dimension: Vec<i64>) -> bool {
        let s1 = s.clone();
        if (self.has_var_now(s1)) {
            return false;
        }
        if self.var_map.contains_key(s) {
            if let Some(vec) = self.var_map.get_mut(s) {
                let temps = self.add_prefix(s.to_string()).as_str();
                vec.push((temps.to_string(), self.layer));
                self.symbol_table.insert(
                    temps.clone().to_string(),
                    Symbol {
                        tp,
                        is_array,
                        dimension,
                    },
                );
            }
        } else {
            let mut v = vec![];
            let temps = self.add_prefix(s.to_string());
            v.push((temps, self.layer));
            self.var_map.insert(s.to_string(), v);
            self.symbol_table.insert(
                temps.clone(),
                Symbol {
                    tp,
                    is_array,
                    dimension,
                },
            );
        }
        true
    }

    pub fn add_layer(&self) {
        self.layer = self.layer + 1;
    }

    pub fn delete_layer(&self) {
        //todo:遍历所有变量，删除layer==layer_now的所有变量
        for (key, mut vec) in self.var_map {
            let mut index_now = 0;
            for (name_changed, layer_) in vec {
                if layer_ == self.layer {
                    vec.remove(index_now);
                    self.symbol_table.remove(&name_changed);
                    for (bbname, mut inst_map) in self.bb_map {
                        inst_map.remove(&name_changed);
                    }
                    break;
                }
                index_now = index_now + 1;
            }
        }
        self.layer = self.layer - 1;
    }

    pub fn has_var_now(&self, s: &str) -> bool {
        if let Some(vec_temp) = self.var_map.get(s) {
            for i in vec_temp {
                if i.1 == self.layer {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_layer(&self) -> i64 {
        return self.layer;
    }

    pub fn add_prefix(&self, s: String) -> String {
        self.index = self.index + 1;
        self.index.to_string() + s.as_str()
    }
}
