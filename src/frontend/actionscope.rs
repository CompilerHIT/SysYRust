use std::collections::HashMap;

pub struct ActionScope {
    var_map: HashMap<String, Vec<(String, i64)>>,
    index: i64,
    layer: i64,
    symbol_table: HashMap<String, Symbol>,
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
            index: 0,
            layer: 0,
            symbol_table: HashMap::new(),
        }
    }
    pub fn get_var() -> String {
        "1".to_string()
    }
    pub fn push_var(s: String) {}
    pub fn delete_layer() {}
    pub fn has_var_now() -> bool {
        true
    }
    pub fn add_prefix(s: String) -> String {
        s
    }
}
