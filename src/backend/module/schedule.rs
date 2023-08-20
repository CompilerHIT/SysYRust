use super::*;

impl AsmModule {
    /// 计算临时变量的个数
    pub fn cal_tmp_var(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().cal_tmp_var();
            }
        });
    }

    /// 清除临时变量
    pub fn clear_tmp_var(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.as_mut().tmp_vars.clear();
            }
        });
    }

    /// 代码调度
    pub fn list_scheduling_tech(&mut self) {
        self.func_map.iter().for_each(|(_, func)| {
            func.as_mut().list_scheduling_tech();
        });
    }
}
