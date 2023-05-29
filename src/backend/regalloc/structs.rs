use std::collections::HashSet;
use std::collections::HashMap;

use crate::backend::block::BB;
use crate::utility::ObjPtr;

#[derive(Clone)]
pub struct RegUsedStat{
    iregs_used: u32,
    fregs_used: u32,
}

impl RegUsedStat {

    pub fn new()->RegUsedStat {
        RegUsedStat { iregs_used: 0, fregs_used: 0 }
    }
    pub fn is_available_ireg(&self,ireg:i32)->bool {
        if (1<<ireg&self.iregs_used) !=0 {
            return  true;
        }
        return  false;
    }
    pub fn is_available_freg(&self,freg:i32)->bool {
        if (1<<freg&self.fregs_used) !=0 {
            return  true;
        }
        return  false;
    }

    pub fn num_available_iregs(&self)->i32 {
        let mut out=0;
        for i in 0..31 {
            if self.is_available_ireg(i) {
                out+=1;
            }
        }
        out
    }
    pub fn num_available_fregs(&self)->i32 {
        let mut out=0;
        for i in 0..31 {
            if self.is_available_freg(i) {
                out+=1;
            }
        }
        out
    }

    // 获取一个可用的整数寄存器
    pub fn get_available_ireg(&self)->Option<i32> {
        // 对于通用寄存器来说，x0-x4有特殊用途
        // x10-x17用来传递函数参数

        if self.iregs_used&(1<<1)==0 {
            // 分配ra,也就是x1
            return Some(1);
        }

        if self.iregs_used&(1<<3)==0 {
            // gp寄存器x3,后面可能保留不分配用来做优化
            return Some(3);
        }
        if self.iregs_used&(1<<8)==0 {
            return Some(8);
        }

        for i in 5..=9 {
            if self.iregs_used&(1<<i)==0 {
                return Some(i)
            }
        }
        // 参数寄存器x10也就是a0保留

        // 但是a1-a7自由使用
        for i in 11..=17 {
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
    pub fn get_available_freg(&self)->Option<i32> {
        // f0作为特殊浮点寄存器保持0
        for i in 1..=31 {
            if self.iregs_used&(1<<i)==0 {
                return Some(i+32)
            }
        }
        None
    }

    // 释放一个寄存器
    pub fn release_ireg(&mut self,reg :i32) {
        self.iregs_used&=!(1<<reg);
    }
    // 占有一个寄存器
    pub fn use_ireg(&mut self,reg :i32){
        self.iregs_used|=1<<reg;
    }
    // 
    pub fn release_freg(&mut self,reg :i32) {
        self.fregs_used&=!(1<<reg);
    }
    pub fn use_freg(&mut self,reg: i32) {
        self.fregs_used|=1<<reg;
    }

}

pub struct FuncAllocStat{
    pub stack_size:usize,
    pub bb_stack_sizes:HashMap<ObjPtr<BB>,usize>,  //统计翻译bb的时候前面已经用过的栈空间
    pub spillings :HashSet<i32>,    //spilling regs
    pub dstr: HashMap<i32,i32>, //distribute regs
}


impl FuncAllocStat {
    pub fn new()->FuncAllocStat {
        let mut out=FuncAllocStat { spillings: HashSet::new(), stack_size:0, bb_stack_sizes:HashMap::new(), dstr: HashMap::new() };
        for i in 0..=63 {
            out.dstr.insert(i, i);
        }
        out
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


