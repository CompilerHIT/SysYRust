use super::*;

impl BackendPass {
    pub fn clear_pass(&mut self) {
        self.module.func_map.iter().for_each(|(_, func)| {
            if !func.is_extern {
                func.blocks.iter().for_each(|block| {
                    self.rm_mv_same(*block);
                })
            }
        });
    }

    fn rm_mv_same(&self, block: ObjPtr<BB>) {
        for inst in block.insts {
            if inst.get_type() == InstrsType::OpReg(SingleOp::IMv) || inst.get_type() == InstrsType::OpReg(SingleOp::FMv) {
                // if inst.get_dst() == 
            }
        }    
    }
}