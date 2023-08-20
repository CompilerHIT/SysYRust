///调度相关模块
use super::*;

impl Func {
    /// 识别根据def use识别局部变量，窗口设为3，若存活区间少于3则认为是局部变量
    /// 局部变量一定在块内，对于born为-1的一定是非局部变量
    pub fn cal_tmp_var(&mut self) {
        self.calc_live_for_handle_call();
        self.build_reg_intervals();
        for block in self.blocks.iter() {
            for (st, ed) in block.reg_intervals.iter() {
                if st.1 != -1 && ed.1 - st.1 < 3 {
                    self.tmp_vars.insert(st.0);
                }
            }
        }
    }

    /// 块内代码调度
    pub fn list_scheduling_tech(&mut self) {
        // 建立数据依赖图
        for b in self.blocks.iter() {
            let mut graph: Graph<ObjPtr<LIRInst>, (i32, ObjPtr<LIRInst>)> = Graph::new();
            let mut control_insts: Vec<ObjPtr<LIRInst>> = Vec::new();
            // 对于涉及控制流的语句，不能进行调度
            let basicblock: Vec<ObjPtr<LIRInst>> = b
                .insts
                .iter()
                .filter(|inst| match inst.get_type() {
                    InstrsType::Ret(..) | InstrsType::Branch(..) | InstrsType::Jump => {
                        // 保存，以便后续恢复
                        control_insts.push(**inst);
                        false
                    }
                    _ => true,
                })
                .map(|x| *x)
                .collect();

            // 对于清除掉控制流语句的块，建立数据依赖图
            for (i, inst) in basicblock.iter().rev().enumerate() {
                let pos = basicblock.len() - i - 1;
                graph.add_node(*inst);

                // call支配后续所有指令
                for index in 1..=pos {
                    let i = basicblock[pos - index];
                    if i.get_type() == InstrsType::Call {
                        graph.add_edge(*inst, (1, i));
                    } else {
                        continue;
                    }
                }

                // call依赖于之前的所有指令
                // if inst.get_type() == InstrsType::Call {
                //     special_inst_pos.insert(*inst, i);
                // }
                if inst.get_type() == InstrsType::Call {
                    for index in 1..=pos {
                        let i = basicblock[pos - index];
                        graph.add_edge(*inst, (1, i));
                    }
                }

                // 认为load/store依赖之前的load/store
                if inst.get_type() == InstrsType::Load || inst.get_type() == InstrsType::Store {
                    // special_inst_pos.insert(*inst, i);
                    for index in 1..=pos {
                        let i = basicblock[pos - index];
                        if sl_conflict(*inst, i) {
                            graph.add_edge(*inst, (1, i));
                        } else {
                            continue;
                        }
                    }
                }

                let use_vec = inst.get_reg_use();
                let def_vec = inst.get_reg_def();

                for reg in use_vec.iter() {
                    // 向上找一个use的最近def,将指令加入图中
                    for index in 1..=pos {
                        let i = basicblock[pos - index];
                        if i.get_reg_def().contains(reg) {
                            graph.add_edge(*inst, (1, i));
                        }
                    }
                }

                for reg in def_vec.iter() {
                    // 向上找一个def的最近use,将指令加入图中
                    for index in 1..=pos {
                        let i = basicblock[pos - index];
                        if i.get_reg_use().contains(reg) {
                            graph.add_edge(*inst, (1, i));
                        }
                    }
                }
            }
            let mut queue: VecDeque<ObjPtr<LIRInst>> = VecDeque::new();
            let mut visited = HashSet::new();

            let mut g = graph
                .get_nodes()
                .into_iter()
                .map(|(n, e)| {
                    (
                        *n,
                        e.iter()
                            .map(|(_, inst)| *inst)
                            .collect::<Vec<ObjPtr<LIRInst>>>(),
                    )
                })
                .collect::<HashMap<ObjPtr<LIRInst>, Vec<ObjPtr<LIRInst>>>>();

            let mut reverse_graph: HashMap<ObjPtr<LIRInst>, Vec<ObjPtr<LIRInst>>> = HashMap::new();
            for (from, to_nodes) in g.iter() {
                for to_node in to_nodes {
                    reverse_graph
                        .entry(*to_node)
                        .or_insert(Vec::new())
                        .push(*from);
                }
            }

            // 拓扑排序
            loop {
                if queue.len() == g.len() {
                    break;
                }
                // 0出度算法，出度度为0代表不依赖于其他指令
                let mut zero_nodes: Vec<ObjPtr<LIRInst>> = Vec::new();
                for (node, _) in graph.get_nodes().iter() {
                    let out_nodes = g.get(node).unwrap();
                    if out_nodes.len() == 0 && !visited.contains(node) {
                        visited.insert(*node);
                        queue.push_back(*node);
                        zero_nodes.push(*node);
                    }
                }
                // 在反向图中查找该节点支配的节点，从而删除两点之间的边
                for node in zero_nodes.iter() {
                    if let Some(in_nodes) = reverse_graph.get(node) {
                        for in_node in in_nodes {
                            if let Some(out_nodes) = g.get_mut(&in_node) {
                                out_nodes.retain(|&n| n != *node);
                            }
                        }
                    }
                }
            }

            // 调度方案，在不考虑资源的情况下i有可能相同
            let mut schedule_map: HashMap<ObjPtr<LIRInst>, i32> = HashMap::new();

            let mut s;
            for inst in queue.iter() {
                if let Some(edges) = graph.get_edges(*inst) {
                    s = edges
                        .iter()
                        .map(|(w, inst)| w + *schedule_map.get(inst).unwrap_or(&0))
                        .max()
                        .unwrap_or(0);
                } else {
                    s = 0;
                }

                // 指令位置相同，若两个是特殊指令则距离增加2，否则增加1
                while let Some((l, _)) = schedule_map.iter().find(|(_, v)| **v == s) {
                    if dep_inst_special(inst.clone(), l.clone()) {
                        s += 2;
                    } else {
                        s += 1;
                    }
                }

                let mut visited: HashSet<ObjPtr<LIRInst>> = HashSet::new();
                while let Some((l, _)) = schedule_map
                    .iter()
                    .find(|(inst, v)| **v == s - 1 && !visited.contains(inst))
                {
                    if def_use_near(inst.clone(), l.clone()) {
                        s += 1;
                    } else {
                        visited.insert(l.clone());
                    }
                }

                // // 对于相邻指令，若是特殊指令则距离增加为2
                // let mut visited2 = HashSet::new();
                // while let Some((l, _)) = schedule_map
                //     .iter()
                //     .find(|(_, v)| **v == s - 1 && !visited2.contains(inst))
                // {
                //     if dep_inst_special(inst.clone(), l.clone()) {
                //         s += 1;
                //     } else {
                //         visited2.insert(l.clone());
                //     }
                // }
                schedule_map.insert(*inst, s);
            }

            let mut schedule_res: Vec<ObjPtr<LIRInst>> =
                schedule_map.iter().map(|(&inst, _)| inst).collect();
            schedule_res.sort_by(|a, b| {
                schedule_map
                    .get(a)
                    .unwrap()
                    .cmp(schedule_map.get(b).unwrap())
            });

            // 打印调度方案
            // 调度前
            log_file!("before_schedule.log", "{}", b.label);
            for inst in b.insts.iter() {
                log_file!("before_schedule.log", "{}", inst.as_ref());
            }

            // 移动代码
            b.as_mut().insts = schedule_res;
            b.as_mut().push_back_list(&mut control_insts);

            // 调度后
            log_file!("after_schedule.log", "{}", b.label);
            for inst in b.insts.iter() {
                log_file!("after_schedule.log", "{}", inst.as_ref());
            }
        }
    }
}

