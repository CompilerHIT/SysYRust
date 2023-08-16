use super::*;
use crate::log;
///一些进行分析需要用到的工具
impl AsmModule {
    pub fn analyse_inst_with_live_now(
        func: &Func,
        inst_analyser: &mut dyn FnMut(ObjPtr<LIRInst>, &HashSet<Reg>),
    ) {
        for bb in func.blocks.iter() {
            let mut livenow: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                livenow.insert(*reg);
            });
            for inst in bb.insts.iter().rev() {
                inst_analyser(*inst, &livenow);
                for reg in inst.get_reg_def() {
                    livenow.remove(&reg);
                }
                for reg in inst.get_reg_use() {
                    livenow.insert(reg);
                }
            }
        }
    }
    pub fn analyse_inst_with_index_and_live_now(
        func: &Func,
        inst_analyser: &mut dyn FnMut(ObjPtr<LIRInst>, usize, &HashSet<Reg>, ObjPtr<BB>),
    ) {
        for bb in func.blocks.iter() {
            let mut livenow: HashSet<Reg> = HashSet::new();
            bb.live_out.iter().for_each(|reg| {
                livenow.insert(*reg);
            });
            for (index, inst) in bb.insts.iter().enumerate().rev() {
                for reg in inst.get_reg_def() {
                    livenow.remove(&reg);
                }
                //
                inst_analyser(*inst, index, &livenow, *bb);
                for reg in inst.get_reg_use() {
                    livenow.insert(reg);
                }
            }
        }
    }

    pub fn iter_insts(func: ObjPtr<Func>, processor: &mut dyn FnMut(&ObjPtr<LIRInst>)) {
        for bb in func.blocks.iter() {
            for inst in bb.insts.iter() {
                processor(inst);
            }
        }
    }
}

impl AsmModule {
    ///创建一个函数调用族群(包括族长以及族长调用和简接调用的所有函数)
    ///调用该函数前应该先调用call map建立直接调用关系表
    pub fn build_func_groups(
        call_map: &HashMap<String, HashSet<String>>,
    ) -> HashMap<String, HashSet<String>> {
        let mut func_groups = HashMap::new();
        //建族,加入自身
        for (name, _) in call_map.iter() {
            func_groups.insert(name.clone(), HashSet::from_iter(vec![name.clone()]));
        }
        //通过call map初步发展成员
        for (caller, callees) in call_map.iter() {
            for callee in callees.iter() {
                func_groups.get_mut(caller).unwrap().insert(callee.clone());
            }
        }
        //递归发展成员
        loop {
            let mut finish_flag = true;
            let mut to_add = Vec::new();
            for (master, members) in func_groups.iter() {
                let mut new_members = members.clone();
                for member in members.iter() {
                    assert!(call_map.contains_key(member), "{}", member);
                    for callee in call_map.get(member).unwrap() {
                        new_members.insert(callee.clone());
                    }
                }
                to_add.push((master.clone(), new_members));
            }
            for (master, new_members) in to_add {
                if func_groups.get(&master).unwrap().len() < new_members.len() {
                    finish_flag = false;
                    func_groups.get_mut(&master).unwrap().extend(new_members);
                }
            }
            if finish_flag {
                break;
            }
        }
        func_groups
    }
}

impl AsmModule {
    pub fn print_asm(&mut self, path: &str) {
        // let mut file = File::create(path).unwrap();
        // self.generate_row_asm(&mut file);
    }
}

impl AsmModule {
    pub fn log_insts(&self) {
        self.name_func.iter().for_each(|(_, func)| {
            func.blocks.iter().for_each(|b| {
                log!("block: {}", b.label);
                b.insts.iter().for_each(|inst| {
                    log!("inst: {:?}", inst.as_ref());
                })
            })
        });
    }
}
