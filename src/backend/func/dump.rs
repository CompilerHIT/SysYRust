use super::*;
/// 打印函数当前的汇编形式
impl Func {
    pub fn generate_row(&mut self, _: ObjPtr<Context>, f: &mut File) {
        AsmBuilder::new(f).show_func(&self.label);
        // self.context.as_mut().call_prologue_event();
        let mut _size = 0;
        for block in self.blocks.iter() {
            _size += block.insts.len();
        }
        for block in self.blocks.iter() {
            block.as_mut().generate_row(self.context, f);
        }
    }

    pub fn print_func(func: ObjPtr<Func>, path: &str) {
        let func_print_path = path.to_string();
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
    }
}
