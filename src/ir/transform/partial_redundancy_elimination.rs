use crate::ir::module::Module;

use super::global_value_numbering;

pub fn pre(module: &mut Module, opt_option: bool){
    let congruence_class = global_value_numbering::gvn(module,opt_option).unwrap();
    
}