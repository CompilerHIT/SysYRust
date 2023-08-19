use std::{collections::{HashMap, HashSet, hash_map::DefaultHasher, btree_map::Entry}, hash::{Hash, Hasher}};

use crate::{
    ir::{
        analysis::{
            call_optimize::call_optimize,
            dominator_tree::{calculate_dominator, DominatorTree},
        },
        instruction::{BinOp, Inst, InstKind, UnOp},
        ir_type::IrType,
        module::Module,
        tools::{bfs_inst_process, func_process, replace_inst},
    },
    utility::ObjPtr,
};

use super::delete_redundant_load_store::load_store_opt;

pub struct CongruenceClass {
    inst_map:HashMap<ObjPtr<Inst>,i32>,
    gep_congruence_map: HashMap<i32,Congruence>,
    pos_congruence_map: HashMap<i32,Congruence>,
    neg_congruence_map: HashMap<i32,Congruence>,
    not_congruence_map: HashMap<i32,Congruence>,
    int_congruence_map: HashMap<i32,Congruence>,
    float_congruence_map: HashMap<i32,Congruence>,
    ftoi_congruence_map: HashMap<i32,Congruence>,
    itof_congruence_map: HashMap<i32,Congruence>,
    add_congruence_map: HashMap<i32,Congruence>,
    sub_congruence_map: HashMap<i32,Congruence>,
    mul_congruence_map: HashMap<i32,Congruence>,
    div_congruence_map: HashMap<i32,Congruence>,
    rem_congruence_map: HashMap<i32,Congruence>,
    ne_congruence_map: HashMap<i32,Congruence>,
    cmp_congruence_map: HashMap<i32,Congruence>,
    call_congruence_map: HashMap<i32,Congruence>,
}
impl CongruenceClass {
    pub fn new() -> CongruenceClass {
        CongruenceClass {
            inst_map: HashMap::new(),
            gep_congruence_map: HashMap::new(),
            pos_congruence_map: HashMap::new(),
            neg_congruence_map: HashMap::new(),
            not_congruence_map: HashMap::new(),
            int_congruence_map: HashMap::new(),
            float_congruence_map: HashMap::new(),
            ftoi_congruence_map: HashMap::new(),
            itof_congruence_map: HashMap::new(),
            add_congruence_map: HashMap::new(),
            sub_congruence_map: HashMap::new(),
            mul_congruence_map: HashMap::new(),
            div_congruence_map: HashMap::new(),
            rem_congruence_map: HashMap::new(),
            ne_congruence_map: HashMap::new(),
            cmp_congruence_map: HashMap::new(),
            call_congruence_map: HashMap::new(),
        }
    }


    pub fn hashcode(&mut self,inst:ObjPtr<Inst>)->i32{
        if let Some(hashcode) = self.inst_map.get(&inst){
            println!("you:{:?}",hashcode);
            return *hashcode;
        }

        match inst.get_kind() {
            // InstKind::Phi =>{
            //     self.inst_map.insert(inst, i);
            //     i
            // }
            InstKind::Alloca(i) | InstKind::ConstInt(i) |InstKind::GlobalConstInt(i)|InstKind::GlobalInt(i)=>{
                self.inst_map.insert(inst, i);
                i
            }
            InstKind::Call(funcname) =>{
                let mut hasher = DefaultHasher::new();
                funcname.hash(&mut hasher);
                let hashcode = hasher.finish() as i32;
                self.inst_map.insert(inst, hashcode);
                hashcode
            }
            InstKind::ConstFloat(f) | InstKind::GlobalConstFloat(f) | InstKind::GlobalFloat(f) =>{
                self.inst_map.insert(inst, f as i32);
                f as i32
            }
            InstKind::Parameter =>{
                self.inst_map.insert(inst, 1129);
                1129
            }
            _=>{
                println!("hash");
                let mut op_hash_vec = vec![];
                for op in inst.get_operands(){
                    println!("op hash");
                    op_hash_vec.push(self.hashcode(*op));
                }
                let mut hashcode = 0;
                for op in op_hash_vec{
                    hashcode += op as i64;
                }
                let hashcode = hashcode as i32;
                self.inst_map.insert(inst, hashcode);
                hashcode
            }
        }
    }

