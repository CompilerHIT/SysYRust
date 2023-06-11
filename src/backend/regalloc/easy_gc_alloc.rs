// a impl of graph color register alloc algo

use std::collections::{HashSet, HashMap, VecDeque};

use crate::{backend::{instrs::{Func, BB}, operand::Reg}, utility::{ObjPool, ObjPtr, ScalarType}};

use super::{regalloc::Regalloc, easy_ls_alloc, structs::{FuncAllocStat, RegUsedStat}};

pub struct Allocator {
    i_regs:Vec<i32>,  //所有虚拟通用寄存器的列表
    f_regs:Vec<i32>,    //所有虚拟浮点寄存器列表
    icolors:HashMap<i32,i32>,    //整数寄存器分配的着色
    fcolors:HashMap<i32,i32>,   //  浮点寄存器分配的着色
    i_alloc_finish:bool,
    f_alloc_finish:bool,
    i_interence_reg:Option<i32>,   //记录冲突的通用寄存器
    f_interence_reg:Option<i32>,    
    f_interference_graph:HashMap<i32,HashSet<i32>>,   //浮点寄存器冲突图
    i_interference_graph:HashMap<i32,HashSet<i32>>,   //浮点寄存器冲突图
    dstr:HashMap<i32,i32>,  //记录每个寄存器分配到的实际寄存器
    spillings:HashSet<i32>, //记录溢出寄存器
}


impl  Allocator {
    fn new()->Allocator{
        Allocator { i_regs:Vec::new(),f_regs:Vec::new(), icolors:HashMap::new(), fcolors: HashMap::new(), i_interference_graph: HashMap::new(),f_interference_graph:HashMap::new()
            ,i_alloc_finish:false,f_alloc_finish:true,
            i_interence_reg:None,f_interence_reg:None,
             dstr: HashMap::new(), spillings: HashSet::new() }
    }

    // 判断两个虚拟寄存器是否是通用寄存器分配冲突
    fn is_i_interference(reg1:i32,reg2:i32)->bool{
        todo!()
    }
    // 判断两个虚拟寄存器是否是浮点寄存器分配冲突
    fn is_f_interference(reg1:i32, reg2:i32)->bool{
        todo!()
    }


