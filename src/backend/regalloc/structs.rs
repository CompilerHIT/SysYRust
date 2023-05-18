use std::collections::HashSet;
use std::collections::HashMap;

#[derive(Clone)]
pub struct RegUsedStat{
    iregs_used: u32,
    fregs_used: u32,
}

impl RegUsedStat {

    pub fn new()->RegUsedStat {
        RegUsedStat { iregs_used: 0, fregs_used: 0 }
    }
    fn is_available_ireg(&self,ireg:i32)->bool {
        if (1<<ireg&self.iregs_used) !=0 {
            return  true;
        }
        return  false;
    }
    fn is_available_freg(&self,freg:i32)->bool {
        if (1<<freg&self.fregs_used) !=0 {
            return  true;
        }
        return  false;
    }

    // 获取一个可用的整数寄存器
    fn get_available_ireg(&self)->Option<i32> {
        // 对于通用寄存器来说，x0-x4有特殊用途
        // x10-x17用来传递函数参数

        // if self.iregs_used&(1<<3)==0 {
        //     // gp寄存器x3保留来做优化
        //     return Some(3)
        // }
        for i in 5..=9 {
            if self.iregs_used&(1<<i)==0 {
                return Some(i)
            }
        }
        for i in 18..=31 {
            if self.iregs_used&(1<<i)==0 {
                return Some(i)
            }
        }
        None
    }
    
    // 获取一个可用的浮点寄存器
    fn get_available_freg(&self)->Option<i32> {
        // f0作为特殊浮点寄存器保持0
        for i in 1..=31 {
            if self.iregs_used&(1<<i)==0 {
                return Some(i)
            }
        }
        None
    }

    // 释放一个寄存器
    fn release_ireg(&mut self,reg :i32) {
        self.iregs_used&=!(1<<reg);
    }
    // 占有一个寄存器
    fn use_ireg(&mut self,reg :i32){
        self.iregs_used|=1<<reg;
    }
    // 
    fn release_freg(&mut self,reg :i32) {
        self.fregs_used&=!(1<<reg);
    }
    fn use_freg(&mut self,reg: i32) {
        self.fregs_used|=1<<reg;
    }


}

pub struct FuncAllocStat{
    pub spillings :HashSet<i32>,    //spilling regs
    pub dstr: HashMap<i32,i32>, //distribute regs
}


impl FuncAllocStat {
    pub fn new()->FuncAllocStat {
        FuncAllocStat { spillings: HashSet::new(), dstr: HashMap::new() }
    }
}

pub struct BlockAllocStat{
    spillings :HashSet<i32>,
    dstr: HashMap<i32,i32>,
}

impl  BlockAllocStat {
    pub fn new()->BlockAllocStat {
        BlockAllocStat { spillings: HashSet::new(), dstr: HashMap::new()}
    }
}