    pub fn get_all_congruence_mut(&mut self) -> Vec<&mut Congruence> {
        let mut vec:Vec<&mut Congruence> = vec![];
        // vec.append(self.add_congruence_map.values_mut().collect());
        self.add_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.sub_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.mul_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.div_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.rem_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.ne_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.cmp_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.call_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.gep_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.pos_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.neg_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.not_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.int_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.float_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.ftoi_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        self.itof_congruence_map.iter_mut().for_each(|(_,cong)|vec.push(cong));
        vec
    }

    pub fn add_congruence(&mut self,inst: ObjPtr<Inst>){
        let hashcode = self.hashcode(inst);
        match inst.get_kind() {
            InstKind::Gep => {self.gep_congruence_map.insert(hashcode, Congruence::new());},
            InstKind::Binary(binop) => match binop {
                BinOp::Eq | BinOp::Ge | BinOp::Gt | BinOp::Le | BinOp::Lt => {self.cmp_congruence_map.insert(hashcode, Congruence::new());},
                BinOp::Add => {self.add_congruence_map.insert(hashcode, Congruence::new());},
                BinOp::Sub => {self.sub_congruence_map.insert(hashcode, Congruence::new());},
                BinOp::Mul => {self.mul_congruence_map.insert(hashcode, Congruence::new());},
                BinOp::Div => {self.div_congruence_map.insert(hashcode, Congruence::new());},
                BinOp::Rem => {self.rem_congruence_map.insert(hashcode, Congruence::new());},
                BinOp::Ne => {self.ne_congruence_map.insert(hashcode, Congruence::new());},
            },
            InstKind::Unary(unop) => match unop {
                UnOp::Pos => {self.pos_congruence_map.insert(hashcode, Congruence::new());},
                UnOp::Neg => {self.neg_congruence_map.insert(hashcode, Congruence::new());},
                UnOp::Not => {self.not_congruence_map.insert(hashcode, Congruence::new());},
            },
            InstKind::ConstInt(_) => {self.int_congruence_map.insert(hashcode, Congruence::new());},
            InstKind::ConstFloat(_) => {self.float_congruence_map.insert(hashcode, Congruence::new());},
            InstKind::FtoI => {self.ftoi_congruence_map.insert(hashcode, Congruence::new());},
            InstKind::ItoF => {self.itof_congruence_map.insert(hashcode, Congruence::new());},
            InstKind::Call(_) => {self.call_congruence_map.insert(hashcode, Congruence::new());},
            _ => {}
        }
    }

    pub fn get_congruence_immut(&self, inst: ObjPtr<Inst>,inst_map:&HashMap<ObjPtr<Inst>,i32>) -> Option<&Congruence> {
        let hashcode = inst_map.get(&inst).unwrap();
        // let hashcode = &self.hashcode(inst);
        match inst.get_kind() {
            InstKind::Gep => self.gep_congruence_map.get(hashcode),
            InstKind::Binary(binop) => match binop {
                BinOp::Eq | BinOp::Ge | BinOp::Gt | BinOp::Le | BinOp::Lt => self.cmp_congruence_map.get(&hashcode),
                BinOp::Add => self.add_congruence_map.get(&hashcode),
                BinOp::Sub => self.sub_congruence_map.get(&hashcode),
                BinOp::Mul => self.mul_congruence_map.get(&hashcode),
                BinOp::Div => self.div_congruence_map.get(&hashcode),
                BinOp::Rem => self.rem_congruence_map.get(&hashcode),
                BinOp::Ne => self.ne_congruence_map.get(&hashcode),
            },
            InstKind::Unary(unop) => match unop {
                UnOp::Pos => self.pos_congruence_map.get(&hashcode),
                UnOp::Neg => self.neg_congruence_map.get(&hashcode),
                UnOp::Not => self.not_congruence_map.get(&hashcode),
            },
            InstKind::ConstInt(_) => self.int_congruence_map.get(&hashcode),
            InstKind::ConstFloat(_) => self.float_congruence_map.get(&hashcode),
            InstKind::FtoI => self.ftoi_congruence_map.get(&hashcode),
            InstKind::ItoF => self.itof_congruence_map.get(&hashcode),
            InstKind::Call(_) => self.call_congruence_map.get(&hashcode),
            _ => {
                None
            }
        }
    }

