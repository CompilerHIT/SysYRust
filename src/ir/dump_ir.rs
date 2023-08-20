use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Write,
};

use crate::utility::ObjPtr;

use super::{
    basicblock::BasicBlock,
    function::Function,
    instruction::{BinOp, Inst, InstKind, UnOp},
    ir_type::IrType,
    module::Module,
};

pub fn dump_now(module: &Module, path: &str) {
    #[cfg(debug_assertions)]
    {
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

        // dump extern functions
        text += format!("{}\n", dump_external_func()).as_str();

        // write to file
        let mut file = File::create(path).unwrap();
        file.write_all(text.as_bytes()).unwrap();
    }
}

fn dump_global_var(var_name: &str, var: ObjPtr<Inst>) -> String {
    match var.get_kind() {
        InstKind::GlobalConstInt(value) => {
            format!(
                "@{} = dso_local constant i32 {}, align 4\n",
                var_name, value
            )
        }
        InstKind::GlobalInt(value) => {
            format!("@{} = dso_local global i32 {}, align 4\n", var_name, value)
        }
        InstKind::GlobalFloat(value) => {
            format!(
                "@{} = dso_local global float {}, align 4\n",
                var_name, value
            )
        }
        InstKind::GlobalConstFloat(value) => {
            format!(
                "@{} = dso_local constant float {}, align 4\n",
                var_name, value
            )
        }
        InstKind::Alloca(value) => match var.get_ir_type() {
            IrType::IntPtr => {
                let init = var.get_int_init();
                let mut text = format!("@{} = dso_local global ", var_name);
                if value > init.1.len() as i32 {
                    let mut value_type = String::new();
                    let mut value_init = String::new();
                    for v in init.1.iter() {
                        value_type += " i32,";
                        value_init += format!(" i32 {},", v.1).as_str();
                    }
                    value_type += format!(" [{} x i32] ", value - init.1.len() as i32).as_str();
                    value_init +=
                        format!(" [{} x i32] zeroinitializer", value - init.1.len() as i32)
                            .as_str();
                    text = format!("{} <{{{}}}> <{{{}}}>", text, value_type, value_init);
                } else {
                    let mut value_init = String::new();
                    for v in init.1.iter() {
                        value_init += format!(" i32 {},", v.1).as_str();
                    }
                    value_init.truncate(value_init.len() - 1);
                    text = format!("{} [{} x i32] [{}]", text, value, value_init);
                }

                text += ", align 4\n";

                text
            }
            IrType::FloatPtr => {
                let init = var.get_float_init();
                let mut text = format!("@{} = dso_local global ", var_name);
                if value > init.1.len() as i32 {
                    let mut value_type = String::new();
                    let mut value_init = String::new();
                    for v in init.1.iter() {
                        value_type += " float,";
                        value_init += format!(" float {},", v.1).as_str();
                    }
                    value_type += format!(" [{} x float] ", value - init.1.len() as i32).as_str();
                    value_init +=
                        format!(" [{} x float] zeroinitializer", value - init.1.len() as i32)
                            .as_str();
                    text = format!("{} <{{{}}}> <{{{}}}", text, value_type, value_init);
                } else {
                    let mut value_init = String::new();
                    for v in init.1.iter() {
                        value_init += format!(" float {},", v.1).as_str();
                    }
                    text = format!("{} [{} x float] [{}]", text, value, value_init);
                }

                text += ", align 4\n";

                text
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

fn dump_external_func() -> String {
    let mut ext_fun = Vec::new();
    // IO functions
    ext_fun.push("declare i32 @getint()\n");
    ext_fun.push("declare i32 @getch()\n");
    ext_fun.push("declare float @getfloat()\n");
    ext_fun.push("declare void @getarray(ptr)\n");
    ext_fun.push("declare void @getfarray(ptr)\n");
    ext_fun.push("declare void @putint(i32)\n");
    ext_fun.push("declare void @putch(i32)\n");
    ext_fun.push("declare void @putfloat(float)\n");
    ext_fun.push("declare void @putarray(i32, ptr)\n");
    ext_fun.push("declare void @putfarray(i32, ptr)\n");
    ext_fun.push("declare void @putf(i8*, i32, ... )\n");

    // time functions
    ext_fun.push("declare i32 @starttime()\n");
    ext_fun.push("declare i32 @stoptime()\n");
    ext_fun.push("declare void @_sysy_starttime()\n");
    ext_fun.push("declare void @_sysy_stoptime()\n");

    // functions interface
    ext_fun.push("declare void @hitsz_thread_init()\n");
    ext_fun.push("declare void @hitsz_thread_exit()\n");
    ext_fun.push("declare i32 @hitsz_thread_create()\n");
    ext_fun.push("declare void @hitsz_thread_join()\n");
    ext_fun.push("declare void @hitsz_get_thread_num()\n");
    ext_fun.push("declare void @hitsz_memset(ptr, i32, i32)\n");
    ext_fun.push("declare void @hitsz_memcopy(ptr,ptr,i32)\n");

    let mut text = String::new();
    text += "; External Functions\n";
    for fun in ext_fun {
        text += fun;
    }
    text += "; End External Functions\n";
    text += "\n";
    text
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
    text = format!("{}{}:\n{}\n", text, format!("bb_{}", bb.get_name()), temp);
    text += "\n";

    if bb.has_next_bb() {
        // dump other blocks
        // bfs
        let mut queue = Vec::new();
        let mut visited = HashSet::new();
        queue.insert(0, bb);
        visited.insert(bb);
        while let Some(bb) = queue.pop() {
            if !visited.contains(&bb) {
                (name_index, temp) = dump_block(bb, global_map, &mut local_map, name_index);
                text = format!("{}{}:\n{}", text, format!("bb_{}", bb.get_name()), temp);
                text += "\n";
                visited.insert(bb);
            }
            for succ in bb.get_next_bb() {
                if !visited.contains(&succ) {
                    queue.insert(0, succ.clone());
                }
            }
        }
    } else {
        text.truncate(text.len() - 3);
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
    for var in param.get_parameter_list().iter() {
        name_index = put_name(local_map, var.clone(), name_index);
        text += format!(
            "{} {}, ",
            dump_para_type(var.get_ir_type()),
            local_map.get(&var).unwrap()
        )
        .as_str();
    }
    if param.get_parameter_list().len() > 0 {
        text.truncate(text.len() - 2);
    }
    (name_index, text)
}

fn dump_para_type(ir_type: IrType) -> String {
    match ir_type {
        IrType::Int => "i32 noundef signext".to_string(),
        IrType::Float => "float noundef".to_string(),
        IrType::IntPtr | IrType::FloatPtr => "ptr noundef".to_string(),
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
    let mut inst = block.get_head_inst();
    while !inst.is_tail() {
        let temp;
        (name_index, temp) = dump_inst(inst, global_map, local_map, name_index);
        text = format!("{}{}", text, temp);
        inst = inst.get_next();
    }
    (name_index, text)
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
                name_index = put_name(local_map, inst.clone(), name_index);
                text = format!(
                    "  {} = alloca [{} x i32], align 4\n",
                    local_map.get(&inst).unwrap(),
                    len
                );

                // 数组初始化
                text += format!("  ; init array begin!!!!\n").as_str();
                let init = inst.get_int_init();
                for (i, v) in init.1.iter().enumerate() {
                    text += format!("  %val_{} = getelementptr inbounds [{} x i32], [{} x i32]* {}, i32 0, i32 {}\n", name_index, len, len, local_map.get(&inst).unwrap(), i).as_str();
                    text += format!("  store i32 {}, i32* %val_{}, align 4\n", v.1, name_index)
                        .as_str();
                    name_index += 1;
                }
                text += format!("  ; init array end!!!!\n").as_str();
            } else {
                name_index = put_name(local_map, inst.clone(), name_index);
                text = format!(
                    "  {} = alloca [{} x float], align 4\n",
                    local_map.get(&inst).unwrap(),
                    len
                );

                // 数组初始化
                text += format!("  ; init array begin!!!!\n").as_str();
                let init = inst.get_float_init();
                for (i, v) in init.1.iter().enumerate() {
                    text += format!("  %{} = getelementptr inbounds [{} x float], [{} x float]* {}, i32 0, i32 {}\n", name_index, len, len, local_map.get(&inst).unwrap(), i).as_str();
                    text += format!("  store float {}, float* %{}, align 4\n", v.1, name_index)
                        .as_str();
                }
            }
        }
        InstKind::Gep => {
            let ptr;
            let name;
            name_index = put_name(local_map, inst, name_index);
            if let InstKind::Load = inst.get_gep_ptr().get_kind() {
                ptr = inst.get_gep_ptr().get_ptr();
                name = global_map.get(&ptr).unwrap().clone();
            } else {
                ptr = inst.get_gep_ptr();
                name = get_inst_value(ptr, local_map, global_map);
            };
            if let InstKind::Parameter = ptr.get_kind() {
                if let IrType::IntPtr = ptr.get_ir_type() {
                    text += format!(
                        "  {} = getelementptr inbounds i32, ptr {}, i32 {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        name,
                        get_inst_value(inst.get_gep_offset(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    text += format!(
                        "  {} = getelementptr inbounds float, ptr {}, i32 {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        name,
                        get_inst_value(inst.get_gep_offset(), local_map, global_map)
                    )
                    .as_str();
                }
            } else if let InstKind::Gep = ptr.get_kind() {
                if let IrType::IntPtr = ptr.get_ir_type() {
                    text += format!(
                        "  {} = getelementptr i32, i32* {}, i32 {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        name,
                        get_inst_value(inst.get_gep_offset(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    text += format!(
                        "  {} = getelementptr float, float* {}, i32 {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        name,
                        get_inst_value(inst.get_gep_offset(), local_map, global_map)
                    )
                    .as_str();
                }
            } else {
                let len = ptr.get_array_length();

                if let IrType::IntPtr = ptr.get_ir_type() {
                    text += format!(
                        "  {} = getelementptr inbounds [{} x i32], [{} x i32]* {}, i32 0, i32 {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        len,
                        len,
                        name,
                        get_inst_value(inst.get_gep_offset(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    text += format!(
                    "  {} = getelementptr inbounds [{} x float], [{} x float]* {}, i32 0, i32 {}\n",
                    local_map.get(&inst).unwrap().clone(),
                    len,
                    len,
                    name,
                    get_inst_value(inst.get_gep_offset(), local_map, global_map)
                )
                    .as_str();
                }
            }
        }
        InstKind::Load => match inst.get_ir_type() {
            IrType::IntPtr | IrType::FloatPtr => {
                text += format!(
                    "  ; Load array label {}\n",
                    global_map.get(&inst.get_ptr()).unwrap()
                )
                .as_str();
            }
            IrType::Int => {
                name_index = put_name(local_map, inst, name_index);
                let ptr = inst.get_ptr();
                let ptr_name = if let Some(name) = local_map.get(&ptr) {
                    name
                } else {
                    global_map.get(&ptr).unwrap()
                };
                text += format!(
                    "  {} = load i32, i32* {}, align 4\n",
                    local_map.get(&inst).unwrap(),
                    ptr_name
                )
                .as_str();
            }
            IrType::Float => {
                name_index = put_name(local_map, inst, name_index);
                let ptr = inst.get_ptr();
                let ptr_name = if let Some(name) = local_map.get(&ptr) {
                    name
                } else {
                    global_map.get(&ptr).unwrap()
                };
                text += format!(
                    "  {} = load float, float* {}, align 4\n",
                    local_map.get(&inst).unwrap(),
                    ptr_name
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
                    get_inst_value(inst.get_dest(), local_map, global_map)
                )
                .as_str();
            }
            IrType::Float => {
                text += format!(
                    "  store float {}, float* {}, align 4\n",
                    get_inst_value(inst.get_value(), local_map, global_map),
                    get_inst_value(inst.get_dest(), local_map, global_map)
                )
                .as_str();
            }
            _ => unreachable!("No other type in store"),
        },
        InstKind::Binary(op) => match op {
            BinOp::Add => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = add i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fadd float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                }
            }
            BinOp::Sub => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = sub i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fsub float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                }
            }
            BinOp::Mul => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = mul i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fmul float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                }
            }
            BinOp::Div => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = sdiv i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fdiv float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                }
            }
            BinOp::Rem => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = srem i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    unreachable!("No float rem in ir");
                }
            }
            BinOp::Gt => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = icmp sgt i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fcmp ogt float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                }
            }
            BinOp::Lt => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = icmp slt i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fcmp olt float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                }
            }
            BinOp::Ge => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = icmp sge i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map)
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fcmp oge float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map),
                    )
                    .as_str();
                }
            }
            BinOp::Le => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = icmp sle i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map),
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fcmp ole float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map),
                    )
                    .as_str();
                }
            }
            BinOp::Eq => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = icmp eq i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map),
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fcmp oeq float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map),
                    )
                    .as_str();
                }
            }
            BinOp::Ne => {
                if let IrType::Int = inst.get_ir_type() {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = icmp ne i32 {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map),
                    )
                    .as_str();
                } else {
                    name_index = put_name(local_map, inst, name_index);
                    text += format!(
                        "  {} = fcmp one float {}, {}\n",
                        local_map.get(&inst).unwrap().clone(),
                        get_inst_value(inst.get_lhs(), local_map, global_map),
                        get_inst_value(inst.get_rhs(), local_map, global_map),
                    )
                    .as_str();
                }
            }
        },
        InstKind::Unary(op) => {
            // 指令替换
            match op {
                UnOp::Pos => {
                    name_index = put_name(local_map, inst, name_index);
                    if let IrType::Int = inst.get_ir_type() {
                        text += format!(
                            "  {} = add i32 0, {}\n",
                            local_map.get(&inst).unwrap().clone(),
                            get_inst_value(inst.get_unary_operand(), local_map, global_map),
                        )
                        .as_str();
                    } else {
                        text += format!(
                            "  {} = fadd float 0.0, {}\n",
                            local_map.get(&inst).unwrap().clone(),
                            get_inst_value(inst.get_unary_operand(), local_map, global_map),
                        )
                        .as_str();
                    }
                }
                UnOp::Neg => {
                    name_index = put_name(local_map, inst, name_index);
                    if let IrType::Int = inst.get_ir_type() {
                        text += format!(
                            "  {} = sub i32 0, {}\n",
                            local_map.get(&inst).unwrap().clone(),
                            get_inst_value(inst.get_unary_operand(), local_map, global_map),
                        )
                        .as_str();
                    } else {
                        text += format!(
                            "  {} = fsub float 0.0, {}\n",
                            local_map.get(&inst).unwrap().clone(),
                            get_inst_value(inst.get_unary_operand(), local_map, global_map),
                        )
                        .as_str();
                    }
                }
                UnOp::Not => {
                    name_index = put_name(local_map, inst, name_index);
                    if let IrType::Int = inst.get_ir_type() {
                        text += format!(
                            "  {} = icmp eq i32 {}, 0 ;not\n",
                            local_map.get(&inst).unwrap().clone(),
                            get_inst_value(inst.get_unary_operand(), local_map, global_map),
                        )
                        .as_str();
                    } else {
                        text += format!(
                            "  {} = fcmp one float {}, 0.0 ;not\n",
                            local_map.get(&inst).unwrap().clone(),
                            get_inst_value(inst.get_unary_operand(), local_map, global_map),
                        )
                        .as_str();
                    }
                }
            }
        }
        InstKind::Branch => {
            if inst.is_br_jmp() {
                text += format!(
                    "  br label %{}\n",
                    format!("bb_{}", inst.get_parent_bb().get_next_bb()[0].get_name()).as_str()
                )
                .as_str();
            } else {
                match inst.get_br_cond().get_kind() {
                    InstKind::Binary(BinOp::Ne)
                    | InstKind::Binary(BinOp::Eq)
                    | InstKind::Binary(BinOp::Le)
                    | InstKind::Binary(BinOp::Lt)
                    | InstKind::Binary(BinOp::Gt)
                    | InstKind::Binary(BinOp::Ge)
                    | InstKind::Unary(UnOp::Not) => {
                        text += format!(
                            "  br i1 {}, label %{}, label %{}\n",
                            get_inst_value(inst.get_br_cond(), local_map, global_map),
                            format!("bb_{}", inst.get_true_bb().get_name()).as_str(),
                            format!("bb_{}", inst.get_false_bb().get_name()).as_str()
                        )
                        .as_str();
                    }
                    _ => {
                        if let IrType::Int = inst.get_br_cond().get_ir_type() {
                            text += format!(
                                "  %val_{}_add = icmp ne i32 {}, 0\n",
                                name_index,
                                get_inst_value(inst.get_br_cond(), local_map, global_map)
                            )
                            .as_str();
                        } else {
                            text += format!(
                                "  %val_{}_add = fcmp one float {}, 0.0\n",
                                name_index,
                                get_inst_value(inst.get_br_cond(), local_map, global_map)
                            )
                            .as_str();
                        }
                        text += format!(
                            "  br i1 %val_{}_add, label %{}, label %{}\n",
                            name_index,
                            format!("bb_{}", inst.get_true_bb().get_name()).as_str(),
                            format!("bb_{}", inst.get_false_bb().get_name()).as_str()
                        )
                        .as_str();
                        name_index += 1;
                    }
                }
            }
        }
        InstKind::Call(callee) => {
            let func_type = match inst.get_ir_type() {
                IrType::Int => "i32",
                IrType::Float => "float",
                IrType::Void => "void",
                _ => unreachable!("No Call in dump_inst"),
            };
            let mut param = String::new();
            for arg in inst.get_operands().iter() {
                let arg_type = match arg.get_ir_type() {
                    IrType::Int => "i32",
                    IrType::Float => "float",
                    IrType::IntPtr | IrType::FloatPtr => "ptr",
                    _ => unreachable!("No Call in dump_inst"),
                };
                param += format!(
                    "{} noundef {}, ",
                    arg_type,
                    get_inst_value(arg.clone(), local_map, global_map)
                )
                .as_str();
            }
            if param.len() >= 2 {
                param.truncate(param.len() - 2);
            }
            if let IrType::Void = inst.get_ir_type() {
                text += format!("  call {} @{}({})\n", func_type, callee, param).as_str();
            } else {
                name_index = put_name(local_map, inst, name_index);
                text += format!(
                    "  {} = call {} @{}({})\n",
                    local_map.get(&inst).unwrap(),
                    func_type,
                    callee,
                    param
                )
                .as_str();
            }
        }
        InstKind::Parameter => unreachable!("No parameter in bb"),
        InstKind::Return => match inst.get_ir_type() {
            IrType::Void => {
                text += "  ret void\n";
            }
            IrType::Int => {
                text += format!(
                    "  ret i32 {}\n",
                    get_inst_value(inst.get_return_value(), local_map, global_map)
                )
                .as_str();
            }
            IrType::Float => {
                text += format!(
                    "  ret float {}\n",
                    get_inst_value(inst.get_return_value(), local_map, global_map)
                )
                .as_str();
            }
            _ => unreachable!("No Return in dump_inst"),
        },
        InstKind::FtoI => {
            name_index = put_name(local_map, inst, name_index);
            text += format!(
                "  {} = fptosi float {} to i32\n",
                local_map.get(&inst).unwrap().clone(),
                get_inst_value(inst.get_float_to_int_value(), local_map, global_map),
            )
            .as_str();
        }
        InstKind::ItoF => {
            name_index = put_name(local_map, inst, name_index);
            text += format!(
                "  {} = sitofp i32 {} to float\n",
                local_map.get(&inst).unwrap().clone(),
                get_inst_value(inst.get_int_to_float_value(), local_map, global_map),
            )
            .as_str();
        }
        InstKind::ConstInt(_) | InstKind::ConstFloat(_) => {
            // 常量不处理
        }
        InstKind::GlobalConstInt(_)
        | InstKind::GlobalConstFloat(_)
        | InstKind::GlobalInt(_)
        | InstKind::GlobalFloat(_) => {
            unreachable!("No Global in bb")
        }
        InstKind::Phi => {
            name_index = put_name(local_map, inst, name_index);

            let phi_type = if let IrType::Int = inst.get_ir_type() {
                "i32"
            } else {
                "float"
            };
            text += format!("  {} = phi {} ", local_map.get(&inst).unwrap(), phi_type).as_str();
            for (index, op) in inst.get_operands().iter().enumerate() {
                text += format!(
                    "[ {}, %{} ], ",
                    get_inst_value(op.clone(), local_map, global_map),
                    format!("bb_{}", inst.get_phi_predecessor(index).get_name()).as_str()
                )
                .as_str();
            }
            if inst.get_operands().len() > 0 {
                text.truncate(text.len() - 2);
            }
            text += "\n";
        }
        InstKind::Head => unreachable!("No Head in dump_inst"),
    }
    (name_index, text)
}

fn put_name(
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    inst: ObjPtr<Inst>,
    name_index: i32,
) -> i32 {
    if local_map.contains_key(&inst) {
        name_index
    } else {
        local_map.insert(inst, format!("%val_{}", name_index));
        name_index + 1
    }
}

fn get_inst_value(
    inst: ObjPtr<Inst>,
    local_map: &mut HashMap<ObjPtr<Inst>, String>,
    global_map: &HashMap<ObjPtr<Inst>, String>,
) -> String {
    static mut NOTFOUND: i32 = 0;
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
                local_map.insert(
                    inst,
                    format!("%notfound{}", unsafe {
                        NOTFOUND += 1;
                        NOTFOUND
                    }),
                );
                local_map.get(&inst).unwrap().clone()
            }
        }
    }
}
