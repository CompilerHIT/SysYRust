use std::{
    collections::{HashMap, HashSet, LinkedList},
    fmt::format,
    time::{self, Duration, Instant},
};

use crate::{
    backend::{instrs::LIRInst, operand::Reg},
    ir::instruction::Inst,
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
    start_time: Instant,
}
impl ConfigInfo {
    pub fn new() -> ConfigInfo {
        ConfigInfo {
            file_infos: HashMap::new(),
            times: HashMap::new(),
            baned_set: HashSet::new(),
            start_time: Instant::now(),
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
            info.times.insert("mem_rearrange".to_string(), 0);
            info.times.insert("reg_merge".to_string(), 0);
        }
        if SRC_PATH.is_none() {
            SRC_PATH = Some(String::from("default.sy"));
        }
    }
}

pub fn get_passed_secs() -> usize {
    let info = unsafe { CONFIG_INFO.as_ref() };
    debug_assert!(info.is_some());
    let info = info.as_ref().unwrap();
    let passed = info.start_time.elapsed().as_secs();
    passed as usize
}
pub fn get_passed_time() -> Duration {
    let info = unsafe { CONFIG_INFO.as_ref() };
    debug_assert!(info.is_some());
    let info = info.as_ref().unwrap();
    info.start_time.elapsed()
}

static mut TIME_LIMIT_SECS: usize = 0;
pub fn set_time_limit_secs(limit: usize) {
    unsafe { TIME_LIMIT_SECS = limit };
}
pub fn get_time_limit_secs() -> usize {
    unsafe { TIME_LIMIT_SECS }
}

///获取剩余秒数
pub fn get_rest_secs() -> usize {
    init();
    let passed = get_passed_secs();
    let limit = unsafe { TIME_LIMIT_SECS };
    if limit > passed {
        limit - passed
    } else {
        0
    }
}

pub fn record_event(event: &str) {
    init();
    let path = "events.txt";
    let msg = format!("{} at:{}s", event, get_passed_secs());
    // println!("{}", msg);
    let info = unsafe { CONFIG_INFO.as_mut().unwrap() };
    if !info.file_infos.contains_key(path) {
        info.file_infos.insert(path.to_string(), LinkedList::new());
    }
    info.file_infos.get_mut(path).unwrap().push_back(msg);
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
        let performance_path = "performance_eval.txt";
        log_file!(
            performance_path,
            "sy_path:{}",
            SRC_PATH.as_ref().unwrap().clone()
        );
        let order = vec![
            "spill",
            "offset_overflow",
            "branch_overflow",
            "caller_save",
            "callee_save",
            "mem_rearrange",
            "reg_merge",
        ];
        for kind in order.iter() {
            let times = CONFIG_INFO
                .as_ref()
                .unwrap()
                .times
                .get(&kind.to_string())
                .unwrap();
            log_file!(performance_path, "{}\t:{} times", kind, times);
        }
        //打印栈重排效果,
    }
}

///not log ban
pub fn dump_not_log(performance_path: &str) {
    macro_rules! log_file {
    ($file:expr, $($arg:tt)*) => {{
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open($file)
        .expect("Failed to open log file");

        writeln!(file, $($arg)*).expect("Failed to write to log file");
    }};
    }
    init();
    unsafe {
        for msg in CONFIG_INFO
            .as_ref()
            .unwrap()
            .file_infos
            .get("events.txt")
            .unwrap()
        {
            log_file!("events.txt", "{msg}");
        }
    }

    //统计的总属性输出到一个专门的文件中 (粒度到源文件)
    unsafe {
        log_file!(
            performance_path,
            "sy_path:{}",
            SRC_PATH.as_ref().unwrap().clone()
        );
        let order = vec![
            "spill",
            "offset_overflow",
            "branch_overflow",
            "caller_save",
            "callee_save",
            "mem_rearrange",
            "reg_merge",
        ];
        for kind in order.iter() {
            let times = CONFIG_INFO
                .as_ref()
                .unwrap()
                .times
                .get(&kind.to_string())
                .unwrap();
            log_file!(performance_path, "{}\t:{} times", kind, times);
        }
        //打印栈重排效果,
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

pub fn record_mem_rearrange(func: &str, old_mem: usize, new_mem: usize) {
    init();
    let path = "rearrange_mem.txt".to_string();
    let info = unsafe { CONFIG_INFO.as_mut().unwrap() };
    if !info.file_infos.contains_key(path.as_str()) {
        info.file_infos.insert(path.clone(), LinkedList::new());
    }
    info.file_infos
        .get_mut(path.as_str())
        .unwrap()
        .push_back(format!("realloc mem func{}:{}/{}", func, old_mem, new_mem).to_string());
    let time = info.times.get_mut("mem_rearrange").unwrap();
    *time += old_mem as i32 - new_mem as i32;
}

///记录寄存器合并所在的函数,并记录该次寄存器合并
pub fn record_merge_reg(func: &str, reg1: &Reg, reg2: &Reg) {
    init();
    let path = "reg_merge.txt";
    let kind = "reg_merge";
    let info = unsafe { CONFIG_INFO.as_mut().unwrap() };
    if !info.file_infos.contains_key(&path.to_string()) {
        info.file_infos.insert(path.to_string(), LinkedList::new());
    }
    info.times.insert(
        kind.to_string(),
        *info.times.get(&kind.to_string()).unwrap_or(&0) + 1,
    );
    let msg = format!("merge {reg1}{reg2} in {func}");
    info.file_infos
        .get_mut(&path.to_string())
        .unwrap()
        .push_back(msg);
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
