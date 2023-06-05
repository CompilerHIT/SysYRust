use std::collections::HashMap;

use crate::{
    ir::{
        basicblock::BasicBlock,
        function::Function,
        instruction::{Inst, InstKind},
        module::Module,
    },
    utility::ObjPtr,
};

use super::irgen::InfuncChoice;

pub struct Context<'a> {
    pub var_map: HashMap<String, Vec<(String, i64)>>,
    pub symbol_table: HashMap<String, Symbol>,
    pub param_usage_table:HashMap<String,bool>,
    pub bb_map: HashMap<String, HashMap<String, ObjPtr<Inst>>>,
    pub bb_now_mut: InfuncChoice,
    pub module_mut: &'a mut Module,
    index: i64,
    layer: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Int,
    Float,
    ConstInt,
    ConstFloat,
}

#[derive(Clone)]
pub struct Symbol {
    pub tp: Type,
    pub is_array: bool,
    pub is_param: bool,
    pub layer: i64,
    pub dimension: Vec<i64>,
}

impl Context<'_> {
    /* -------------------------------------------------------------------------- */
    /*                               constructor                                  */
    /* -------------------------------------------------------------------------- */
    pub fn make_context(module_mut: &mut Module) -> Context {
        Context {
            var_map: HashMap::new(),
            bb_map: HashMap::new(),
            param_usage_table:HashMap::new(),
            bb_now_mut: InfuncChoice::NInFunc(),
            module_mut,
            index: 0,
            layer: -1,
            symbol_table: HashMap::new(),
        }
    }

    /* -------------------------------------------------------------------------- */
    /*                               for bb_now_mut                               */
    /* -------------------------------------------------------------------------- */
    pub fn push_inst_bb(&mut self, inst_ptr: ObjPtr<Inst>) {
        match self.bb_now_mut {
            InfuncChoice::InFunc(bbptr) => {
                let bb = bbptr.as_mut();
                bb.push_back(inst_ptr)
            }
            InfuncChoice::NInFunc() => {}
        }
    }

    pub fn push_var_bb(&mut self, name: String, inst_ptr: ObjPtr<Inst>) {
        match self.bb_now_mut {
            // InfuncChoice::InFunc(bb) => *bb.push_back(inst_ptr),
            InfuncChoice::InFunc(bbptr) => {
                let bb = bbptr.as_mut();
                bb.push_back(inst_ptr)
            }
            InfuncChoice::NInFunc() => self.module_mut.push_var(name, inst_ptr),
        }
    }

    pub fn bb_now_set(&mut self, bb: ObjPtr<BasicBlock>) {
        self.bb_now_mut = InfuncChoice::InFunc(bb);
    }

    // pub fn get_bb_now_ref(&mut self) ->&'static BasicBlock{
    //     match & self.bb_now_mut{
    //         InfuncChoice::InFunc(&bb){

    //         }
    //     }
    // }

    pub fn get_bb_now_name(&mut self) -> String {
        match self.bb_now_mut {
            InfuncChoice::InFunc(bbptr) => {
                let bb = bbptr.as_mut();
                bb.get_name().to_string()
            }
            InfuncChoice::NInFunc() => "global".to_string(),
        }
    }

    /* -------------------------------------------------------------------------- */
    /*                               for module_mut                               */
    /* -------------------------------------------------------------------------- */

    pub fn push_func_module(&mut self, name: String, func_ptr: ObjPtr<Function>) {
        self.module_mut.push_function(name, func_ptr);
    }

    pub fn push_globalvar_module(&mut self, name: String, var_ptr: ObjPtr<Inst>) {
        self.module_mut.push_var(name, var_ptr);
    }

    /* -------------------------------------------------------------------------- */
    /*                               for actionscope                              */
    /* -------------------------------------------------------------------------- */
    pub fn get_var(&self, s: &str, bbname: &str) {}

    // pub fn get_var_changed_name(&self, s: &str) -> String {
    //     if let Some(vec_temp) = self.var_map.get(s.clone()) {
    //         if let Some((last_element0, _last_element1)) = vec_temp.last() {
    //             return last_element0.clone();
    //         }
    //     }
    // }

    pub fn get_var_bbname(&self, s: &str, bbname: &str) -> Option<(ObjPtr<Inst>, Symbol)> {
        if let Some(vec_temp) = self.var_map.get(s.clone()) {
            if let Some((last_element0, _last_element1)) = vec_temp.last() {
                let name = last_element0.clone();
                if let Some(symbol_temp) = self.symbol_table.get(&name) {
                    let symbol = symbol_temp.clone();
                    if let Some(var_inst_map) = self.bb_map.get(bbname) {
                        if let Some(inst) = var_inst_map.get(s.clone()) {
                            let ret_inst = inst.clone();
                            return Option::Some((ret_inst, symbol));
                        }
                    }
                }
            }
        }
        Option::None
    }

    pub fn update_var_scope_now(&mut self, s: &str, inst: ObjPtr<Inst>) -> bool {
        self.update_var_scope(s, inst, self.bb_now_mut)
    }

    pub fn update_var_scope(&mut self, s: &str, inst: ObjPtr<Inst>, bb: InfuncChoice) -> bool {
        let mut bbname = " ";
        // println!("进来了");
        match bb {
            InfuncChoice::InFunc(bbptr) => {
                let bbn = bbptr.as_mut();
                bbname = bbn.get_name();
            }
            InfuncChoice::NInFunc() => {
                if self.get_layer()==-1{
                    bbname = "notinblock";
                    self.push_var_bb(s.to_string(), inst);
                }else if self.get_layer()==0 {
                    bbname = "params";
                }
                // bbname = "notinblock";
                // let kind = inst.as_ref().get_kind();
                // match tp {
                //     Type::ConstFloat =>{
                //         let inst_temp =
                //         self.push_var_bb(s.to_string(), inst);
                //     }
                //     Type::ConstFloat =>{

                //     }
                //     Type::Float =>{

                //     }
                //     Type::Int =>{

                //     }
                //     // InstKind
                //     _=>{}
                // }
                
                
                // self.push_var_bb(s.to_string(), inst);
            }
        }
        if self.var_map.contains_key(s) {
            
            if let Some(vec) = self.var_map.get_mut(s) {
                // println!("进来了");
                // let temps = self.add_prefix(s.to_string()).as_str()
                if let Some(tempvar) = vec.last() {
                    let temps = tempvar.0.clone();
                    if let Some(inst_map) = self.bb_map.get_mut(bbname) {
                        inst_map.insert(temps, inst);
                    } else {
                        let mut map = HashMap::new();
                        map.insert(temps, inst);
                        self.bb_map.insert(bbname.to_string(), map);
                    }
                    // println!("bbname:{:?}插入:{:?}",bbname,s);
                    return true;
                }
            }
        }
        false
    }

    pub fn add_const_int(&mut self, i: i32, inst: ObjPtr<Inst>) -> Option<ObjPtr<Inst>> {
        let s = "@".to_string() + i.to_string().as_str();
        let mut v = vec![];
        let temps = self.add_prefix(s.clone());

        match &mut self.bb_now_mut {
            InfuncChoice::InFunc(bbptr) => {
                let bb = bbptr.as_mut();
                bb.push_back(inst);
                v.push((temps.clone(), 1));
                self.update_var_scope_now(&s, inst); //update global会把var存到module变量作用域中
            }
            InfuncChoice::NInFunc() => {
                // self.push_globalvar_module(temps.clone(), inst);
                v.push((temps.clone(), -1));
            }
        }

        let stemp = s.clone();
        self.var_map.insert(stemp, v);
        self.symbol_table.insert(
            temps.clone(),
            Symbol {
                tp: Type::Int,
                is_array: false,
                is_param: false,
                layer: self.layer,
                dimension: vec![],
            },
        );

        // self.update_var_scope_now(&s, inst);
        self.get_const_int(i)
    }

    pub fn add_const_float(&mut self, f: f32, inst: ObjPtr<Inst>) -> Option<(ObjPtr<Inst>)> {
        let s = "%".to_string() + f.to_string().as_str();
        let mut v = vec![];
        let temps = self.add_prefix(s.clone());
        match &mut self.bb_now_mut {
            InfuncChoice::InFunc(bbptr) => {
                let bb = bbptr.as_mut();
                bb.push_back(inst);
                v.push((temps.clone(), 1));
                self.update_var_scope_now(&s, inst);
            }
            InfuncChoice::NInFunc() => {
                // self.push_globalvar_module(temps.clone(), inst);
                v.push((temps.clone(), -1));
            }
        }
        // v.push((temps.clone(), 1));
        let stemp = s.clone();
        self.var_map.insert(stemp, v);
        self.symbol_table.insert(
            temps.clone(),
            Symbol {
                tp: Type::Float,
                is_array: false,
                is_param:false,
                layer: self.layer,
                dimension: vec![],
            },
        );
        // self.update_var_scope_now(&s, inst);
        self.get_const_float(f)
    }

    pub fn get_const_int(&self, i: i32) -> Option<(ObjPtr<Inst>)> {
        if self.layer > 0 {
            let iname = "@".to_string() + i.to_string().as_str();
            if let Some(vec) = self.var_map.get(&iname) {
                for (name_changed, layer_) in vec {
                    if *layer_ == 1 {
                        for ((bbname, inst_vec)) in &self.bb_map {
                            if let Some(inst) = inst_vec.get(name_changed) {
                                return Option::Some(*inst);
                            }
                        }
                    }
                }
            }
            return Option::None;
        } else {
            let iname = "@".to_string() + i.to_string().as_str();
            if let Some(vec) = self.var_map.get(&iname) {
                for (name_changed, layer_) in vec {
                    if *layer_ == -1 {
                        if let Some(inst_vec) = self.bb_map.get("notinblock") {
                            if let Some(inst) = inst_vec.get(name_changed) {
                                return Option::Some(*inst);
                            }
                        }
                    }
                }
            }
            return Option::None;
        }
    }

    pub fn get_const_float(&self, f: f32) -> Option<(ObjPtr<Inst>)> {
        if self.layer > 0 {
            let iname = "%".to_string() + f.to_string().as_str();
            if let Some(vec) = self.var_map.get(&iname) {
                for (name_changed, layer_) in vec {
                    if *layer_ == 1 {
                        for ((bbname, inst_vec)) in &self.bb_map {
                            if let Some(inst) = inst_vec.get(name_changed) {
                                return Option::Some(*inst);
                            }
                        }
                    }
                }
            }
            return Option::None;
        } else {
            let iname = "%".to_string() + f.to_string().as_str();
            if let Some(vec) = self.var_map.get(&iname) {
                for (name_changed, layer_) in vec {
                    if *layer_ == -1 {
                        if let Some(inst_vec) = self.bb_map.get("notinblock") {
                            if let Some(inst) = inst_vec.get(name_changed) {
                                return Option::Some(*inst);
                            }
                        }
                    }
                }
            }
            return Option::None;
        }
    }

    pub fn add_var(&mut self, s: &str, tp: Type, is_array: bool,is_param:bool, dimension: Vec<i64>) -> bool {
        let s1 = s.clone();
        if (self.has_var_now(s1)) {
            println!("当前作用域中已声明过变量{:?}", s);
            return false;
        }
        let temps = self.add_prefix(s.to_string());
        if self.var_map.contains_key(s) {
            if let Some(vec) = self.var_map.get_mut(s) {
                // let temps = self.add_prefix(s.to_string()).as_str();
                vec.push((temps.to_string(), self.layer));
                self.symbol_table.insert(
                    temps.clone().to_string(),
                    Symbol {
                        tp,
                        is_array,
                        is_param,
                        layer: self.layer,
                        dimension,
                    },
                );
            }
        } else {
            let mut v = vec![];
            // let temps = self.add_prefix(s.to_string());
            v.push((temps.to_string(), self.layer));
            self.var_map.insert(s.to_string(), v);
            self.symbol_table.insert(
                temps.to_string(),
                Symbol {
                    tp,
                    is_array,
                    is_param,
                    layer: self.layer,
                    dimension,
                },
            );
        }

        //for params
        if self.get_layer()==0{
            self.param_usage_table.insert(temps.to_string(), false);
        }

        // if self.layer==-1{
        //     self.update_var_scope_now(s, inst, bb)
        // }
        true
    }

    pub fn add_layer(&mut self) {
        self.layer = self.layer + 1;
    }

    pub fn delete_layer(&mut self) {
        //todo:遍历所有变量，删除layer==layer_now的所有变量
        for (_, vec) in &mut self.var_map {
            loop {
                if let Some(index) = vec.iter().position(|(_, layer)| *layer == self.layer) {
                    let (name_changed, _) = vec.remove(index);
                    self.symbol_table.remove(&name_changed);
                    for (_, inst_map) in &mut self.bb_map {
                        inst_map.remove(&name_changed);
                    }
                } else {
                    break;
                }
            }
        }
        self.layer = self.layer - 1;
    }

    pub fn has_var_now(&self, s: &str) -> bool {
        if let Some(vec_temp) = self.var_map.get(s) {
            for i in vec_temp {
                if i.1 == self.layer {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_layer(&self) -> i64 {
        return self.layer;
    }

    pub fn add_prefix(&mut self, s: String) -> String {
        self.index = self.index + 1;
        self.index.to_string() + s.as_str()
    }
}
