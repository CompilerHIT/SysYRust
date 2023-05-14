use std::collections::HashMap;

pub struct ActionScope {
    var_map: HashMap<String, Vec<(String, i64)>>,
    index: i64,
    layer: i64,
}
impl ActionScope {
    fn get_var() -> String {
        "1".to_string()
    }
    fn push_var(s: String) {}
    fn delete_layer() {}
    fn has_var_now() -> bool {
        true
    }
    fn add_prefix(s: String) -> String {
        s
    }
}
