use lalrpop_util::lalrpop_mod;
use sysylib::backend::generate_asm;
use sysylib::backend::module::AsmModule;
use std::collections::HashMap;
use sysylib::frontend::irgen::irgen;
use sysylib::ir::instruction::Inst;
use sysylib::{self, ir::module::Module, utility::ObjPool};
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
    let file = std::fs::read_to_string("src/input.txt").unwrap();
    let mut pool_module = ObjPool::new();
    let module_ptr = pool_module.put(Module::new()); //module的指针
    let module_mut = module_ptr.as_mut();
    let mut pool_inst: ObjPool<Inst> = ObjPool::new();
    let mut pool_inst_mut = &mut pool_inst;
    let mut compunit = SysYRust::CompUnitParser::new()
        .parse(file.as_str())
        .unwrap();
    let mut pool_bb = ObjPool::new();
    let mut pool_bb_mut = &mut pool_bb;
    let mut pool_func = ObjPool::new();
    let mut pool_func_mut = &mut pool_func;
    irgen(
        &mut compunit,
        module_mut,
        pool_inst_mut,
        pool_bb_mut,
        pool_func_mut,
    );
    // TODO 后端解析
    generate_asm(&mut AsmModule::new(module_mut));
}
