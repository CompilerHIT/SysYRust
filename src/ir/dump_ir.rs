use std::collections::{HashMap, HashSet};

///! 此文件用于debug时打印IR，方便调试
use crate::ir::basicblock::BasicBlock;
use crate::ir::instruction::Inst;
use crate::utility::ObjPtr;

use super::{
    function::Function,
    instruction::{BinOp, InstKind, UnOp},
    module::Module,
};

static mut I_NAME: i32 = 0;

pub fn dump_now(module: &Module) {
    let mut map = HashMap::new();
    // 遍历module中的全局变量
    let mut text = format!("global variable:\n");
    for (_, inst) in module.get_all_var() {
        text = dump_inst(inst, text, &mut map);
    }

    text += "\n";

    // 遍历module中的函数
    for (name, func) in module.get_all_func() {
        if func.is_empty_bb() {
            continue;
        }
        text += "---------- func begin ----------\n";
        text = dump_func(func, &mut map, text, name);
        text += "---------- func end ----------\n\n";
    }

    // 将text写入文件
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create("ir.txt").unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

pub fn dump_func(
    func: ObjPtr<Function>,
    map: &mut HashMap<ObjPtr<Inst>, i32>,
    mut text: String,
    func_name: &str,
) -> String {
    text = format!("{}function: {}\n", text, func_name);
    text = format!("{}  return type: {:?}\n", text, func.get_return_type());
    text = format!("{}  parameters:\n", text);
    for (name, inst) in func.get_params() {
        text = format!(
            "{}    %{} = {}\n",
            text,
            map_get_name(map, inst.clone()),
            name
        );
    }
    text = format!("{}\nBasicBlock:\n", text);

    // 广度优先遍历bb
    let mut queue = Vec::new();
    let mut visited = HashSet::new();
    if !func.is_empty_bb() {
        queue.insert(0, func.get_head());
        while let Some(bb) = queue.pop() {
            if visited.contains(&bb) {
                continue;
            }
            visited.insert(bb);
            text = dump_block(bb, text, map);
            for succ in bb.get_next_bb() {
                queue.insert(0, succ.clone());
            }
        }
    }

    text
}

fn dump_block(
    block: ObjPtr<BasicBlock>,
    mut text: String,
    map: &mut HashMap<ObjPtr<Inst>, i32>,
) -> String {
    text = format!("{}block: {}\n  pred: ", text, block.get_name());
    for pred in block.get_up_bb() {
        text = format!("{} {}", text, pred.get_name());
    }
    text += "\n";
    text = format!("{}  succ: ", text);
    for succ in block.get_next_bb() {
        text = format!("{} {}", text, succ.get_name());
    }
    text += "\n  inst:\n";
    if !block.is_empty() {
        let mut inst = block.get_head_inst();
        loop {
            text = dump_inst(inst, text, map);
            inst = inst.get_next();
            if inst.is_tail() {
                text = dump_inst(inst, text, map);
                break;
            }
        }
    }

    text += "\n";
    text
}

pub fn dump_inst(
    inst: ObjPtr<Inst>,
    mut text: String,
    map: &mut HashMap<ObjPtr<Inst>, i32>,
) -> String {
    text += "   ";
    let name = map_get_name(map, inst);
    match inst.get_kind() {
        InstKind::Alloca(init) => {
            text = format!("{}%{} = alloca {:?}\n", text, name, init);
        }
        InstKind::Binary(op) => {
            text = format!(
                "{}%{} = %{} {} %{}\n",
                text,
                name,
                map_get_name(map, inst.get_lhs()),
                dump_bin_op(op),
                map_get_name(map, inst.get_rhs())
            );
        }
        InstKind::Branch => {
            if inst.is_jmp() {
                text = format!(
                    "{}jum {}\n",
                    text,
                    inst.get_parent_bb().get_next_bb()[0].get_name()
                );
            } else {
                text = format!(
                    "{}br %{}, {} {}\n",
                    text,
                    map.get(&inst.get_br_cond()).unwrap(),
                    inst.get_parent_bb().get_next_bb()[0].get_name(),
                    inst.get_parent_bb().get_next_bb()[1].get_name()
                );
            }
        }
        InstKind::Call(callee) => {
            text = format!("{}%{} = call {} Parameter: ", text, name, callee);
            for arg in inst.get_operands() {
                text = format!("{}%{} ", text, map_get_name(map, arg.clone()));
            }
            text += "\n";
        }
        InstKind::Gep => {
            text = format!(
                "{}%{} = gep ptr:{} offset:{}\n",
                text,
                name,
                map_get_name(map, inst.get_gep_ptr()),
                map_get_name(map, inst.get_gep_offset())
            );
        }
        InstKind::FtoI => {
            text = format!(
                "{}%{} = ftoi %{}\n",
                text,
                name,
                map_get_name(map, inst.get_float_to_int_value())
            );
        }
        InstKind::ItoF => {
            text = format!(
                "{}%{} = itof %{}\n",
                text,
                name,
                map_get_name(map, inst.get_int_to_float_value())
            );
        }
        InstKind::Return => {
            if let super::ir_type::IrType::Void = inst.get_ir_type() {
                text = format!("{}return\n", text);
            } else {
                text = format!(
                    "{}return %{}\n",
                    text,
                    map_get_name(map, inst.get_return_value())
                );
            }
        }
        InstKind::Load => {
            text = format!(
                "{}%{} = load %{}\n",
                text,
                name,
                map_get_name(map, inst.get_ptr())
            );
        }
        InstKind::Phi => {
            text = format!("{}%{} = phi ", text, name);
            for &value in inst.get_operands() {
                text = format!("{}%{} ", text, map_get_name(map, value));
            }
            text += "\n";
        }
        InstKind::Store => {
            text = format!(
                "{}store ptr:%{} value:%{}\n",
                text,
                map_get_name(map, inst.get_ptr()),
                map_get_name(map, inst.get_value())
            );
        }
        InstKind::Unary(op) => {
            text = format!(
                "{}%{} = {} %{}\n",
                text,
                name,
                dump_un_op(op),
                map_get_name(map, inst.get_unary_operand())
            );
        }
        InstKind::ConstInt(value) => {
            text = format!("{}%{} = const int {}\n", text, name, value);
        }
        InstKind::ConstFloat(value) => {
            text = format!("{}%{} = const float {}\n", text, name, value);
        }
        InstKind::Parameter | InstKind::Head(_) => {}
        InstKind::GlobalConstInt(value) => {
            text = format!("{}%{} = global const int {}\n", text, name, value);
        }
        InstKind::GlobalFloat(value) => {
            text = format!("{}%{} = global const float {}\n", text, name, value);
        }
        InstKind::GlobalInt(value) => {
            text = format!("{}%{} = global const int {}\n", text, name, value);
        }
        InstKind::GlobalConstFloat(value) => {
            text = format!("{}%{} = global const float {}\n", text, name, value);
        }
    }
    text
}

fn map_get_name(map: &mut HashMap<ObjPtr<Inst>, i32>, inst: ObjPtr<Inst>) -> String {
    if let Some(name) = map.get(&inst) {
        format!("{}", name)
    } else {
        let name = get_name();
        map.insert(inst, name);
        format!("{}", name)
    }
}

fn dump_bin_op(op: BinOp) -> String {
    match op {
        BinOp::Add => "+".to_string(),
        BinOp::Sub => "-".to_string(),
        BinOp::Mul => "*".to_string(),
        BinOp::Div => "/".to_string(),
        BinOp::Rem => "%".to_string(),
        BinOp::And => "&".to_string(),
        BinOp::Or => "|".to_string(),
        BinOp::Eq => "==".to_string(),
        BinOp::Ne => "!=".to_string(),
        BinOp::Le => "<=".to_string(),
        BinOp::Lt => "<".to_string(),
        BinOp::Ge => ">=".to_string(),
        BinOp::Gt => ">".to_string(),
    }
}

fn dump_un_op(op: UnOp) -> String {
    match op {
        UnOp::Pos => "+".to_string(),
        UnOp::Neg => "-".to_string(),
        UnOp::Not => "!".to_string(),
    }
}

fn get_name() -> i32 {
    unsafe {
        let name = I_NAME;
        I_NAME += 1;
        name
    }
}
