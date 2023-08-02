use super::*;

impl AsmModule {
    pub fn rm_inst_suf_p2v(&mut self, pool: &mut BackendPool) {
        for (name, func) in self.name_func.iter() {
            func.as_mut().remove_unuse_inst_suf_v2p(pool);
        }
    }
}
