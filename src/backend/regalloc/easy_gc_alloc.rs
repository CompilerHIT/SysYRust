// a impl of graph color register alloc algo

use std::collections::{HashSet, HashMap, VecDeque};

use crate::{backend::instrs::{Func, BB}, utility::{ObjPool, ObjPtr}};

use super::{regalloc::Regalloc, easy_ls_alloc, structs::FuncAllocStat};

pub struct Allocator {
    regs:Vec<i32>,  //所有虚拟寄存器的列表
    icolors:HashMap<i32,i32>,    //整数寄存器分配的着色
    fcolors:HashMap<i32,i32>,   //  浮点寄存器分配的着色
    f_interference_graph:HashMap<i32,HashSet<i32>>,   //浮点寄存器冲突图
    i_interference_graph:HashMap<i32,HashSet<i32>>,   //浮点寄存器冲突图
    dstr:HashMap<i32,i32>,  //记录每个寄存器分配到的实际寄存器
    spillings:HashSet<i32>, //记录溢出寄存器
}


impl  Allocator {
    fn new()->Allocator{
        Allocator { regs:Vec::new(), icolors:HashMap::new(), fcolors: HashMap::new(), i_interference_graph: HashMap::new(),f_interference_graph:HashMap::new()
            , dstr: HashMap::new(), spillings: HashSet::new() }
    }


    // 建立虚拟寄存器之间的冲突图
    fn build_interference_graph(&mut self,func:&Func){
        // 遍历所有块,得到所有虚拟寄存器和所有虚拟寄存器之间的冲突关系
        let mut que:VecDeque<ObjPtr<BB>>=VecDeque::new();    //广度优先遍历块用到的队列




    }
    
    // 寻找最小度寄存器进行着色,作色成功返回true,着色失败返回true
    fn color(&mut self)->bool{
        // 着色完全后停止,或者是
        false
    }

    // 简化成功返回true,简化失败返回falses
    fn simplify(&mut self)->bool{
        false
    }

    // 简化失败后执行溢出操作,选择一个节点进行溢出,然后对已经作色节点进行重新分配
    fn spill(&mut self){

    }

    // 返回分配结果
    fn alloc_register(&mut self)->(HashSet<i32>, HashMap<i32, i32>){
        let mut spillings=HashSet::new();
        let mut dstr=HashMap::new();
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
