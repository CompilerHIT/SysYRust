pub mod ast;
pub mod backend;
pub mod cfgir;
pub mod global_lalrpop;
pub mod ir;
pub mod tests;
pub mod utility;

// TODO: to add call for generate and new module.
// for example: asm_module = AsmModule::new(ir_module);

// #[cfg(test)]
// mod tests {
//     lalrpop_util::lalrpop_mod!(SysYRust);
//     #[test]
//     fn global_variable_test() {
//         let result = 2 + 2;
//         // sysy::CompUnitParser::new();
//         let comp_unit = SysYRust::CompUnitParser::new().parse("int a;");
//         assert_eq!(result, 4);
//         // "int".to_string();
//     }
// }
