use super::*;
/// 打印函数当前的汇编形式
impl Func {
    pub fn generate_row(&mut self, f: &mut File) {
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
        func.as_mut().generate_row(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(func_print_path)
                .as_mut()
                .unwrap(),
        );
    }
}