fn dep_inst_special(inst: ObjPtr<LIRInst>, last: ObjPtr<LIRInst>) -> bool {
    // 若相邻的指令是内存访问
    match inst.get_type() {
        InstrsType::LoadFromStack
        | InstrsType::StoreToStack
        | InstrsType::LoadParamFromStack
        | InstrsType::StoreParamToStack
        | InstrsType::Load
        | InstrsType::Store => match last.get_type() {
            InstrsType::LoadFromStack
            | InstrsType::StoreToStack
            | InstrsType::LoadParamFromStack
            | InstrsType::StoreParamToStack
            | InstrsType::Load
            | InstrsType::Store
            | InstrsType::OpReg(SingleOp::LoadAddr) => true,
            _ => false,
        },

        // 若相邻的指令是乘法运算
        InstrsType::Binary(BinaryOp::Mul) => match last.get_type() {
            InstrsType::Binary(BinaryOp::Mul) => true,
            _ => false,
        },

        // 若相邻的指令是浮点运算
        InstrsType::Binary(..) => match last.get_type() {
            InstrsType::Binary(..) => {
                let inst_float = inst.operands.iter().any(|op| match op {
                    Operand::Reg(reg) => reg.get_type() == ScalarType::Float,
                    _ => false,
                });
                let last_float = last.operands.iter().any(|op| match op {
                    Operand::Reg(reg) => reg.get_type() == ScalarType::Float,
                    _ => false,
                });
                if last_float && inst_float {
                    true
                } else {
                    false
                }
            }
            _ => false,
        },
        _ => false,
    }
}

fn def_use_near(inst: ObjPtr<LIRInst>, last: ObjPtr<LIRInst>) -> bool {
    // 若def use相邻
    if let Some(inst_def) = last.get_reg_def().last() {
        inst.get_reg_use().iter().any(|reg_use| {
            if reg_use == inst_def {
                return true;
            }
            false
        });
    };
    false
}

fn sl_conflict(inst1: ObjPtr<LIRInst>, inst2: ObjPtr<LIRInst>) -> bool {
    // 写后写
    if inst1.get_type() == inst2.get_type() && inst1.get_type() == InstrsType::Store {
        if inst1.operands[1] == inst2.operands[1] {
            return true;
        }
    }
    // 写后读/读后写
    if (inst1.get_type() == InstrsType::Load && inst2.get_type() == InstrsType::Store)
        || (inst1.get_type() == InstrsType::Store && inst2.get_type() == InstrsType::Load)
    {
        if inst1.operands[1] == inst2.operands[1] {
            return true;
        }
    }
    false
}
