use std::collections::HashMap;

use crate::{
    ir::{
        basicblock::BasicBlock,
        function::Function,
        instruction::Inst,
        module::Module,
    },
    utility::ObjPtr,
};

use super::InfuncChoice;

pub struct Context<'a> {
    pub var_map: HashMap<String, Vec<(String, i64)>>,
    pub symbol_table: HashMap<String, Symbol>,
    pub param_usage_table: HashMap<String, bool>,
    pub bb_map: HashMap<String, HashMap<String, ObjPtr<Inst>>>,
    pub phi_list: HashMap<String, (Vec<(String, ObjPtr<Inst>, bool)>, bool)>,
    pub is_terminated_map: HashMap<String, bool>,
    pub terminated_map: HashMap<String, Vec<(ObjPtr<BasicBlock>, ObjPtr<Inst>)>>,
    pub bb_now_mut: InfuncChoice,
    pub module_mut: &'a mut Module,
    pub stop_genir: bool,
    pub func_now: String,
    pub ret_tp: Type,
    num_bb: i64,
    index: i64,
    layer: i64,
}

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Int,
    Float,
    ConstInt,
    ConstFloat,
    NotForce,
}

#[derive(Clone)]
pub struct Symbol {
    pub tp: Type,
    pub is_array: bool,
    pub is_param: bool,
    pub array_inst: Option<ObjPtr<Inst>>,
    pub global_inst: Option<ObjPtr<Inst>>,
    pub layer: i64,
    pub dimension: Vec<i32>,
}

impl Context<'_> {
    /* -------------------------------------------------------------------------- */
    /*                               constructor                                  */
    /* -------------------------------------------------------------------------- */
    pub fn make_context(module_mut: &mut Module) -> Context {
        Context {
            var_map: HashMap::new(),
            bb_map: HashMap::new(),
            param_usage_table: HashMap::new(),
            phi_list: HashMap::new(),
            is_terminated_map: HashMap::new(),
            terminated_map: HashMap::new(),
            bb_now_mut: InfuncChoice::NInFunc(),
            stop_genir: false,
            module_mut,
            func_now: "none".to_string(),
            ret_tp: Type::NotForce,
            num_bb: 0,
            index: 0,
            layer: -1,
            symbol_table: HashMap::new(),
        }
    }

    /* -------------------------------------------------------------------------- */
    /*                               for bb_now_mut                               */
    /* -------------------------------------------------------------------------- */
    pub fn get_newbb_name(&mut self) -> String {
        self.num_bb = self.num_bb + 1;
        self.num_bb.to_string()
    }

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
        let mut bbname = " ";
        match self.bb_now_mut {
            InfuncChoice::InFunc(bbptr) => {
                let bbn = bbptr.as_mut();
                bbname = bbn.get_name();
                self.update_var_scope(s, inst, &bbname)
            }
            InfuncChoice::NInFunc() => {
                if self.get_layer() == -1 {
                    bbname = "notinblock";
                    self.push_var_bb(s.to_string(), inst);
                } else if self.get_layer() == 0 {
                    bbname = "params";
                }
                self.update_var_scope(s, inst, &bbname)
            }
        }
    }

    pub fn update_var_scope(&mut self, s: &str, inst: ObjPtr<Inst>, bbname: &str) -> bool {
        if self.var_map.contains_key(s) {
            if let Some(vec) = self.var_map.get_mut(s) {
                if let Some(tempvar) = vec.last() {
                    let temps = tempvar.0.clone();
                    if let Some(inst_map) = self.bb_map.get_mut(bbname) {
                        inst_map.insert(temps, inst);
                    } else {
                        let mut map = HashMap::new();
                        map.insert(temps, inst);
                        self.bb_map.insert(bbname.to_string(), map);
                    }
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
                array_inst: None,
                global_inst: None, //可能得改
                layer: self.layer,
                dimension: vec![],
            },
        );

        self.get_const_int(i)
    }

    pub fn add_const_float(&mut self, f: f32, inst: ObjPtr<Inst>) -> Option<ObjPtr<Inst>> {
        let s = "%".to_string() + f.to_string().as_str();
        let mut v = vec![];
        let temps = self.add_prefix(s.clone());
        match &mut self.bb_now_mut {
            InfuncChoice::InFunc(bbptr) => {
                let bb = bbptr.as_mut();
                bb.push_back(inst); //应该往头节点插，往头节点取
                v.push((temps.clone(), 1));
                self.update_var_scope_now(&s, inst);
            }
            InfuncChoice::NInFunc() => {
                v.push((temps.clone(), -1));
            }
        }
        let stemp = s.clone();
        self.var_map.insert(stemp, v);
        self.symbol_table.insert(
            temps.clone(),
            Symbol {
                tp: Type::Float,
                is_array: false,
                is_param: false,
                array_inst: None,
                global_inst: None, //可能得改
                layer: self.layer,
                dimension: vec![],
            },
        );
        self.get_const_float(f)
    }

    pub fn get_const_int(&self, i: i32) -> Option<ObjPtr<Inst>> {
        if self.layer > 0 {
            let iname = "@".to_string() + i.to_string().as_str();
            if let Some(vec) = self.var_map.get(&iname) {
                for (name_changed, layer_now) in vec {
                    if *layer_now == 1 {
                        for (_, inst_vec) in &self.bb_map {
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

    pub fn get_const_float(&self, f: f32) -> Option<ObjPtr<Inst>> {
        if self.layer > 0 {
            let iname = "%".to_string() + f.to_string().as_str();
            if let Some(vec) = self.var_map.get(&iname) {
                for (name_changed, layer_) in vec {
                    if *layer_ == 1 {
                        for (_bbname, inst_vec) in &self.bb_map {
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

    pub fn add_var(
        &mut self,
        s: &str,
        tp: Type,
        is_array: bool,
        is_param: bool,
        array_inst: Option<ObjPtr<Inst>>,
        global_inst: Option<ObjPtr<Inst>>,
        dimension: Vec<i32>,
    ) -> bool {
        let s1 = s.clone();
        if self.has_var_now(s1) {
            return false;
        }
        let temps = self.add_prefix(s.to_string());
        if self.var_map.contains_key(s) {
            if let Some(vec) = self.var_map.get_mut(s) {
                vec.push((temps.to_string(), self.layer));
                self.symbol_table.insert(
                    temps.clone().to_string(),
                    Symbol {
                        tp,
                        is_array,
                        is_param,
                        array_inst,
                        global_inst: global_inst, //可能得改
                        layer: self.layer,
                        dimension,
                    },
                );
            }
        } else {
            let mut v = vec![];
            v.push((temps.to_string(), self.layer));
            self.var_map.insert(s.to_string(), v);
            self.symbol_table.insert(
                temps.to_string(),
                Symbol {
                    tp,
                    is_array,
                    is_param,
                    array_inst,
                    global_inst: global_inst, //可能得改
                    layer: self.layer,
                    dimension,
                },
            );
        }
        if self.get_layer() == 0 {
            self.param_usage_table.insert(temps.to_string(), false);
        }
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
                    vec.remove(index);
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

    pub fn set_stop_genir(&mut self, flag: bool) {
        self.stop_genir = flag;
    }

    pub fn set_funcnow(&mut self, func_name: String) {
        self.func_now = func_name;
    }

    pub fn set_return_type(&mut self, tp: Type) {
        self.ret_tp = tp;
    }
}
