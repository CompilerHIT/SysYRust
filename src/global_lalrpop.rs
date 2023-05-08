use crate::cfgir::instruction_cfg::binary_inst_cfg;
use crate::utility::Pointer;
use crate::{cfgir::module_cfg::CfgModule, utility};
use lazy_static::lazy_static;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};
use std::{collections::HashMap, sync::Mutex};
// pub mod global_lalrpop {
//     // lazy_static! {
//     //     static ref MODULE:CfgModule =
//     //         CfgModule::make_module();
//     //     // static ref IN_FUNC:int =0;
//     //         // let mut inFunc = 0;

//     // }

// }
// lazy_static! {
// static ref MODULE: CfgModule = CfgModule::make_module();
// static ref MODULE: Mutex<CfgModule> = Mutex::new(CfgModule::make_module());
// static ref MODULE: CfgModule = CfgModule::make_module();
// static ref HASHMAP:HashMap<String,String> = HashMap::new();
// static ref MODULE:utility::Pointer<dyn crate::cfgir::instruction_cfg::CfgInstruction> = binary_inst_cfg::CfgBinaryOpInst::make_add_inst();
// static ref MY_GLOBAL_VAR: Rc<String> = Rc::new("Hello, world!".to_string());
// static ref MY_GLOBAL_VAR: Mutex<Rc<String>> = Mutex::new(Rc::new("Hello, world!".to_string()));
// }
thread_local! {
    pub static MODULE:RefCell<CfgModule> = RefCell::new(CfgModule::make_module());
    pub static IN_FUNC: RefCell<i32> = RefCell::new(0);
}

// pub static IN_FUNC: i32 = 0;
// lazy_static! {
//     pub static ref IN_FUNC: i32 = 0;
// }
// static mut MODULE: module_cfg::CfgModule = module_cfg::CfgModule::make_module();
