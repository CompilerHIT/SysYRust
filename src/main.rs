use lalrpop_util::lalrpop_mod;
use std::collections::HashMap;
use sysylib;
use sysylib::cfgir::instruction_cfg::CfgInstruction;
use sysylib::global_lalrpop::IN_FUNC;
use sysylib::global_lalrpop::MODULE;
use sysylib::utility::Pointer;
lalrpop_mod! {
  #[allow(clippy::all)]
  SysYRust
}
fn main() {
    run_main();
}

fn run_main() {
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
    let mut compunit = SysYRust::CompUnitParser::new().parse(file.as_str());

    // TODO 后端解析
}