    pub fn get_congruence_mut(&mut self, inst: ObjPtr<Inst>) -> Option<&mut Congruence> {
        let hashcode = Self::hashcode(self,inst);
        match inst.get_kind() {
            InstKind::Gep => Some(self.gep_congruence_map.get_mut(&hashcode).unwrap()),
            InstKind::Binary(binop) => match binop {
                BinOp::Eq | BinOp::Ge | BinOp::Gt | BinOp::Le | BinOp::Lt => {
                    Some(self.cmp_congruence_map.get_mut(&hashcode).unwrap())
                }
                BinOp::Add => Some(self.add_congruence_map.get_mut(&hashcode).unwrap()),
                BinOp::Sub => Some(self.sub_congruence_map.get_mut(&hashcode).unwrap()),
                BinOp::Mul => Some(self.mul_congruence_map.get_mut(&hashcode).unwrap()),
                BinOp::Div => Some(self.div_congruence_map.get_mut(&hashcode).unwrap()),
                BinOp::Rem => Some(self.rem_congruence_map.get_mut(&hashcode).unwrap()),
                BinOp::Ne => Some(self.ne_congruence_map.get_mut(&hashcode).unwrap()),
            },
            InstKind::Unary(unop) => match unop {
                UnOp::Pos => Some(self.pos_congruence_map.get_mut(&hashcode).unwrap()),
                UnOp::Neg => Some(self.neg_congruence_map.get_mut(&hashcode).unwrap()),
                UnOp::Not => Some(self.not_congruence_map.get_mut(&hashcode).unwrap()),
            },
            InstKind::ConstInt(_) => Some(self.int_congruence_map.get_mut(&hashcode).unwrap()),
            InstKind::ConstFloat(_) => Some(self.float_congruence_map.get_mut(&hashcode).unwrap()),
            InstKind::FtoI => Some(self.ftoi_congruence_map.get_mut(&hashcode).unwrap()),
            InstKind::ItoF => Some(self.itof_congruence_map.get_mut(&hashcode).unwrap()),
            InstKind::Call(_) => Some(self.call_congruence_map.get_mut(&hashcode).unwrap()),
            _ => None,
        }
    }

    pub fn remove_inst(&mut self, inst: ObjPtr<Inst>) {
        match inst.get_kind() {
            InstKind::Alloca(_)
            | InstKind::Branch
            | InstKind::Head
            | InstKind::Parameter
            | InstKind::Return
            | InstKind::Store
            | InstKind::Load
            | InstKind::GlobalConstFloat(_)
            | InstKind::GlobalConstInt(_)
            | InstKind::GlobalFloat(_)
            | InstKind::Phi
            | InstKind::GlobalInt(_) => {
                return;
            }
            _ => {}
        }
        let congruence = self.get_congruence_mut(inst).unwrap();
        let index1 = congruence.map.get(&inst).unwrap();
        if let Some(index2) = congruence.vec_class[*index1]
            .iter()
            .position(|&x| x == inst)
        {
            congruence.vec_class[*index1].remove(index2);
        }
        congruence.map.remove(&inst);
    }

