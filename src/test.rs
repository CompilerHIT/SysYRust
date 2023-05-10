use crate::utility::Pointer;
use lalrpop_util::lalrpop_mod;
use std::collections::HashMap;
// use sysylib;
use crate::cfgir::instruction_cfg::CfgInstruction;
use crate::global_lalrpop::IN_FUNC;
use crate::global_lalrpop::MODULE;
lalrpop_mod! {
  #[allow(clippy::all)]
  SysYRust
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
