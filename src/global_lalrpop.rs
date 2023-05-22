use crate::cfgir::instruction_cfg::binary_inst_cfg;
// use crate::utility::Pointer;
use crate::{cfgir::module_cfg::CfgModule, utility};
use lazy_static::lazy_static;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};
use std::{collections::HashMap, sync::Mutex};

thread_local! {
    pub static MODULE:RefCell<CfgModule> = RefCell::new(CfgModule::make_module());
    pub static IN_FUNC: RefCell<i32> = RefCell::new(0);
}