    pub fn add_inst(&mut self, inst: ObjPtr<Inst>) {
        match inst.get_kind() {
            InstKind::Alloca(_)
            | InstKind::Branch
            | InstKind::Head
            | InstKind::Parameter
            | InstKind::Return
            | InstKind::Store
            | InstKind::Load
            | InstKind::GlobalConstFloat(_)
            | InstKind::GlobalConstInt(_)
            | InstKind::GlobalFloat(_)
            | InstKind::Phi
            | InstKind::GlobalInt(_) => {
                return;
            }
            _ => {}
        }
        self.hashcode(inst);
        match self.get_congruence_immut(inst,&self.inst_map) {
            Some(_) =>{}
            None =>{
                self.add_congruence(inst);
            }
        }
        let congruence = self.get_congruence_immut(inst,&self.inst_map).unwrap();
        if let Some(_index) = congruence.map.get(&inst) {
            return;
        }
        let mut index_final = 0;
        for vec_congruent in &congruence.vec_class {
            if vec_congruent.len() == 0 {
                continue;
            }
            if compare_two_inst(inst, vec_congruent[0], &self) {
                if let Some(index) = congruence.map.get(&vec_congruent[0]) {
                    index_final = index + 1;
                }
                if index_final != 0 {
                    let congruence_mut = self.get_congruence_mut(inst).unwrap();
                    congruence_mut.vec_class[index_final - 1].push(inst);
                    congruence_mut.map.insert(inst, index_final - 1);
                }
                return;
            }
        }
        let congruence = self.get_congruence_mut(inst).unwrap();
        let index = congruence.vec_class.len();
        congruence.vec_class.push(vec![inst]); //加入新的congruent class
        congruence.map.insert(inst, index); //增加索引映射
    }
}

pub struct Congruence {
    pub vec_class: Vec<Vec<ObjPtr<Inst>>>,
    pub map: HashMap<ObjPtr<Inst>, usize>,
}

impl Congruence {
    pub fn new() -> Congruence {
        Congruence {
            vec_class: vec![],
            map: HashMap::new(),
        }
    }

    pub fn remove_inst(&mut self, inst: ObjPtr<Inst>) {
        if let Some(index1) = self.map.get(&inst) {
            if let Some(index2) = self.vec_class[*index1].iter().position(|&x| x == inst) {
                self.vec_class[*index1].remove(index2);
            }
            self.map.remove(&inst);
        }
    }

    pub fn add_inst(&mut self, inst: ObjPtr<Inst>, index: usize) {
        if let Some(_index) = self.map.get(&inst) {
            return;
        }
        self.map.insert(inst, index); //增加索引映射
        self.vec_class[index].push(inst);
    }
}

pub fn gvn(module: &mut Module, opt_option: bool) -> Option<Vec<CongruenceClass>> {
    if opt_option {
        loop {
            let mut changed = false;
            let (gvn_changed, vec_congruence_class) = easy_gvn(module);
            changed |= gvn_changed;
            changed |= load_store_opt(module);
            if !changed {
                return Some(vec_congruence_class);
            }
        }
    }
    None
}

pub fn easy_gvn(module: &mut Module) -> (bool, Vec<CongruenceClass>) {
    let mut vec_congruence_class = vec![];
    let mut changed = false;
    let set = call_optimize(module);
    func_process(module, |_, func| {
        let mut congruence_class = CongruenceClass::new();
        let dominator_tree = calculate_dominator(func.get_head());
        bfs_inst_process(func.get_head(), |inst| {
            changed |= has_val(&mut congruence_class, inst, &dominator_tree, &set)
        });
        vec_congruence_class.push(congruence_class);
    });
    (changed, vec_congruence_class)
}

