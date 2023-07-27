use std::collections::{HashMap, HashSet, LinkedList};

use crate::{
    backend::{instrs::LIRInst, operand::Reg},
    log_file,
    utility::ObjPtr,
};

///记录源代码路径
static mut SRC_PATH: Option<String> = None;

///记录需要保存打印的各种信息
///统计优化后的性能,记录产生的各种属性
/// 1. spill数量
/// 2. 访问内存下标溢出数量
/// 3. 长跳转插入数量
/// 4. caller save的保存和恢复数量
/// 5. callee save的保存和恢复数量
/// 信息格式: {信息要输出到的文件名}-{函数名}-{块名}-{信息}
struct ConfigInfo {
    file_infos: HashMap<String, LinkedList<String>>, //记录要写入的文件以及要往文件中写入的信息 (默认是append模式)
    times: HashMap<String, i32>,                     //统计各种事件次数
    baned_set: HashSet<String>,
}
impl ConfigInfo {
    pub fn new() -> ConfigInfo {
        ConfigInfo {
            file_infos: HashMap::new(),
            times: HashMap::new(),
            baned_set: HashSet::new(),
        }
    }
}

static mut CONFIG_INFO: Option<ConfigInfo> = None;

pub fn set_file_path(path: &String) {
    unsafe { SRC_PATH = Some(String::from(path)) };
}

pub fn get_file_path() -> Option<String> {
    if unsafe { SRC_PATH.is_none() } {
        return None;
    } else {
        return Some(unsafe {
            let str = SRC_PATH.to_owned().unwrap();
            str
        });
    }
}

///init:初始化,只能够调用一次
pub fn init() {
    unsafe {
        if CONFIG_INFO.is_none() {
            CONFIG_INFO = Some(ConfigInfo::new());
            let info = CONFIG_INFO.as_mut().unwrap();
            info.times.insert("spill".to_string(), 0);
            info.times.insert("offset_overflow".to_string(), 0);
            info.times.insert("branch_overflow".to_string(), 0);
            info.times.insert("caller_save".to_string(), 0);
            info.times.insert("callee_save".to_string(), 0);
        }
        if SRC_PATH.is_none() {
            SRC_PATH = Some(String::from("default.sy"));
        }
    }
}

///把信息打印出来
pub fn dump() {
    init();
    for (file, infos) in unsafe { CONFIG_INFO.as_ref().unwrap().file_infos.iter() } {
        if unsafe { CONFIG_INFO.as_ref().unwrap().baned_set.contains(file) } {
            continue;
        }
        // 打印信息
        for info in infos.iter() {
            log_file!(file, "{info}");
        }
    }

    //统计的总属性输出到一个专门的文件中 (粒度到源文件)
    unsafe {
        log_file!(
            "performance_eval.txt",
            "sy_path:{}",
            SRC_PATH.as_ref().unwrap().clone()
        );
        let order = vec![
            "spill",
            "offset_overflow",
            "branch_overflow",
            "caller_save",
            "callee_save",
        ];
        for kind in order.iter() {
            let times = CONFIG_INFO
                .as_ref()
                .unwrap()
                .times
                .get(&kind.to_string())
                .unwrap();
            log_file!("performance_eval.txt", "{}\t:{} times", kind, times);
        }
    }
}

///记录在ban列表中的文件就不会被打印
pub fn ban(path: &str) {
    init();
    unsafe {
        CONFIG_INFO
            .as_mut()
            .unwrap()
            .baned_set
            .insert(path.to_string())
    };
}

///每次发生调用一次,
pub fn record_spill(func: &str, block: &str, msg: &str) {
    init();
    let path = "spill.txt";
    let kind = "spill";
    unsafe {
        let info = CONFIG_INFO.as_mut().unwrap();
        if !info.file_infos.contains_key(&path.to_string()) {
            info.file_infos.insert(path.to_string(), LinkedList::new());
        }
        info.times.insert(
            kind.to_string(),
            *info.times.get(&kind.to_string()).unwrap_or(&0) + 1,
        );
        let msg = format!("{}-{} :{}", func, block, msg);
        info.file_infos
            .get_mut(&path.to_string())
            .unwrap()
            .push_back(msg);
    }
}