    // 建立虚拟寄存器之间的冲突图
    fn build_interference_graph(&mut self,func:&Func){
        // 遍历所有块,得到所有虚拟寄存器和所有虚拟寄存器之间的冲突关系
        let mut que:VecDeque<ObjPtr<BB>>=VecDeque::new();    //广度优先遍历块用到的队列
        let mut passed:HashSet<ObjPtr<BB>>=HashSet::new();
        que.push_front(func.entry.unwrap());
        let mut passed_reg:HashSet<Reg>=HashSet::new();
        while(!que.is_empty()){
            let cur_bb=que.pop_front().unwrap();
            if passed.contains(&cur_bb) {
                continue;
            }
            passed.insert(cur_bb);

            // 把不同类型寄存器统计加入表中
            for inst in cur_bb.insts.iter() {
                for reg in inst.get_reg_def() {
                    if passed_reg.contains(&reg) {continue;}
                    passed_reg.insert(reg);
                    if reg.get_type()==ScalarType::Int {
                        self.i_regs.push(reg.get_id());
                    }else{
                        self.f_regs.push(reg.get_id());
                    }
                }
                for reg in inst.get_reg_use() {
                    if passed_reg.contains(&reg) {continue;}
                    passed_reg.insert(reg);
                    if reg.get_type()==ScalarType::Int {
                        self.i_regs.push(reg.get_id());
                    }else{
                        self.f_regs.push(reg.get_id());
                    }
                }
            }
            // 定义处理函数,对于整数情况进行处理
            let process=|colors:&mut HashMap<i32,i32>,interef_graph:&mut HashMap<i32,HashSet<i32>>,kind: ScalarType|{
                let mut livenow:HashSet<i32>=HashSet::new();
                // 先对所有live in 且live out进行处理
                for reg in cur_bb.live_in.iter() {
                    if(self.spillings.contains(&reg.get_id())) {continue;}
                    if(!reg.is_virtual()||reg.get_type()!=kind) {continue;}
                    if(!cur_bb.live_out.contains(reg)) {continue;}
                    livenow.insert(reg.get_id());
                }
                for id in &livenow {
                    if let Some(tos)=interef_graph.get(&id) {
                        continue;
                    }
                    interef_graph.insert(*id, HashSet::new());
                }
                // 遍历所有指令,找到冲突的寄存器
                for inst in cur_bb.insts.iter() {
                    // 同时处于live in 和live out 中的寄存器不会与其他寄存器合并
                    for reg in inst.get_reg_def() {
                        if self.spillings.contains(&reg.get_id()) {continue;}
                        if reg.get_type()!=kind {continue;}
                        if !reg.is_virtual() {continue;}
                        // 加入冲突列表
                        for id in &livenow {
                            if *id==reg.get_id() {continue;}
                            interef_graph.get_mut(id).unwrap().insert(reg.get_id());
                            // 加入反向冲突列表
                            if let Some(tos)=interef_graph.get_mut(&reg.get_id()) {
                                tos.insert(*id);
                            }else{
                                let mut tos=HashSet::new();
                                tos.insert(reg.get_id());
                                interef_graph.insert(reg.get_id(), tos);
                            }
                        }
                    }
                }

                // 然后对所有live in 进行处理,直到找到最后一次use,然后往前
                livenow.clear();
                for id in &livenow {
                    if let Some(tos)=interef_graph.get(&id) {
                        continue;
                    }
                    interef_graph.insert(*id, HashSet::new());
                }
                // 从后往前遍历指令
                for (_,inst) in cur_bb.insts.iter().enumerate().rev(){
                    // 先刷新活着的寄存器
                    for reg in inst.get_reg_use() {
                        if self.spillings.contains(&reg.get_id()) {continue;}
                        if !reg.is_virtual() ||reg.get_type()!=kind {continue;}
                        if(!cur_bb.live_in.contains(&reg)) {continue;}
                        livenow.insert(reg.get_id());
                    }
                    // 更新冲突
                    for reg in inst.get_reg_def() {
                        if self.spillings.contains(&reg.get_id()) {continue;}
                        if !reg.is_virtual() ||reg.get_type()!=kind {continue;}
                        //
                        if livenow.contains(&reg.get_id()) {continue;}
                        // 否则就与live now 冲突
                        for id in &livenow {
                            interef_graph.get_mut(id).unwrap().insert(reg.get_id());
                            if let Some(tos)=interef_graph.get_mut(&reg.get_id()) {
                                tos.insert(*id);
                            }else{
                                let mut tos:HashSet<i32>=HashSet::new();
                                tos.insert(*id);
                                interef_graph.insert(reg.get_id(), tos);
                            }
                        }
                    }
                }

                // 对所有live out 进行处理,从前往后,找到第一个def之后
                livenow.clear();
                for id in &livenow {
                    if let Some(tos)=interef_graph.get(&id) {
                        continue;
                    }
                    interef_graph.insert(*id, HashSet::new());
                }

                // 从前往后遍历,只处理在cur_bb中定义并且live out的寄存器的冲突关系
                for inst in cur_bb.insts.iter() {
                    // 找到第一次定义的时候,并且更新冲突
                    for reg in inst.get_reg_def() {
                        if self.spillings.contains(&reg.get_id()) {continue;}
                        if reg.get_type()!=kind ||!reg.is_virtual() {continue;}
                        // 判断是否在live out中
                        if cur_bb.live_out.contains(&reg) {
                            livenow.insert(reg.get_id());
                            continue;
                        }
                        // 判断是否在live now中
                        if livenow.contains(&reg.get_id()) {
                            panic!("理论上这里不应该包含");
                        }else{
                            // 否则与live now中的内容冲突
                            for id in &livenow {
                                interef_graph.get_mut(id).unwrap().insert(reg.get_id());
                                if let Some(tos)=interef_graph.get_mut(&reg.get_id()) {
                                    tos.insert(*id);
                                }else{
                                    let mut tos:HashSet<i32>=HashSet::new();
                                    tos.insert(*id);
                                    interef_graph.insert(reg.get_id(), tos);
                                }
                            }
                        }
                    }
                }

            };
            //分别处理浮点寄存器的情况和通用寄存器的情况
            process(&mut self.icolors,&mut self.i_interference_graph,ScalarType::Int);
            process(&mut self.fcolors,&mut self.f_interference_graph,ScalarType::Float);
            // 加入还没有处理过的bb
            for bb_next in cur_bb.out_edge.iter() {
                if passed.contains(bb_next) {continue;}
                que.push_back(*bb_next);
            }
        }
    }
    
