use crate::sysylib::utility::Pointer;
use std::collections::HashMap;

use lalrpop_util::lalrpop_mod;
use sysylib;
use sysylib::cfgir::instruction_cfg::CfgInstruction;
use sysylib::global_lalrpop::IN_FUNC;
use sysylib::global_lalrpop::MODULE;
lalrpop_mod! {
  #[allow(clippy::all)]
  SysYRust
}
fn main() {
    println!("Hello, world!");
}
#[test]
fn global_variable_test() {
    let result = 2 + 2;
    let comp_unit = SysYRust::CompUnitParser::new().parse("int a;int b; int c,d,e;");
    MODULE.with(|foo| {
        let mut valtemp = foo.borrow_mut();
        let mut tt = &mut valtemp.global_variable;
        print_hashmap(tt);
    });
    assert_eq!(result, 4);
    // "int".to_string();
}

fn print_hashmap(map: &mut HashMap<String, Pointer<Box<dyn CfgInstruction>>>) {
    for (key, value) in map.iter() {
        println!("{} / {}", key, 0);
    }
}