///每次发生调用一次
pub fn record_offset_overflow(func: &str, block: &str, msg: &str) {
    init();
    let path = "offset_overflow.txt";
    let kind = "offset_overflow";
    unsafe {
        let info = CONFIG_INFO.as_mut().unwrap();
        if !info.file_infos.contains_key(&path.to_string()) {
            info.file_infos.insert(path.to_string(), LinkedList::new());
        }
        info.times.insert(
            kind.to_string(),
            *info.times.get(&kind.to_string()).unwrap_or(&0) + 1,
        );
        let msg = format!("{}-{} :{}", func, block, msg);
        info.file_infos
            .get_mut(&path.to_string())
            .unwrap()
            .push_back(msg);
    }
}

pub fn record_branch_overflow(func: &str, block: &str, msg: &str) {
    init();
    let path = "branch_overflow.txt";
    let kind = "branch_overflow";
    unsafe {
        let info = CONFIG_INFO.as_mut().unwrap();
        if !info.file_infos.contains_key(&path.to_string()) {
            info.file_infos.insert(path.to_string(), LinkedList::new());
        }
        info.times.insert(
            kind.to_string(),
            *info.times.get(&kind.to_string()).unwrap_or(&0) + 1,
        );
        let msg = format!("{}-{} :{}", func, block, msg);
        info.file_infos
            .get_mut(&path.to_string())
            .unwrap()
            .push_back(msg);
    }
}

pub fn record_caller_save_sl(func: &str, block: &str, msg: &str) {
    init();
    let path = "caller_save.txt";
    let kind = "caller_save";
    unsafe {
        let info = CONFIG_INFO.as_mut().unwrap();
        if !info.file_infos.contains_key(&path.to_string()) {
            info.file_infos.insert(path.to_string(), LinkedList::new());
        }
        info.times.insert(
            kind.to_string(),
            *info.times.get(&kind.to_string()).unwrap_or(&0) + 1,
        );
        let msg = format!("{}-{} :{}", func, block, msg);
        info.file_infos
            .get_mut(&path.to_string())
            .unwrap()
            .push_back(msg);
    }
}

pub fn record_callee_save_sl(func: &str, msg: &str) {
    init();
    let path = "callee_save.txt";
    let kind = "callee_save";
    unsafe {
        let info = CONFIG_INFO.as_mut().unwrap();
        if !info.file_infos.contains_key(&path.to_string()) {
            info.file_infos.insert(path.to_string(), LinkedList::new());
        }
        info.times.insert(
            kind.to_string(),
            *info.times.get(&kind.to_string()).unwrap_or(&0) + 1,
        );
        let msg = format!("{}:{}", func, msg);
        info.file_infos
            .get_mut(&path.to_string())
            .unwrap()
            .push_back(msg);
    }
}

//实现一个全局寄存器表
static mut STR_REG: Option<HashMap<String, Reg>> = None;
static mut STR_INST: Option<HashMap<String, ObjPtr<crate::backend::instrs::LIRInst>>> = None;

fn init_str_reg() {
    unsafe {
        if STR_REG.is_none() {
            STR_REG = Some(HashMap::new());
        }
    }
}

pub fn set_reg(key: &str, reg: &Reg) {
    init_str_reg();
    unsafe {
        STR_REG.as_mut().unwrap().insert(key.to_string(), *reg);
    }
}

pub fn get_reg(key: &str) -> Option<Reg> {
    init_str_reg();
    let reg = unsafe { STR_REG.as_ref().unwrap().get(key) };
    if reg.is_none() {
        return None;
    }
    Some(*reg.unwrap())
}

pub fn set_inst(key: &str, reg: &ObjPtr<LIRInst>) {
    init_str_reg();
    unsafe {
        STR_INST.as_mut().unwrap().insert(key.to_string(), *reg);
    }
}
pub fn get_inst(key: &str) -> Option<ObjPtr<LIRInst>> {
    init_str_reg();
    let out = unsafe { STR_INST.as_ref().unwrap().get(key) };
    if out.is_none() {
        return None;
    }
    let out = out.unwrap();
    Some(*out)
}
