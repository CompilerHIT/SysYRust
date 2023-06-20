use super::*;
pub fn copy_func(
    func_name: &str,
    func: ObjPtr<Function>,
    arg_list: Vec<ObjPtr<Inst>>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<BasicBlock> {
    let mut bb_map = HashMap::new();
    let mut inst_map = HashMap::new();

    // 先做形参与实参的映射
    for (i, arg) in arg_list.iter().enumerate() {
        inst_map.insert(func.get_parameter_list()[i], arg.clone());
    }

    // 广度优先遍历，拷贝bb
    let mut copy_bb_list = Vec::new();
    let mut queue = vec![func.get_head()];
    let mut visited = HashSet::new();
    while let Some(bb) = queue.pop() {
        if visited.contains(&bb) {
            continue;
        }
        visited.insert(bb);
        let copy_bb = copy_bb(func_name, bb, pools, &mut inst_map, &mut bb_map);
        copy_bb_list.push(copy_bb);

        for next_bb in bb.get_next_bb() {
            queue.insert(0, next_bb.clone());
        }
    }

    // 映射bb
    for bb in copy_bb_list.iter() {
        map_bb(bb.clone(), &mut bb_map, &mut inst_map);
    }

    copy_bb_list
        .iter()
        .find(|x| x.get_up_bb().len() == 0)
        .unwrap()
        .clone()
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
    let mut copy_bb = pools.0.put(bb.as_ref().clone());

    // 初始化bb
    copy_bb.init_head();
    let name = copy_bb.get_name().to_string();
    copy_bb.set_name(format!("func_{}_{}_inline", func_name, name));

    // 复制指令
    let mut inst = bb.get_head_inst();
    loop {
        let copy_inst = copy_inst(inst, inst_map, pools);
        copy_bb.push_back(copy_inst);
        if inst.is_tail() {
            break;
        }
        inst = inst.get_next();
    }

    bb_map.insert(bb, copy_bb);

    copy_bb
}

fn map_inst_in_bb(inst: ObjPtr<Inst>, inst_map: &mut HashMap<ObjPtr<Inst>, ObjPtr<Inst>>) {
    let mut inst = inst;
    loop {
        let operands = inst
            .get_operands()
            .iter()
            .map(|x| inst_map.get(x).unwrap().clone())
            .collect();

        inst.set_operands(operands);

        if inst.is_tail() {
            break;
        }
        inst = inst.get_next();
    }
}

fn copy_inst(
    inst: ObjPtr<Inst>,
    inst_map: &mut HashMap<ObjPtr<Inst>, ObjPtr<Inst>>,
    pools: &mut (&mut ObjPool<BasicBlock>, &mut ObjPool<Inst>),
) -> ObjPtr<Inst> {
    let copy_inst = pools.1.put(inst.as_ref().clone());
    inst_map.insert(inst, copy_inst);
    copy_inst
}
