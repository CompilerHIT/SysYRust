use crate::ir::tools::bfs_bb_proceess;

use super::*;
pub fn copy_func(
    func_name: &str,
    func: ObjPtr<Function>,
    global_var: Vec<(&String, ObjPtr<Inst>)>,
    arg_list: Vec<ObjPtr<Inst>>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> (ObjPtr<BasicBlock>, ObjPtr<BasicBlock>) {
    let mut bb_map = HashMap::new();
    let mut inst_map = HashMap::new();

    // 先做形参与实参的映射
    for (i, arg) in arg_list.iter().enumerate() {
        inst_map.insert(func.get_parameter_list()[i], arg.clone());
    }

    // 再做全局变量的映射
    for (_, inst) in global_var.iter() {
        inst_map.insert(inst.clone(), inst.clone());
    }

    // 广度优先遍历，拷贝bb
    let mut copy_bb_list = Vec::new();
    bfs_bb_proceess(func.get_head(), |bb| {
        copy_bb_list.push(copy_bb(func_name, bb, pools, &mut inst_map, &mut bb_map));
    });

    // 映射bb
    for bb in copy_bb_list.iter() {
        map_bb(bb.clone(), &mut bb_map, &mut inst_map);
    }

    (
        copy_bb_list.iter().find(|x| x.is_entry()).unwrap().clone(),
        copy_bb_list.iter().find(|x| x.is_exit()).unwrap().clone(),
    )
}

fn map_bb(
    bb: ObjPtr<BasicBlock>,
    bb_map: &mut HashMap<ObjPtr<BasicBlock>, ObjPtr<BasicBlock>>,
    inst_map: &mut HashMap<ObjPtr<Inst>, ObjPtr<Inst>>,
) {
    bb.as_mut().set_up_bb(
        bb.get_up_bb()
            .iter()
            .map(|x| bb_map.get(x).unwrap().clone())
            .collect(),
    );

    bb.as_mut().set_next_bb(
        bb.get_next_bb()
            .iter()
            .map(|x| bb_map.get(x).unwrap().clone())
            .collect(),
    );

    map_inst_in_bb(bb.get_head_inst(), inst_map);
}

fn copy_bb(
    func_name: &str,
    bb: ObjPtr<BasicBlock>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
    inst_map: &mut HashMap<ObjPtr<Inst>, ObjPtr<Inst>>,
    bb_map: &mut HashMap<ObjPtr<BasicBlock>, ObjPtr<BasicBlock>>,
) -> ObjPtr<BasicBlock> {
    let mut bb_copy = pools.0.put(bb.as_ref().clone());

    // 初始化bb
    bb_copy.init_head();
    let name = bb_copy.get_name().to_string();
    bb_copy.set_name(format!("{}_{}_inline", func_name, name));

    // 复制指令
    inst_process_in_bb(bb.get_head_inst(), |inst| {
        let inst_copy = copy_inst(inst, inst_map, pools);
        bb_copy.push_back(inst_copy);
    });

    bb_map.insert(bb, bb_copy);

    bb_copy
}

fn map_inst_in_bb(inst: ObjPtr<Inst>, inst_map: &mut HashMap<ObjPtr<Inst>, ObjPtr<Inst>>) {
    let inst_map = |inst_list: &Vec<ObjPtr<Inst>>| {
        inst_list
            .iter()
            .map(|op| inst_map.get(op).unwrap().clone())
            .collect()
    };
    inst_process_in_bb(inst, |x| {
        x.as_mut().set_operands(inst_map(x.get_operands()));
        x.as_mut().set_users(inst_map(x.get_use_list()));
    });
}

fn copy_inst(
    inst: ObjPtr<Inst>,
    inst_map: &mut HashMap<ObjPtr<Inst>, ObjPtr<Inst>>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<Inst> {
    let copyed_inst = pools.1.put(inst.as_ref().clone());
    debug_assert_eq!(inst_map.contains_key(&inst), false);
    inst_map.insert(inst, copyed_inst);
    copyed_inst
}