    // 寻找最小度寄存器进行着色,作色成功返回true,着色失败返回true
    fn color(&mut self)->bool{
        // 开始着色,着色直到无法再找到可着色的点为止,如果全部点都可以着色,返回true,否则返回false
        let mut out=false;
        // TODO
        // 对通用虚拟寄存器进行着色 (已经着色的点不用着色)
        if !self.i_alloc_finish {
            self.i_alloc_finish=true;
            for reg in self.i_regs.iter() {
                //
                let mut usestat=RegUsedStat::new();
                if let Some(neighbors)=self.i_interference_graph.get(reg){
                    for neighbor in neighbors {
                        if !self.icolors.contains_key(neighbor) {continue;}
                        usestat.use_ireg(*self.icolors.get(neighbor).unwrap());
                    }
                }
                let color=usestat.get_available_ireg();
                if let Some(color)=color {
                    self.icolors.insert(*reg, color);
                }else{
                    self.i_alloc_finish=false;
                    out=false;
                    self.i_interence_reg=Some(*reg);
                    break;
                }
            }
        }
        
        if !self.f_alloc_finish {
            self.f_alloc_finish=true;
            // 对虚拟浮点寄存器进行着色
            for reg in self.f_regs.iter() {
                let mut usestat=RegUsedStat::new();
                if let Some(neighbors)=self.f_interference_graph.get(reg){
                    for neighbor in neighbors {
                        if !self.fcolors.contains_key(neighbor) {continue;}
                        usestat.use_freg(*self.fcolors.get(neighbor).unwrap());
                    }
                }
                let color=usestat.get_available_freg();
                if let Some(color)=color {
                    self.fcolors.insert(*reg, color);
                }else{
                    self.f_alloc_finish=false;
                    out=false;
                    self.f_interence_reg=Some(*reg);
                    break;
                }
            }
        }
        out
    }

    // 简化成功返回true,简化失败返回falses
    fn simplify(&mut self)->bool{
        // 简化方式
        // 处理产生冲突的寄存器
        // 方式,周围的寄存器变换颜色,直到该寄存器有不冲突颜色
        
        // 对通用寄存器的化简
        if let Some(i_reg)=self.i_interence_reg {
            let neighbors=self.i_interference_graph.get(&i_reg).unwrap();
            // 对周围的所有寄存器尝试更换颜色
            for reg in neighbors {
                let available_colors:HashSet<i32>=HashSet::new();
                // TODO
                // 获取所有能够去尝试使用的颜色

            }
        }
        // 对浮点寄存器的化简
        if let Some(f_reg)=self.f_interence_reg {
             
        }
        false   //化简失败
    }

    // 简化失败后执行溢出操作,选择一个节点进行溢出,然后对已经作色节点进行重新分配
    fn spill(&mut self){
        // 选择冲突寄存器或者是周围寄存器中的一个进行溢出,
        // 溢出的选择贪心: f=cost/degree.
        // 选择使得贪心函数最小的一个
        // TODO

    }

    // 返回分配结果
    fn alloc_register(&mut self)->(HashSet<i32>, HashMap<i32, i32>){
        let mut spillings:HashSet<i32>=HashSet::new();
        let mut dstr:HashMap<i32, i32>=HashMap::new();
        for r in self.spillings.iter() {
            spillings.insert(*r);
        }
        for (k,v) in self.dstr.iter() {
            dstr.insert(*k, *v);
        }
        (spillings,dstr)

    }


}

impl Regalloc for Allocator {
    fn alloc(&mut self, func: &crate::backend::func::Func) -> super::structs::FuncAllocStat {
        self.build_interference_graph(func);
        while(!self.color()){
            if(self.simplify()) {
                continue;
            }
            self.spill();
        }
        let (spillings,dstr)=   self.alloc_register();
        let (func_stack_size,bb_sizes)=easy_ls_alloc::Allocator::countStackSize(func, &spillings);
        FuncAllocStat{
            stack_size: func_stack_size,
            bb_stack_sizes: bb_sizes,
            spillings,
            dstr,
        }
    }
}
