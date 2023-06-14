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
        global_map.insert(var, format!("@{}", name));
        text = format!("{}{}", text, dump_global_var(name, var));
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

fn dump_global_var(var_name: &str, var: ObjPtr<Inst>) -> String {
    match var.get_kind() {
        InstKind::GlobalConstInt(value) => {
            format!("{} = dso_local constant i32 {}, align 4", var_name, value)
        }
        InstKind::GlobalInt(value) => {
            format!("{} = dso_local global i32 {}, align 4", var_name, value)
        }
        InstKind::GlobalFloat(value) => {
            format!("{} = dso_local global float {}, align 4", var_name, value)
        }
        InstKind::GlobalConstFloat(value) => {
            format!("{} = dso_local constant float {}, align 4", var_name, value)
        }
        InstKind::Alloca(value) => match var.get_ir_type() {
            IrType::IntPtr => {
                let init = var.get_int_init();
                let mut text = format!("{} = dso_local global ", var_name);
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
                let mut text = format!("{} = dso_local global ", var_name);
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
    let mut name_index = 0;

    // dump function header
    let mut text;
    (name_index, text) = dump_parameter(func, &mut local_map, name_index);
    text = format!(
        "define dso_local {} @{}({}) #0 {{\n",
        dump_ir_type(func.get_return_type()),
        func_name,
        text
    );

    // dump head block
    let bb = func.get_head();
    let mut temp;
    (name_index, temp) = dump_block(bb, global_map, &mut local_map, name_index);
    text = format!("{}{}:\n", text, temp);
    text += "\n";

    // dump other blocks
    // bfs
    let mut queue = Vec::new();
    let mut visited = HashSet::new();
    queue.insert(0, bb);
    visited.insert(bb);
    while let Some(bb) = queue.pop() {
        if !visited.contains(&bb) {
            (name_index, temp) = dump_block(bb, global_map, &mut local_map, name_index);
            text = format!("{}{}:\n{}", text, bb.get_name(), temp);
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
        IrType::Int => "signext i32".to_string(),
        IrType::Float => "float".to_string(),
        _ => unreachable!(),
    }
}

fn dump_parameter(
    param: ObjPtr<Function>,
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    mut name_index: i32,
) -> (i32, String) {
    let mut text = String::new();
    for (_, var) in param.get_params().iter() {
        local_map.insert(var.clone(), format!("%{}", name_index));
        text += format!(
            "{} {}, ",
            dump_para_type(var.get_ir_type()),
            local_map.get(&var).unwrap()
        )
        .as_str();
        name_index += 1;
    }
    text.truncate(text.len() - 2);
    (name_index, text)
}

fn dump_para_type(ir_type: IrType) -> String {
    match ir_type {
        IrType::Int => "i32 noundef signext".to_string(),
        IrType::Float => "float noundef".to_string(),
        _ => unreachable!(),
    }
}

fn dump_block(
    block: ObjPtr<BasicBlock>,
    global_map: &mut HashMap<ObjPtr<Inst>, String>,
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    mut name_index: i32,
) -> (i32, String) {
    let mut text = String::new();
    if block.is_empty() {
        (name_index, text)
    } else {
        let mut inst = block.get_head_inst();
        loop {
            let temp;
            (name_index, temp) = dump_inst(inst, global_map, local_map, name_index);
            text = format!("{}{}", text, temp);
            if inst.is_tail() {
                break;
            } else {
                inst = inst.get_next();
            }
        }
        (name_index, text)
    }
}

fn dump_inst(
    inst: ObjPtr<Inst>,
    global_map: &mut HashMap<ObjPtr<Inst>, String>,
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    mut name_index: i32,
) -> (i32, String) {
    let mut text = String::new();
    match inst.get_kind() {
        InstKind::Alloca(len) => {
            if let IrType::IntPtr = inst.get_ir_type() {
                local_map.insert(inst, format!("%{}", name_index));
                text = format!(
                    "  {} = alloca [{} x i32], align 4\n",
                    local_map.get(&inst).unwrap(),
                    len
                );
                name_index += 1;

                // 数组初始化
                text += format!("  ; init array begin!!!!\n").as_str();
                let init = inst.get_int_init();
                for (i, v) in init.iter().enumerate() {
                    text += format!("  %{} = getelementptr inbounds [{} x i32], [{} x i32]* {}, i32 0, i32 {}\n", name_index, len, len, local_map.get(&inst).unwrap(), i).as_str();
                    text += format!("  store i32 {}, i32* %{}, align 4\n", v, name_index).as_str();
                    name_index += 1;
                }

                text += format!("  ; init array end!!!!\n").as_str();
            } else {
                local_map.insert(inst, format!("%{}", name_index));
                text = format!(
                    "  {} = alloca [{} x float], align 4\n",
                    local_map.get(&inst).unwrap(),
                    len
                );

                // 数组初始化
                text += format!("  ; init array begin!!!!\n").as_str();
                let init = inst.get_float_init();
                for (i, v) in init.iter().enumerate() {
                    text += format!("  %{} = getelementptr inbounds [{} x float], [{} x float]* {}, i32 0, i32 {}\n", name_index, len, len, local_map.get(&inst).unwrap(), i).as_str();
                    text +=
                        format!("  store float {}, float* %{}, align 4\n", v, name_index).as_str();
                    name_index += 1;
                }
            }
        }
        InstKind::Gep => {
            let ptr = inst.get_ptr();
            let len = ptr.get_array_length();

            if let IrType::IntPtr = ptr.get_ir_type() {
                local_map.insert(inst, format!("%{}", name_index));
                text += format!(
                    "  %{} = getelementptr inbounds [{} x i32], [{} x i32]* {}, i32 0, i32 {}\n",
                    local_map.get(&inst).unwrap(),
                    len,
                    len,
                    local_map.get(&ptr).unwrap(),
                    get_inst_value(inst, local_map, global_map)
                )
                .as_str();
            } else {
                local_map.insert(inst, format!("%{}", name_index));
                text += format!(
                    "  %{} = getelementptr inbounds [{} x float], [{} x float]* {}, i32 0, i32 {}\n",
                    local_map.get(&inst).unwrap(),
                    len,
                    len,
                    local_map.get(&ptr).unwrap(),
                    get_inst_value(inst, local_map, global_map)
                )
                .as_str()
            }
        }
        InstKind::Load => match inst.get_ir_type() {
            IrType::IntPtr | IrType::FloatPtr => {}
            IrType::Int => {
                local_map.insert(inst, format!("%{}", name_index));
                text += format!(
                    "  {} = load i32, i32* {}, align 4\n",
                    local_map.get(&inst).unwrap(),
                    get_inst_value(inst, local_map, global_map)
                )
                .as_str();
            }
            IrType::Float => {
                local_map.insert(inst, format!("%{}", name_index));
                text += format!(
                    "  {} = load float, float* {}, align 4\n",
                    local_map.get(&inst).unwrap(),
                    get_inst_value(inst, local_map, global_map)
                )
                .as_str();
            }
            _ => unreachable!("No other type in load"),
        },
        InstKind::Store => match inst.get_value().get_ir_type() {
            IrType::Int => {
                text += format!(
                    "  store i32 {}, i32* {}, align 4\n",
                    get_inst_value(inst.get_value(), local_map, global_map),
                    get_inst_value(inst, local_map, global_map)
                )
                .as_str();
            }
            IrType::Float => {
                text += format!(
                    "  store float {}, float* {}, align 4\n",
                    get_inst_value(inst.get_value(), local_map, global_map),
                    get_inst_value(inst, local_map, global_map)
                )
                .as_str();
            }
            _ => unreachable!("No other type in store"),
        },
        InstKind::Binary(op) => {}
        InstKind::Unary(op) => {}
        InstKind::Branch => {}
        InstKind::Call(callee) => {}
        InstKind::Parameter => unreachable!("No parameter in bb"),
        InstKind::Return => {}
        InstKind::FtoI => {}
        InstKind::ItoF => {}
        InstKind::ConstInt(_) | InstKind::ConstFloat(_) => {
            // 常量不处理
        }
        InstKind::GlobalConstInt(_)
        | InstKind::GlobalConstFloat(_)
        | InstKind::GlobalInt(_)
        | InstKind::GlobalFloat(_) => {
            unreachable!("No Global in bb")
        }
        InstKind::Phi => {}
        InstKind::Head(_) => unreachable!("No Head in dump_inst"),
    }
    (name_index, text)
}

fn get_inst_value(
    inst: ObjPtr<Inst>,
    local_map: &HashMap<ObjPtr<Inst>, String>,
    global_map: &HashMap<ObjPtr<Inst>, String>,
) -> String {
    match inst.get_kind() {
        InstKind::ConstInt(value) => value.to_string(),
        InstKind::ConstFloat(value) => value.to_string(),
        InstKind::GlobalConstInt(value) => value.to_string(),
        InstKind::GlobalConstFloat(value) => value.to_string(),
        _ => {
            if local_map.contains_key(&inst) {
                local_map.get(&inst).unwrap().clone()
            } else if global_map.contains_key(&inst) {
                global_map.get(&inst).unwrap().clone()
            } else {
                unreachable!("value not found")
            }
        }
    }
}
