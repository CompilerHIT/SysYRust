use std::collections::HashMap;
use std::env;

use lalrpop_util::lalrpop_mod;
use sysylib::backend::module::AsmModule;
use sysylib::frontend::irgen::irgen;
use sysylib::ir::basicblock::BasicBlock;
use sysylib::ir::instruction::{Inst, InstKind};
use sysylib::utility::ObjPtr;
use sysylib::{self, backend::generate_asm, ir::module::Module, utility::ObjPool};
lalrpop_mod! {
  #[allow(clippy::all)]
  SysYRust
}
fn main() {
    run_main();
}

fn run_main() {
    // ---------------------测试代码---------------------
    env::set_var("RUST_BACKTRACE", "1");
    // --------------------------------------------------
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

    // 生成IR

    let mut module = Module::new(); //module的指针

    let mut pool_func = ObjPool::new();

    let mut pool_bb = ObjPool::new();

    let mut pool_inst: ObjPool<Inst> = ObjPool::new();

    let mut compunit = SysYRust::CompUnitParser::new()
        .parse(file.as_str())
        .unwrap();

    irgen(
        &mut compunit,
        &mut module,
        &mut pool_inst,
        &mut pool_bb,
        &mut pool_func,
    );

    drop(compunit);
    // ir优化
    sysylib::ir::optimizer_run(&mut module, o1_option);

    // 后端解析
    generate_asm(filename, output, &mut AsmModule::new(&module));
}
