use super::*;
/// 打印函数当前的汇编形式
impl Func {
    pub fn generate_row(&mut self, _: ObjPtr<Context>, f: &mut File) {
        debug_assert!(|| -> bool {
            AsmBuilder::new(f).show_func(&self.label);
            // self.context.as_mut().call_prologue_event();
            let mut _size = 0;
            for block in self.blocks.iter() {
                _size += block.insts.len();
            }
            for block in self.blocks.iter() {
                block.as_mut().generate_row(self.context, f);
            }
            true
        }());
    }

    pub fn print_func(func: ObjPtr<Func>) {
        let func_print_path = "print_func.txt";
        let mut bp = BackendPool::new();
        let context = bp.put_context(Context::new());
        func.as_mut().generate_row(
            context,
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(func_print_path)
                .as_mut()
                .unwrap(),
        );
        bp.free_all();
        // log_file!(func_print_path, "func:{}", self.label);
        // for block in self.blocks.iter() {
        //     log_file!(func_print_path, "\tblock:{}", block.label);
        //     for inst in block.insts.iter() {
        //         log_file!(func_print_path, "\t\t{}", inst.as_ref().to_string());
        //     }
        // }
    }
}
