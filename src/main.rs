use lalrpop_util::lalrpop_mod;
use sysylib::config;
use sysylib::frontend::preprocess::preprocess;
extern crate biheap;
// extern crate hexf_parse;
// extern crate libm;
use sysylib::backend::module::AsmModule;
use sysylib::frontend::irgen::irgen;
use sysylib::ir::dump_now;
use sysylib::ir::instruction::Inst;
use sysylib::{self, backend::generate_asm, ir::module::Module, utility::ObjPool};
lalrpop_mod! {
  #[allow(clippy::all)]
  SysYRust
}
fn main() {
    run_main();
}

fn run_main() {
    // let m=LinkedList::new();
    // let cursor=m.cursor_front_mut();
    // ---------------------测试代码---------------------
    #[cfg(debug_assertions)]
    {
        use std::env;
        env::set_var("RUST_BACKTRACE", "1");
        println!("debug mode");
    }
    // --------------------------------------------------
    use clap::{App, Arg};
    // 获取命令行解析
    let matches = App::new("compiler")
        .arg(Arg::with_name("filename").required(true))
        .arg(Arg::with_name("S").short("S"))
        .arg(Arg::with_name("o").short("o").takes_value(true))
        .arg(Arg::with_name("O1").short("O").takes_value(true))
        .get_matches();

    // 获取文件名
    let filename = matches.value_of("filename").unwrap();

    crate::config::init();
    crate::config::set_file_path(&String::from(filename)); //把函数名加载到全局

    // 生成汇编的标志
    let _s_option = matches.is_present("S");
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

    let file_preprocessed = preprocess(file.as_str());

    let mut compunit = SysYRust::CompUnitParser::new()
        .parse(file_preprocessed.as_str())
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
    sysylib::ir::optimizer_run(&mut module, (&mut pool_bb, &mut pool_inst), true);
    let output2 = "row_asm.log";

    // dump_now(&module, "dump.ll");

    // 后端解析
    let is_opt = o1_option;
    // let is_opt = true;
    // let is_opt = false;
    generate_asm(
        filename,
        output,
        output2,
        &mut AsmModule::new(module),
        is_opt,
    );

    // 编译结束后打印记录的属性
    config::dump();
}
