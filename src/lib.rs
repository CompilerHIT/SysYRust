pub mod ast;
pub mod backend;
pub mod cfgir;
pub mod global_lalrpop;
pub mod ir;
pub mod test;
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
//

pub fn run_main() {
    use clap::{App, Arg};
    // 获取命令行解析
    let matches = App::new("compiler")
        .arg(Arg::with_name("filename").required(true))
        .arg(Arg::with_name("S").short("S"))
        .arg(Arg::with_name("o").short("o").takes_value(true))
        .arg(Arg::with_name("O1").long("O1"))
        .get_matches();

    // 获取文件名
    let filename = matches.value_of("filename").unwrap();

    // 生成汇编的标志
    let s_option = matches.is_present("S");
    // 输出文件名
    let output = matches.value_of("o").unwrap_or("testcase.s");

    // 是否使用优化
    let o1_option = matches.is_present("O1");

    // 读取文件
    let file = std::fs::read_to_string(filename).unwrap();

    // TODO 生成IR

    // TODO 后端解析
}
