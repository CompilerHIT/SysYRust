use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
};

use crate::utility::ObjPtr;

use super::{
    basicblock::BasicBlock,
    function::Function,
    instruction::{Inst, InstKind},
    ir_type::IrType,
    module::Module,
};

pub fn dump_now(module: &Module) {
    let mut global_map = HashMap::new();
    let mut text = String::new();

    // dump global variables
    for (name, var) in module.get_all_var() {
        global_map.insert(var, name.clone());
        text = format!("{}{}", text, dump_global_var(name, var, &mut global_map));
    }

    text += "\n";

    // dump functions
    for (name, func) in module.get_all_func() {
        if func.is_empty_bb() {
            continue;
        }

        text = format!("{}{}\n\n\n", text, dump_func(name, func, &mut global_map));
    }

    // write to file
    let mut file = File::create("dump.ll").unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

fn dump_global_var(
    var_name: &str,
    var: ObjPtr<Inst>,
    global_map: &mut HashMap<ObjPtr<Inst>, String>,
) -> String {
    match var.get_kind() {
        InstKind::GlobalConstInt(value) => {
            format!("@{} = dso_local constant i32 {}, align 4", var_name, value)
        }
        InstKind::GlobalInt(value) => {
            format!("@{} = dso_local global i32 {}, align 4", var_name, value)
        }
        InstKind::GlobalFloat(value) => {
            format!("@{} = dso_local global float {}, align 4", var_name, value)
        }
        InstKind::GlobalConstFloat(value) => {
            format!(
                "@{} = dso_local constant float {}, align 4",
                var_name, value
            )
        }
        InstKind::Alloca(value) => match var.get_ir_type() {
            IrType::IntPtr => {
                let init = var.get_int_init();
                let mut text = format!("@{} = dso_local global ", var_name);
                if value > init.len() as i32 {
                    let mut value_type = String::new();
                    let mut value_init = String::new();
                    for v in init {
                        value_type += " i32,";
                        value_init += format!(" i32 {},", v).as_str();
                    }
                    value_type += format!(" [{} x i32] ", value - init.len() as i32).as_str();
                    value_init +=
                        format!(" [{} x i32] zeroinitializer", value - init.len() as i32).as_str();
                    text = format!("{} <{{{}}}> <{{{}}}", text, value_type, value_init);
                } else {
                    let mut value_init = String::new();
                    for v in init {
                        value_init += format!(" i32 {},", v).as_str();
                    }
                    text = format!("{} [{} x i32] [{}]", text, value, value_init);
                }

                text += ", align 4";

                text
            }
            IrType::FloatPtr => {
                let init = var.get_float_init();
                let mut text = format!("@{} = dso_local global ", var_name);
                if value > init.len() as i32 {
                    let mut value_type = String::new();
                    let mut value_init = String::new();
                    for v in init {
                        value_type += " float,";
                        value_init += format!(" float {},", v).as_str();
                    }
                    value_type += format!(" [{} x float] ", value - init.len() as i32).as_str();
                    value_init +=
                        format!(" [{} x float] zeroinitializer", value - init.len() as i32)
                            .as_str();
                    text = format!("{} <{{{}}}> <{{{}}}", text, value_type, value_init);
                } else {
                    let mut value_init = String::new();
                    for v in init {
                        value_init += format!(" float {},", v).as_str();
                    }
                    text = format!("{} [{} x float] [{}]", text, value, value_init);
                }

                text += ", align 4";

                text
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn dump_func(
    func_name: &str,
    func: ObjPtr<Function>,
    global_map: &mut HashMap<ObjPtr<Inst>, String>,
) -> String {
    let mut local_map = HashMap::new();
    let mut text = String::new();
    let mut name_index = 0;

    // dump function header
    text = format!(
        "define dso_local {} @{}({}) #0 {{\n",
        dump_ir_type(func.get_return_type()),
        func_name,
        dump_parameter(func, &mut local_map, name_index)
    );

    // dump head block
    let bb = func.get_head();
    text = format!(
        "{}{}:\n",
        text,
        dump_block(bb, global_map, &mut local_map, name_index)
    );
    text += "\n";

    // dump other blocks
    // bfs
    let mut queue = Vec::new();
    let mut visited = HashSet::new();
    queue.insert(0, bb);
    visited.insert(bb);
    while let Some(bb) = queue.pop() {
        if !visited.contains(&bb) {
            text = format!(
                "{}{}:\n{}",
                text,
                bb.get_name(),
                dump_block(bb, global_map, &mut local_map, name_index)
            );
            text += "\n";
            visited.insert(bb);
        }
        for succ in bb.get_next_bb() {
            if !visited.contains(&succ) {
                queue.insert(0, succ.clone());
            }
        }
    }

    text += "}\n";

    text
}

fn dump_ir_type(ir_type: IrType) -> String {
    match ir_type {
        IrType::Void => "void".to_string(),
        IrType::Int => "i32".to_string(),
        IrType::ConstInt => "i32".to_string(),
        _ => unreachable!(),
    }
}

fn dump_parameter(
    param: ObjPtr<Function>,
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    name_index: i32,
) -> String {
    todo!()
}

fn dump_block(
    block: ObjPtr<BasicBlock>,
    global_map: &mut HashMap<ObjPtr<Inst>, String>,
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    name_index: i32,
) -> String {
    todo!()
}

fn dump_inst(
    inst: ObjPtr<Inst>,
    global_map: &mut HashMap<ObjPtr<Inst>, String>,
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    name_index: i32,
) -> String {
    todo!()
}
