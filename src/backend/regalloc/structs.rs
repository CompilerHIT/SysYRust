use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;

use crate::backend::block::BB;
use crate::backend::operand::Reg;
use crate::utility::ObjPtr;
use crate::utility::ScalarType;

#[derive(Clone)]
pub struct RegUsedStat {
    iregs_used: u32,
    fregs_used: u32,
}

// 对于regusedstat来说，通用寄存器映射到0-31，浮点寄存器映射到32-63
impl RegUsedStat {
    pub fn new() -> RegUsedStat {
        RegUsedStat {
            iregs_used: 0,
            fregs_used: 0,
        }
    }

    pub fn get_color(reg:&Reg)->i32 {
        if reg.get_id()>=32 {panic!("get color from virtual reg!")}
        match reg.get_type() {
            ScalarType::Float=>reg.get_id()+32,
            ScalarType::Int=>reg.get_id(),
            _=>panic!("tocolor:unlegal type reg")
        }
    }

    

    pub fn is_available_ireg(&self, ireg: i32) -> bool {
        let mut unusable:HashSet<i32>= HashSet::from([0]); //保存x0
        unusable.insert(2); //保留sp
        unusable.insert(1); //保留ra
        unusable.insert(3); //保留gp
        unusable.insert(4); //保留tp寄存器
        unusable.extend(5..=7);//三个临时寄存器用来处理spill逻辑
        unusable.insert(10);        //保留a0
        // unusable.extend(11..=17);   //保留a1-a7
        if unusable.contains(&ireg) {return  false;}
        if (1 << ireg & self.iregs_used) == 0 {
            return true;
        }
        return false;
    }
    pub fn is_available_freg(&self, freg: i32) -> bool {
        let freg=freg-32;
        let mut unusable=HashSet::from([18, 19, 20]);
        if unusable.contains(&freg) {return  false;}
        if (1 << freg & self.fregs_used) == 0 {
            return true;
        }
        return false;
    }


    // 获取可用寄存器数量
    pub fn num_available_regs(&self,kind:ScalarType)->usize {
        match kind {
            ScalarType::Float=>self.num_available_fregs(),
            ScalarType::Int=>self.num_avialable_iregs(),
            _=>panic!("{:?}",kind),
        }
    }
    pub fn num_avialable_iregs(&self)->usize{
        (0..=31).filter(|ireg|self.is_available_ireg(*ireg)).count()
    }
    pub fn num_available_fregs(&self)->usize{
        let m:Range<i32>;
        (32..=63).filter(|freg|self.is_available_freg(*freg)).count()
    }

    // 获取一个可用的整数寄存器
    pub fn get_available_ireg(&self) -> Option<i32> {
        // 对于通用寄存器来说，x0-x4有特殊用途
        // x10-x17用来传递函数参数
        for i in 0..=31 {
            if self.is_available_ireg(i) {
                return  Some(i);
            }
        }
        None
    }

    // 获取一个可用的浮点寄存器
    pub fn get_available_freg(&self) -> Option<i32> {
        // f0作为特殊浮点寄存器保持0
        for i in 32..=63 {
           if self.is_available_freg(i) {
            return  Some(i);
           }
        }
        None
    }

    pub fn get_available_reg(&self,kind:ScalarType) ->Option<i32>{
        match kind {
            ScalarType::Float=>self.get_available_freg(),
            ScalarType::Int=>self.get_available_ireg(),
            _=>panic!("{:?}",kind),
        }
    }

    // 获取剩余的可用通用寄存器
    pub fn get_rest_iregs(&self)->Vec<i32>{
        let mut out=Vec::new();
        for i in 1..=31 {
            if self.is_available_ireg(i){
                out.push(i);
            }
        }
        out
    }

    // 获取剩余的可用浮点寄存器
    pub fn get_rest_fregs(&self)->Vec<i32>{
        let mut out=Vec::new();
        for i in 32..=63 {
            if self.is_available_freg(i){
                out.push(i);
            }
        }
        out
    }


    pub fn release_reg(&mut self,reg:i32){
        if reg>=0&&reg<32 {self.release_ireg(reg);}
        else if reg>=32&&reg<=63 {self.release_freg(reg);}
    }
    pub fn use_reg(&mut self,reg:i32){
        if reg>=0&&reg<32 {self.use_ireg(reg);}
        else if reg>=32&&reg<=63 {self.use_freg(reg);}
    }
    
    pub fn is_available_reg(&self,reg:i32)->bool {
        if reg>=0&&reg<32 {self.is_available_ireg(reg)}
        else if reg>=32&&reg<63 {self.is_available_freg(reg)}
        else {  panic!("not legal reg") }
    }

    // 释放一个通用寄存器
    pub fn release_ireg(&mut self, reg: i32) {
        self.iregs_used &= !(1 << reg);
    }
    // 占有一个整数寄存器
    pub fn use_ireg(&mut self, reg: i32) {
        self.iregs_used |= 1 << reg;
    }
    // 释放浮点寄存器
    pub fn release_freg(&mut self, reg: i32) {
        let reg = reg - 32;
        self.fregs_used &= !(1 << reg);
    }
    // 占有一个浮点寄存器
    pub fn use_freg(&mut self, reg: i32) {
        let reg=reg-32;
        self.fregs_used |= 1 << reg;
    }
}

#[derive(Clone)]
pub struct FuncAllocStat {
    pub stack_size: usize,
    pub bb_stack_sizes: HashMap<ObjPtr<BB>, usize>, //统计翻译bb的时候前面已经用过的栈空间
    pub spillings: HashSet<i32>,                    //spilling regs
    pub dstr: HashMap<i32, i32>,                    //distribute regs
}

impl FuncAllocStat {

    pub fn new() -> FuncAllocStat {
        let mut out = FuncAllocStat {
            spillings: HashSet::new(),
            stack_size: 0,
            bb_stack_sizes: HashMap::new(),
            dstr: HashMap::new(),
        };
        for i in 0..=63 {
            out.dstr.insert(i, i);
        }
        out
    }
}

pub struct BlockAllocStat {
    spillings: HashSet<i32>,
    dstr: HashMap<i32, i32>,
}

impl BlockAllocStat {
    pub fn new() -> BlockAllocStat {
        BlockAllocStat {
            spillings: HashSet::new(),
            dstr: HashMap::new(),
        }
    }
}



#[cfg(test)]
mod test_regusestat{
    use crate::backend::regalloc::regalloc::Regalloc;

    use super::RegUsedStat;

    #[test]
    fn test_num(){
        // TODO
        let a=RegUsedStat::new();
        assert_eq!(a.num_available_fregs(),31);
    }
}