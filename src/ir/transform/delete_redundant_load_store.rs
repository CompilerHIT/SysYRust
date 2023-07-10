use crate::{utility::ObjPtr, ir::{instruction::{Inst, InstKind}, tools::{replace_inst, func_process, bfs_bb_proceess}, module::Module}};

pub fn load_store_opt(module: &mut Module) {
    func_process(module, |_, func| {
            bfs_bb_proceess(func.get_head(), |bb| {
                let mut vec = vec![];
                let mut inst = bb.get_head_inst();
                while !inst.is_tail() {
                    let next = inst.get_next();
                    delete_inst(&mut vec, inst);
                    inst = next;
                }
            });
    });
}

pub fn delete_inst(vec:&mut Vec<ObjPtr<Inst>>,inst:ObjPtr<Inst>)->bool{
    match inst.get_kind(){
        InstKind::Load =>{
            let operands = inst.get_operands();
            for i in vec.clone(){
                let operands_temp = i.get_operands();
                if operands[0]==operands_temp[0]{
                    match i.get_kind(){
                        InstKind::Load =>{
                            replace_inst(inst,i);
                            return true;
                        }
                        InstKind::Store =>{
                            replace_inst(inst,operands_temp[1]);
                            return true;
                        }
                        _=>unreachable!()
                    }
                }
            }
            vec.push(inst);
        }
        InstKind::Store =>{
            let operands = inst.get_operands();
            for i in 0..vec.len(){
                let operands_temp = vec[i].get_operands();
                if operands[0]==operands_temp[0]{
                    match vec[i].get_kind(){
                        InstKind::Load =>{
                            vec[i] = inst;
                            return true;
                        }
                        InstKind::Store =>{
                            replace_inst(vec[i],inst);
                            return true;
                        }
                        _=>unreachable!()
                    }
                }
            }
            vec.push(inst);
        }
        _=>{}
    }
    false
}