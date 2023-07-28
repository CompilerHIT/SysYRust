use super::*;

impl Func {
    // 计算活跃区间的时候, 主动把6个临时寄存器的生存周期设置为无限
    pub fn calc_live_for_alloc_reg(&self) {
        //TODO, 去除allocable限制!
        let calc_live_file = "callive.txt";
        // fs::remove_file(calc_live_file);
        log_file!(
            calc_live_file,
            "-----------------------------------cal live func:{}---------------------------",
            self.label
        );
        // 打印函数里面的寄存器活跃情况
        let printinterval = || {
            let mut que: VecDeque<ObjPtr<BB>> = VecDeque::new();
            let mut passed_bb = HashSet::new();
            que.push_front(self.entry.unwrap());
            passed_bb.insert(self.entry.unwrap());
            log_file!(calc_live_file, "func:{}", self.label);
            while !que.is_empty() {
                let cur_bb = que.pop_front().unwrap();
                log_file!(calc_live_file, "block {}:", cur_bb.label);
                log_file!(calc_live_file, "live in:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_in);
                log_file!(calc_live_file, "live out:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_out);
                log_file!(calc_live_file, "live use:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_use);
                log_file!(calc_live_file, "live def:");
                log_file!(calc_live_file, "{:?}", cur_bb.live_def);
                for next in cur_bb.out_edge.iter() {
                    if passed_bb.contains(next) {
                        continue;
                    }
                    passed_bb.insert(*next);
                    que.push_back(*next);
                }
            }
        };

        // 计算公式，live in 来自于所有前继的live out的集合 + 自身的live use
        // live out等于所有后继块的live in的集合与 (自身的livein 和live def的并集) 的交集
        // 以块为遍历单位进行更新
        // TODO 重写
        // 首先计算出live def和live use
        // if self.label == "main" {
        //     log!("to");
        // }

        let mut queue: VecDeque<(ObjPtr<BB>, Reg)> = VecDeque::new();
        for block in self.blocks.iter() {
            log_file!(calc_live_file, "block:{}", block.label);
            block.as_mut().live_use.clear();
            block.as_mut().live_def.clear();
            for it in block.as_ref().insts.iter().rev() {
                log_file!(calc_live_file, "{}", it.as_ref());
                for reg in it.as_ref().get_reg_def().into_iter() {
                    block.as_mut().live_use.remove(&reg);
                    block.as_mut().live_def.insert(reg);
                }
                for reg in it.as_ref().get_reg_use().into_iter() {
                    block.as_mut().live_def.remove(&reg);
                    block.as_mut().live_use.insert(reg);
                }
            }
            log_file!(
                calc_live_file,
                "live def:{:?},live use:{:?}",
                block
                    .live_def
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<Vec<i32>>(),
                block
                    .live_use
                    .iter()
                    .map(|e| e.get_id())
                    .collect::<Vec<i32>>()
            );
            //
            for reg in block.as_ref().live_use.iter() {
                queue.push_back((block.clone(), reg.clone()));
            }

            block.as_mut().live_in = block.as_ref().live_use.clone();
            block.as_mut().live_out.clear();
            // if let Some(last_isnt) = block.insts.last() {
            //     match last_isnt.get_type() {
            //         InstrsType::Ret(r_type) => {
            //             match r_type {
            //                 ScalarType::Int => {
            //                     let ret_reg = Reg::new(10, r_type);
            //                     block.as_mut().live_out.insert(ret_reg);
            //                     if !block.live_def.contains(&ret_reg) {
            //                         queue.push_front((*block, ret_reg));
            //                     }
            //                 }
            //                 ScalarType::Float => {
            //                     let ret_reg = Reg::new(10 + FLOAT_BASE, r_type);
            //                     block.as_mut().live_out.insert(ret_reg);
            //                     if !block.live_def.contains(&ret_reg) {
            //                         queue.push_front((*block, ret_reg));
            //                     }
            //                 }
            //                 _ => (),
            //             };
            //         }
            //         _ => (),
            //     }
            // }
        }

        //然后计算live in 和live out
        while let Some(value) = queue.pop_front() {
            let (block, reg) = value;
            log_file!(
                calc_live_file,
                "block {} 's ins:{:?}, transport live out:{}",
                block.label,
                block
                    .in_edge
                    .iter()
                    .map(|b| &b.label)
                    .collect::<HashSet<&String>>(),
                reg
            );
            for pred in block.as_ref().in_edge.iter() {
                if pred.as_mut().live_out.insert(reg) {
                    if pred.as_mut().live_def.contains(&reg) {
                        continue;
                    }
                    if pred.as_mut().live_in.insert(reg) {
                        queue.push_back((pred.clone(), reg));
                    }
                }
            }
        }

        //把sp和ra寄存器加入到所有的块的live out,live in中，表示这些寄存器永远不能在函数中自由分配使用
        for bb in self.blocks.iter() {
            //0:zero, 1:ra, 2:sp,3:gp,4:tp 是必须保存的,5-7做临时寄存器
            //8:s0用于处理overflow
            for id in 0..=8 {
                bb.as_mut().live_in.insert(Reg::new(id, ScalarType::Int));
                bb.as_mut().live_out.insert(Reg::new(id, ScalarType::Int));
            }
            for id in 18..=20 {
                bb.as_mut()
                    .live_in
                    .insert(Reg::new(id + FLOAT_BASE, ScalarType::Float));
                bb.as_mut()
                    .live_out
                    .insert(Reg::new(id + FLOAT_BASE, ScalarType::Float));
            }
        }

        log_file!(calc_live_file,"-----------------------------------after count live in,live out----------------------------");
        printinterval();
    }

    pub fn allocate_reg(&mut self) {
        // 函数返回地址保存在ra中
        self.calc_live_for_alloc_reg();
        // for bb in self.blocks.iter() {
        //     if self.label != "float_eq" {
        //         continue;
        //     }
        //     for inst in bb.insts.iter() {
        //         inst.get_regs().iter().for_each(|r| {
        //             log!("{:?}", inst);
        //             log!("{}", inst.as_ref());
        //             log!("{}", r);
        //         })
        //     }
        // }
        // let mut allocator = crate::backend::regalloc::easy_ls_alloc::Allocator::new();
        let mut allocator = crate::backend::regalloc::easy_gc_alloc::Allocator::new();
        // let mut allocator = crate::backend::regalloc::opt_ls_alloc::Allocator::new();
        // let mut allocator = crate::backend::regalloc::opt_gc_alloc2::Allocator::new();
        // let mut allocator = crate::backend::regalloc::opt_gc_alloc::Allocator::new();
        // let mut allocator = crate::backend::regalloc::base_alloc::Allocator::new();
        let alloc_stat = allocator.alloc(self);

        // 评价估计结果
        log_file!(
            "000_eval_alloc.txt",
            "func:{},alloc_cost:{}",
            self.label,
            regalloc::eval_alloc(self, &alloc_stat.dstr, &alloc_stat.spillings)
        );

        log_file!(
            "calout.txt",
            "dstr,num:{} :{:?},\nspillings,num:{}:{:?}",
            alloc_stat.dstr.len(),
            alloc_stat.dstr,
            alloc_stat.spillings.len(),
            alloc_stat.spillings
        );
        let file_path = config::get_file_path().unwrap();
        if alloc_stat.spillings.len() == 0 {
            log_file!(
                "./bestalloc.txt",
                "func: {}-{}",
                file_path.to_owned(),
                self.label
            );
        } else {
            log_file!(
                "./badalloc.txt",
                "func:{}-{},dstr/spill:{}",
                file_path.to_owned(),
                self.label,
                alloc_stat.dstr.len() as f32 / alloc_stat.spillings.len() as f32
            );
        }
        let check_alloc_path = "./check_alloc.txt";
        log_file!(check_alloc_path, "{:?}", self.label);
        regalloc::check_alloc_v2(&self, &alloc_stat.dstr, &alloc_stat.spillings);
        // log_file!(
        //     check_alloc_path,
        //     "{:?}",
        //     regalloc::check_alloc(self, &alloc_stat.dstr, &alloc_stat.spillings)
        // );
        // TODO
        // simulate_assign::Simulator::simulate(&self, &alloc_stat);

        self.reg_alloc_info = alloc_stat;
        self.context.as_mut().set_reg_map(&self.reg_alloc_info.dstr);
        // log!("dstr map info{:?}", self.reg_alloc_info.dstr);
        // log!("spills:{:?}", self.reg_alloc_info.spillings);

        // let stack_size = self.max_params * ADDR_SIZE;
        // log!("set stack size:{}", stack_size);
        // self.context.as_mut().set_offset(stack_size);
    }
}
