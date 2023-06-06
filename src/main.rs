use lalrpop_util::lalrpop_mod;
use sysylib::backend::module::AsmModule;
use sysylib::frontend::irgen::irgen;
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

    // 后端解析
    generate_asm(filename, output, &mut AsmModule::new(&module));
}
#[test]
fn test() {
    // let i: i32 = 42;
    // let f: f32 = i as f32;
    // println!("i32: {}, f32: {}", i, f);
    let file = std::fs::read_to_string("src/a.sy").unwrap();
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
    // println!("{:#?}", compunit);
    let module_ref = module_ptr.as_ref();
    let func_inst = module_ref.get_function("main");
    assert!(!func_inst.as_ref().is_empty_bb());
    let head_bb = func_inst.as_ref().get_head();
    assert!(!head_bb.as_ref().is_empty());
    println!("{:?}", head_bb.as_ref().get_name());
    let mut return_inst = head_bb.as_ref().get_head_inst();
    loop {
        println!("{:?}", return_inst.as_ref().get_kind());
        if return_inst.as_ref().is_tail(){
            println!("{:?}",return_inst.as_ref().get_return_value().as_ref().get_kind());
            break;
        }
        return_inst = return_inst.as_ref().get_next();
    // println!("{:?}", return_inst.as_ref().get_kind());
    }
    println!("global:");
    // let global = module_ref.
    // loop{

    // }
    for (global,var_inst) in &module_ref.global_variable{
        println!("全局变量:{:?},对应指令{:?}",global,var_inst.as_ref().get_kind());
    }
    // assert!(return_inst.as_ref().is_tail());
    // println!("{:#?}", );
}