use super::*;
/// 打印函数当前的汇编形式
impl Func {
    pub fn generate_row(&mut self, f: &mut File) {
        if self.const_array.len() > 0 || self.float_array.len() > 0 {
            writeln!(f, "	.data\n   .align  3").unwrap();
        }
        if self.is_header {
            for mut a in self.const_array.clone() {
                a.generate(self.context, f);
            }
            for mut a in self.float_array.clone() {
                a.generate(self.context, f);
            }
        }
        AsmBuilder::new(f).show_func(&self.label);
        let mut _size = 0;
        for block in self.blocks.iter() {
            _size += block.insts.len();
        }
        for block in self.blocks.iter() {
            block.as_mut().generate_row(ObjPtr::new(&Context::new()), f);
        }
    }

    pub fn print_func(func: ObjPtr<Func>, path: &str) {
        let func_print_path = path.to_string();
        debug_assert!({
            func.as_mut().generate_row(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(func_print_path)
                    .as_mut()
                    .unwrap(),
            );
            true
        });
    }
}

impl Func {
    ///依赖外部的calc live
    pub fn print_live_interval(&self, path: &str) {
        log_file!(path, "func:{}", self.label);
        for bb in self.blocks.iter() {
            log_file!(path, "bb:{}", bb.label);
            log_file!(
                path,
                "live in:{:?}",
                bb.live_in
                    .iter()
                    .map(|reg| reg.to_string(true))
                    .collect::<Vec<String>>()
            );
            log_file!(
                path,
                "in edges:{:?}",
                bb.in_edge
                    .iter()
                    .map(|bb| bb.label.clone())
                    .collect::<Vec<String>>()
            );
            for inst in bb.insts.iter() {
                log_file!(path, "{}", inst.as_ref());
            }
            log_file!(
                path,
                "live out:{:?}",
                bb.live_out
                    .iter()
                    .map(|reg| reg.to_string(true))
                    .collect::<Vec<String>>()
            );
            log_file!(
                path,
                "out edges:{:?}",
                bb.out_edge
                    .iter()
                    .map(|bb| bb.label.clone())
                    .collect::<Vec<String>>()
            );
        }
    }
}