pub fn has_val(
    congruence_class: &mut CongruenceClass,
    inst: ObjPtr<Inst>,
    dominator_tree: &DominatorTree,
    set: &HashSet<String>,
) -> bool {
    match inst.get_kind() {
        InstKind::Alloca(_)
        | InstKind::Branch
        | InstKind::Head
        | InstKind::Parameter
        | InstKind::Return
        | InstKind::Store
        | InstKind::Load
        | InstKind::GlobalConstFloat(_)
        | InstKind::GlobalConstInt(_)
        | InstKind::GlobalFloat(_)
        // | InstKind::Phi
        | InstKind::GlobalInt(_) => {
            return false;
        } //todo:phi可以被优化吗
        InstKind::Phi => {
            let operands = inst.get_operands();
            let len = operands.len();
            if is_the_same(operands.clone()) {
                //操作数属于同一类型
                let mut flag = true;
                let mut index = 0;
                for j in 0..len {
                    flag &= compare_two_inst(operands[0], operands[j], congruence_class); //操作数一一同质
                    if dominator_tree
                        .is_dominate(&operands[j].get_parent_bb(), &inst.get_parent_bb())
                    {
                        index = j + 1;
                    }
                }
                if flag && index > 0 {
                    //操作数一一同质且有操作数所在节点支配当前节点
                    replace_inst(inst, operands[index - 1]); //用该操作数替换phi节点
                    return true;
                }
            }
            return false;
        }
        InstKind::Call(funcname) => {
            congruence_class.hashcode(inst);
            match congruence_class.get_congruence_immut(inst,&congruence_class.inst_map) {
                Some(_) =>{}
                None =>{
                    congruence_class.add_congruence(inst);
                }
            }
            let congruence = congruence_class.get_congruence_immut(inst,&congruence_class.inst_map).unwrap(); //副本
            if set.contains(&funcname) {
                //纯函数，可复用
                if let Some(_index) = congruence.map.get(&inst) {
                    return false;
                }
                let mut index_final = 0;
                for vec_congruent in &congruence.vec_class {
                    if compare_two_inst(inst, vec_congruent[0], congruence_class) {
                        if dominator_tree
                            .is_dominate(&vec_congruent[0].get_parent_bb(), &inst.get_parent_bb())
                        {
                            replace_inst(inst, vec_congruent[0]);
                            return true;
                        } else {
                            for i in 1..vec_congruent.len() {
                                if dominator_tree.is_dominate(
                                    &vec_congruent[i].get_parent_bb(),
                                    &inst.get_parent_bb(),
                                ) {
                                    replace_inst(inst, vec_congruent[i]);
                                    return true;
                                }
                            }
                        }
                        //都没有可以替代这条指令的congruent inst,将这条指令加入congruent inst中

                        if let Some(index) = congruence.map.get(&vec_congruent[0]) {
                            index_final = index + 1;
                        }
                        if index_final != 0 {
                            let congruence_mut = congruence_class.get_congruence_mut(inst).unwrap();
                            congruence_mut.vec_class[index_final - 1].push(inst);
                            congruence_mut.map.insert(inst, index_final - 1);
                        }
                        return false;
                    }
                }
                let congruence = congruence_class.get_congruence_mut(inst).unwrap();
                let index = congruence.vec_class.len();
                congruence.vec_class.push(vec![inst]); //加入新的congruent class
                congruence.map.insert(inst, index); //增加索引映射
            }
            return false;
        }
        _ => {
            congruence_class.hashcode(inst);
            match congruence_class.get_congruence_immut(inst,&congruence_class.inst_map) {
                Some(_) =>{}
                None =>{
                    congruence_class.add_congruence(inst);
                }
            }
            let congruence = congruence_class.get_congruence_immut(inst,&congruence_class.inst_map).unwrap();
            if let Some(_index) = congruence.map.get(&inst) {
                return false;
            }
            let mut index_final = 0;
            for vec_congruent in &congruence.vec_class {
                if compare_two_inst(inst, vec_congruent[0], congruence_class) {
                    if dominator_tree
                        .is_dominate(&vec_congruent[0].get_parent_bb(), &inst.get_parent_bb())
                    {
                        replace_inst(inst, vec_congruent[0]);
                        return true;
                    } else {
                        for i in 1..vec_congruent.len() {
                            if dominator_tree.is_dominate(
                                &vec_congruent[i].get_parent_bb(),
                                &inst.get_parent_bb(),
                            ) {
                                replace_inst(inst, vec_congruent[i]);
                                return true;
                            }
                        }
                    }
                    //都没有可以替代这条指令的congruent inst,将这条指令加入congruent inst中
                    if let Some(index) = congruence.map.get(&vec_congruent[0]) {
                        index_final = index + 1;
                    }
                    if index_final != 0 {
                        let congruence_mut = congruence_class.get_congruence_mut(inst).unwrap();
                        congruence_mut.vec_class[index_final - 1].push(inst);
                        congruence_mut.map.insert(inst, index_final - 1);
                    }
                    return false;
                }
            }
        }
    }
    let congruence = congruence_class.get_congruence_mut(inst).unwrap();
    let index = congruence.vec_class.len();
    congruence.vec_class.push(vec![inst]); //加入新的congruent class
    congruence.map.insert(inst, index); //增加索引映射
    false
}

