use lalrpop_util::lalrpop_mod;
use sysylib;
use sysylib::global_lalrpop::IN_FUNC;
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
    // sysy::CompUnitParser::new();
    let comp_unit = SysYRust::CompUnitParser::new().parse("int a;");
    assert_eq!(result, 4);
    // "int".to_string();
}