pub fn is_the_same(inst_vec: Vec<ObjPtr<Inst>>) -> bool {
    let first = inst_vec[0].get_kind();
    for i in inst_vec {
        if i.get_kind() != first {
            return false;
        }
    }
    match first {
        InstKind::Alloca(_)
        | InstKind::Branch
        | InstKind::Head
        | InstKind::Parameter
        | InstKind::Return
        | InstKind::Store
        | InstKind::Load
        | InstKind::GlobalConstFloat(_)
        | InstKind::GlobalConstInt(_)
        | InstKind::GlobalFloat(_)
        | InstKind::Phi
        | InstKind::GlobalInt(_) => {
            return false;
        } //todo:phi可以被优化吗
        _ => {
            return true;
        }
    }
}

pub fn compare_two_inst(
    inst1: ObjPtr<Inst>,
    inst2: ObjPtr<Inst>,
    congrunce_class: &CongruenceClass,
) -> bool {
    let tpflag = inst1.get_ir_type() == inst2.get_ir_type();
    if inst1.get_kind() == inst2.get_kind() && tpflag {
        match inst1.get_kind() {
            InstKind::Gep => {
                let operands1 = inst1.get_operands();
                let operands2 = inst2.get_operands();
                return compare_two_inst_with_index(operands1[0], operands2[0], congrunce_class)
                    && compare_two_inst_with_index(operands1[1], operands2[1], congrunce_class);
            }
            InstKind::Unary(unop1) => match inst2.get_kind() {
                InstKind::Unary(unop2) => {
                    let operands1 = inst1.get_operands();
                    let operands2 = inst2.get_operands();
                    return unop1 == unop2
                        && compare_two_inst_with_index(
                            operands1[0],
                            operands2[0],
                            congrunce_class,
                        );
                }
                _ => unreachable!(),
            },
            InstKind::ConstInt(i1) => match inst2.get_kind() {
                InstKind::ConstInt(i2) => {
                    if i1 == i2 {
                        return true;
                    } else {
                        return false;
                    }
                }
                _ => {
                    unreachable!()
                }
            },
            InstKind::ConstFloat(f1) => match inst2.get_kind() {
                InstKind::ConstFloat(f2) => {
                    if f1 == f2 {
                        return true;
                    } else {
                        return false;
                    }
                }
                _ => {
                    unreachable!()
                }
            },
            InstKind::FtoI => {
                let operands1 = inst1.get_operands();
                let operands2 = inst2.get_operands();
                return compare_two_inst_with_index(operands1[0], operands2[0], congrunce_class);
            }
            InstKind::ItoF => {
                let operands1 = inst1.get_operands();
                let operands2 = inst2.get_operands();
                return compare_two_inst_with_index(operands1[0], operands2[0], congrunce_class);
            }
            InstKind::Binary(binop1) => match inst2.get_kind() {
                InstKind::Binary(binop2) => {
                    if binop1 == binop2 && inst1.get_ir_type() == IrType::Int {
                        match binop1 {
                            BinOp::Add | BinOp::Eq | BinOp::Mul | BinOp::Ne => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                return compare_two_operands(operands1, operands2, congrunce_class);
                            }
                            _ => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                if compare_two_inst_with_index(
                                    operands1[0],
                                    operands2[0],
                                    congrunce_class,
                                ) && compare_two_inst_with_index(
                                    operands1[1],
                                    operands2[1],
                                    congrunce_class,
                                ) {
                                    return true;
                                } else {
                                    return false;
                                }
                            }
                        }
                    } else if binop1 == binop2 && inst1.get_ir_type() == IrType::Float {
                        match binop1 {
                            BinOp::Eq | BinOp::Ne => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                return compare_two_operands(operands1, operands2, congrunce_class);
                            }
                            _ => {
                                let operands1 = inst1.get_operands();
                                let operands2 = inst2.get_operands();
                                if compare_two_inst_with_index(
                                    operands1[0],
                                    operands2[0],
                                    congrunce_class,
                                ) && compare_two_inst_with_index(
                                    operands1[1],
                                    operands2[1],
                                    congrunce_class,
                                ) {
                                    return true;
                                } else {
                                    return false;
                                }
                            }
                        }
                    } //todo:比较指令
                }
                _ => unreachable!(),
            },
            _ => {}
        }
    } else if tpflag {
        match inst1.get_kind() {
            InstKind::Binary(binop1) => match inst2.get_kind() {
                InstKind::Binary(binop2) => {
                    let operands1 = inst1.get_operands();
                    let operands2 = inst2.get_operands();
                    if (binop1 == BinOp::Ge && binop2 == BinOp::Lt)
                        || (binop1 == BinOp::Gt && binop2 == BinOp::Le)
                        || (binop1 == BinOp::Le && binop2 == BinOp::Gt)
                        || (binop1 == BinOp::Lt && binop2 == BinOp::Ge)
                    {
                        return compare_two_inst_with_index(
                            operands1[0],
                            operands2[1],
                            congrunce_class,
                        ) && compare_two_inst_with_index(
                            operands1[1],
                            operands2[0],
                            congrunce_class,
                        );
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    false
}

pub fn compare_two_inst_with_index(
    inst1: ObjPtr<Inst>,
    inst2: ObjPtr<Inst>,
    congrunce_class: &CongruenceClass,
) -> bool {
    match inst1.get_kind() {
        InstKind::Alloca(_)
        | InstKind::Branch
        | InstKind::Head
        | InstKind::Parameter
        | InstKind::Return
        | InstKind::Store
        | InstKind::Load
        | InstKind::GlobalConstFloat(_)
        | InstKind::GlobalConstInt(_)
        | InstKind::GlobalFloat(_)
        | InstKind::GlobalInt(_)
        | InstKind::Phi => {
            return inst1 == inst2;
        }
        _ => match inst2.get_kind() {
            InstKind::Alloca(_)
            | InstKind::Branch
            | InstKind::Head
            | InstKind::Parameter
            | InstKind::Return
            | InstKind::Store
            | InstKind::Load
            | InstKind::GlobalConstFloat(_)
            | InstKind::GlobalConstInt(_)
            | InstKind::GlobalFloat(_)
            | InstKind::GlobalInt(_)
            | InstKind::Phi => {
                return inst1 == inst2;
            }
            _ => {}
        },
    }
    if inst1 == inst2 {
        //针对全局指针
        return true;
    }
    // match congrunce_class.get_congruence_immut(inst1,&congrunce_class.inst_map) {
    //     Some(_) =>{}
    //     None =>{
    //         congrunce_class.add_congruence(inst1);
    //     }
    // }
    let congruence = congrunce_class.get_congruence_immut(inst1,&congrunce_class.inst_map).unwrap();

    if let Some(index1) = congruence.map.get(&inst1) {
        if let Some(index2) = congruence.map.get(&inst2) {
            //如果不是同一类则获得不了索引
            if index1 == index2 {
                return true;
            }
        }
    }
    false
}

pub fn compare_two_operands(
    operands1: &Vec<ObjPtr<Inst>>,
    operands2: &Vec<ObjPtr<Inst>>,
    congrunce_class: &CongruenceClass,
) -> bool {
    if compare_two_inst_with_index(operands1[0], operands2[0], congrunce_class)
        && compare_two_inst_with_index(operands1[1], operands2[1], congrunce_class)
    {
        return true;
    } else if compare_two_inst_with_index(operands1[1], operands2[0], congrunce_class)
        && compare_two_inst_with_index(operands1[0], operands2[1], congrunce_class)
    {
        return true;
    }
    false
}
