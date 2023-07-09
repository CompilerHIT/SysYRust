use std::collections::HashMap;

use super::context::Type;
use super::InfuncChoice;
use crate::frontend::context::Context;
use crate::frontend::context::Symbol;
use crate::frontend::error::Error;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
use crate::ir::instruction::InstKind;
use crate::ir::ir_type::IrType;
use crate::utility::ObjPool;
use crate::utility::ObjPtr;
pub struct Kit<'a> {
    pub context_mut: &'a mut Context<'a>,
    pub pool_inst_mut: &'a mut ObjPool<Inst>,
    pub pool_func_mut: &'a mut ObjPool<Function>,
    pub pool_bb_mut: &'a mut ObjPool<BasicBlock>,
}

impl Kit<'_> {
    pub fn init_external_funcs(&mut self) {
        let inst_getint = self.pool_func_mut.new_function();
        inst_getint.as_mut().set_return_type(IrType::Int);

        let inst_getch = self.pool_func_mut.new_function();
        inst_getch.as_mut().set_return_type(IrType::Int);

        let inst_getfloat = self.pool_func_mut.new_function();
        inst_getfloat.as_mut().set_return_type(IrType::Float);

        let inst_getarray = self.pool_func_mut.new_function();
        let param_getarray = self.pool_inst_mut.make_param(IrType::IntPtr);
        inst_getarray
            .as_mut()
            .set_parameter("a".to_string(), param_getarray); //
        inst_getarray.as_mut().set_return_type(IrType::Int);

        let inst_getfarray = self.pool_func_mut.new_function();
        let param_getfarray = self.pool_inst_mut.make_param(IrType::FloatPtr);
        inst_getfarray
            .as_mut()
            .set_parameter("a".to_string(), param_getfarray); //
        inst_getfarray.as_mut().set_return_type(IrType::Int);

        let inst_putint = self.pool_func_mut.new_function();
        let param_putint = self.pool_inst_mut.make_param(IrType::Int);
        inst_putint
            .as_mut()
            .set_parameter("a".to_string(), param_putint); //
        inst_putint.as_mut().set_return_type(IrType::Void);

        let inst_putch = self.pool_func_mut.new_function();
        let param_putch = self.pool_inst_mut.make_param(IrType::Int);
        inst_putch
            .as_mut()
            .set_parameter("a".to_string(), param_putch); //
        inst_putch.as_mut().set_return_type(IrType::Void);

        let inst_putfloat = self.pool_func_mut.new_function();
        let param_putfloat = self.pool_inst_mut.make_param(IrType::Float);
        inst_putfloat
            .as_mut()
            .set_parameter("a".to_string(), param_putfloat); //
        inst_putfloat.as_mut().set_return_type(IrType::Void);

        let inst_putarray = self.pool_func_mut.new_function();
        let param_putarray1 = self.pool_inst_mut.make_param(IrType::Int);
        let param_putarray2 = self.pool_inst_mut.make_param(IrType::IntPtr);
        inst_putarray
            .as_mut()
            .set_parameter("a".to_string(), param_putarray1); //
        inst_putarray
            .as_mut()
            .set_parameter("b".to_string(), param_putarray2); //
        inst_putarray.as_mut().set_return_type(IrType::Void);

        let inst_putfarray = self.pool_func_mut.new_function();
        let param_putfarray1 = self.pool_inst_mut.make_param(IrType::Int);
        let param_putfarray2 = self.pool_inst_mut.make_param(IrType::FloatPtr);
        inst_putfarray
            .as_mut()
            .set_parameter("a".to_string(), param_putfarray1); //
        inst_putfarray
            .as_mut()
            .set_parameter("b".to_string(), param_putfarray2); //
        inst_putfarray.as_mut().set_return_type(IrType::Void);

        let inst_starttime = self.pool_func_mut.new_function();
        inst_starttime.as_mut().set_return_type(IrType::Void);

        let inst_stoptime = self.pool_func_mut.new_function();
        inst_stoptime.as_mut().set_return_type(IrType::Void);

        let inst_sysy_starttime = self.pool_func_mut.new_function();
        inst_sysy_starttime.as_mut().set_return_type(IrType::Void);
        let param_sysy_starttime = self.pool_inst_mut.make_param(IrType::Int);
        inst_sysy_starttime
            .as_mut()
            .set_parameter("a".to_string(), param_sysy_starttime);

        let inst_sysy_stoptime = self.pool_func_mut.new_function();
        inst_sysy_stoptime.as_mut().set_return_type(IrType::Void);
        let param_sysy_stoptime = self.pool_inst_mut.make_param(IrType::Int);
        inst_sysy_stoptime
            .as_mut()
            .set_parameter("a".to_string(), param_sysy_stoptime);

        self.context_mut
            .module_mut
            .push_function("getint".to_string(), inst_getint);
        self.context_mut
            .module_mut
            .push_function("getch".to_string(), inst_getch);
        self.context_mut
            .module_mut
            .push_function("getfloat".to_string(), inst_getfloat);
        self.context_mut
            .module_mut
            .push_function("getarray".to_string(), inst_getarray);
        self.context_mut
            .module_mut
            .push_function("getfarray".to_string(), inst_getfarray);
        self.context_mut
            .module_mut
            .push_function("putint".to_string(), inst_putint);
        self.context_mut
            .module_mut
            .push_function("putch".to_string(), inst_putch);
        self.context_mut
            .module_mut
            .push_function("putfloat".to_string(), inst_putfloat);
        self.context_mut
            .module_mut
            .push_function("putarray".to_string(), inst_putarray);
        self.context_mut
            .module_mut
            .push_function("putfarray".to_string(), inst_putfarray);
        self.context_mut
            .module_mut
            .push_function("starttime".to_string(), inst_starttime);
        self.context_mut
            .module_mut
            .push_function("stoptime".to_string(), inst_stoptime);
        self.context_mut
            .module_mut
            .push_function("_sysy_starttime".to_string(), inst_sysy_starttime);
        self.context_mut
            .module_mut
            .push_function("_sysy_stoptime".to_string(), inst_sysy_stoptime);
    }

    pub fn push_inst(&mut self, inst_ptr: ObjPtr<Inst>) {
        self.context_mut.push_inst_bb(inst_ptr);
    }

    pub fn phi_padding_allfunctions(&mut self) {
        //填充所有函数中的phi
        let vec_funcs = self.get_functions().unwrap().clone();
        for func in vec_funcs {
            if func.is_empty_bb() {
                continue;
            }
            let head_bb_temp = func.as_ref().get_head();
            self.phi_padding_bb(head_bb_temp); //填充该函数中所有bb中的phi
        }
    }

    pub fn get_functions(&self) -> Option<Vec<ObjPtr<Function>>> {
        //获得函数ptr
        if !self.context_mut.module_mut.get_all_func().is_empty() {
            let vec_funcs = self
                .context_mut
                .module_mut
                .get_all_func()
                .iter()
                .map(|(_, y)| *y)
                .collect();
            Some(vec_funcs)
        } else {
            None
        }
    }

    pub fn phi_padding_bb(&mut self, bb: ObjPtr<BasicBlock>) {
        //填充该bb中的phi
        let bbname = bb.get_name();
        let option_phi = self.context_mut.phi_list.get(bbname);
        let mut vec_phi = vec![];
        if let Some((vec_phi_temp, _)) = option_phi {
            vec_phi = vec_phi_temp.clone();
        }
        for (name_changed, inst_phi, _) in vec_phi.clone() {
            self.phi_padding_inst(&name_changed, inst_phi, bb);
        }
        self.context_mut
            .phi_list
            .insert(bbname.to_string(), (vec![], true));
        let bb_success = bb.get_next_bb();
        if bb_success.is_empty() {}
        for bb_next in bb_success {
            if let Some((_, is_padded_temp)) = self.context_mut.phi_list.get(bb_next.get_name()) {
                if *is_padded_temp {
                    continue;
                }
            }
            self.phi_padding_bb(*bb_next);
        }
    }

    pub fn phi_padding_inst(
        &mut self,
        name_changed: &str,
        inst_phi: ObjPtr<Inst>,
        bb: ObjPtr<BasicBlock>,
    ) {
        //填充bb中的变量为name_changed的inst_phi
        let vec_pre = bb.get_up_bb();
        for pre in vec_pre {
            let inst_find = self.find_var(*pre, &name_changed).unwrap();
            inst_phi.as_mut().add_operand(inst_find); //向上找,填充
        }
    }

    pub fn find_var(
        &mut self,
        bb: ObjPtr<BasicBlock>,
        var_name_changed: &str,
    ) -> Result<ObjPtr<Inst>, Error> {
        let bbname = bb.get_name();
        let inst_opt = self
            .context_mut
            .bb_map
            .get(bbname)
            .and_then(|var_inst_map| var_inst_map.get(var_name_changed));
        if let Some(inst_var) = inst_opt {
            Ok(*inst_var)
        } else {
            let sym_opt = self.context_mut.symbol_table.get(var_name_changed);
            if let Some(sym) = sym_opt {
                match sym.tp {
                    Type::ConstFloat | Type::Float => {
                        let inst_phi = self.pool_inst_mut.make_float_phi();
                        bb.as_mut().push_front(inst_phi);
                        //填phi
                        if let Some(inst_map) = self.context_mut.bb_map.get_mut(bbname) {
                            inst_map.insert(var_name_changed.to_string(), inst_phi);
                        } else {
                            let mut map = HashMap::new();
                            map.insert(var_name_changed.to_string(), inst_phi);
                            self.context_mut.bb_map.insert(bbname.to_string(), map);
                        }
                        self.phi_padding_inst(var_name_changed, inst_phi, bb);

                        Ok(inst_phi)
                    }
                    Type::ConstInt | Type::Int => {
                        let inst_phi = self.pool_inst_mut.make_int_phi();
                        bb.as_mut().push_front(inst_phi);

                        if let Some(inst_map) = self.context_mut.bb_map.get_mut(bbname) {
                            inst_map.insert(var_name_changed.to_string(), inst_phi);
                        } else {
                            let mut map = HashMap::new();
                            map.insert(var_name_changed.to_string(), inst_phi);
                            self.context_mut.bb_map.insert(bbname.to_string(), map);
                        }
                        self.phi_padding_inst(var_name_changed, inst_phi, bb);

                        Ok(inst_phi)
                    }
                    _ => {
                        todo!()
                    }
                }
            } else {
                Err(Error::FindVarError)
            }
        }
    }

    pub fn merge_allfunctions(&mut self) {
        //填充所有函数中的phi
        for (func_name, func) in self.context_mut.module_mut.function.clone().iter() {
            if func.is_empty_bb() {
                continue;
            }
            self.merge_function(func_name.to_string(), *func);
        }
    }

    pub fn merge_function(&mut self, func_name: String, inst_func: ObjPtr<Function>) {
        let ret_type = inst_func.get_return_type();
        let vec_endpoint = self
            .context_mut
            .terminated_map
            .get(&func_name)
            .unwrap()
            .clone();
        if vec_endpoint.len() > 1 {
            match ret_type {
                IrType::Void => {
                    let bb_merge_name = self.context_mut.get_newbb_name();
                    let bb_merge = self.pool_bb_mut.new_basic_block(bb_merge_name); //新建汇合点
                    for (endpoint_bb, _) in vec_endpoint {
                        //对于每个终结点
                        let inst_ret = endpoint_bb.get_tail_inst(); //获得ret指令
                        match inst_ret.get_kind() {
                            InstKind::Return => {
                                endpoint_bb.as_mut().add_next_bb(bb_merge); //设置后继节点为汇合点
                                inst_ret.as_mut().remove_self(); //删除ret
                                endpoint_bb
                                    .as_mut()
                                    .push_back(self.pool_inst_mut.make_jmp()); //添加jump
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    bb_merge
                        .as_mut()
                        .push_back(self.pool_inst_mut.make_return_void());
                }
                IrType::Float => {
                    let bb_merge_name = self.context_mut.get_newbb_name();
                    let bb_merge = self.pool_bb_mut.new_basic_block(bb_merge_name); //新建汇合点
                    let inst_phi = self.pool_inst_mut.make_float_phi();
                    bb_merge.as_mut().push_back(inst_phi);
                    for (endpoint_bb, ret_val) in vec_endpoint {
                        //对于每个终结点
                        let inst_ret = endpoint_bb.get_tail_inst(); //获得ret指令
                        match inst_ret.get_kind() {
                            InstKind::Return => {
                                endpoint_bb.as_mut().add_next_bb(bb_merge); //设置后继节点为汇合点
                                inst_ret.as_mut().remove_self(); //删除ret
                                endpoint_bb
                                    .as_mut()
                                    .push_back(self.pool_inst_mut.make_jmp()); //添加jump
                                inst_phi.as_mut().add_operand(ret_val); //向phi指令添加参数
                            }
                            _ => {
                                unreachable!()
                            }
                        }
                    }
                    let inst_ret = self.pool_inst_mut.make_return(inst_phi);
                    bb_merge.as_mut().push_back(inst_ret);
                }
                IrType::Int => {
                    let bb_merge_name = self.context_mut.get_newbb_name();
                    let bb_merge = self.pool_bb_mut.new_basic_block(bb_merge_name); //新建汇合点
                    let inst_phi = self.pool_inst_mut.make_int_phi();
                    bb_merge.as_mut().push_back(inst_phi);
                    for (endpoint_bb, ret_val) in vec_endpoint {
                        //对于每个终结点
                        let inst_ret = endpoint_bb.get_tail_inst(); //获得ret指令
                        match inst_ret.get_kind() {
                            InstKind::Return => {
                                endpoint_bb.as_mut().add_next_bb(bb_merge); //设置后继节点为汇合点
                                inst_ret.as_mut().remove_self(); //删除ret
                                endpoint_bb
                                    .as_mut()
                                    .push_back(self.pool_inst_mut.make_jmp()); //添加jump
                                inst_phi.as_mut().add_operand(ret_val); //向phi指令添加参数
                            }
                            _ => {
                                unreachable!(
                                    "func:{:?}最后一条指令是{:?}类型",
                                    func_name,
                                    inst_ret.get_kind()
                                )
                            }
                        }
                    }
                    let inst_ret = self.pool_inst_mut.make_return(inst_phi);
                    bb_merge.as_mut().push_back(inst_ret);
                }
                _ => {
                    unreachable!()
                }
            }
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
    ) {
        self.context_mut.add_var(
            s,
            tp,
            is_array,
            is_param,
            array_inst,
            global_inst,
            dimension,
        );
    }

    pub fn param_used(&mut self, s: &str) {
        let mut name_changed = " ".to_string();
        let mut layer_var = 0;

        self.context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, layer)| {
                name_changed = last_elm.clone();
                layer_var = *layer;
                self.context_mut.symbol_table.get(last_elm)
            }); //获得改名后的名字

        self.context_mut
            .param_usage_table
            .insert(name_changed, true);
    }

    pub fn push_phi(
        &mut self,
        name: String,
        tp: Type,
        bb: ObjPtr<BasicBlock>,
    ) -> Result<ObjPtr<Inst>, Error> {
        match tp {
            Type::ConstFloat | Type::Float => {
                let inst_phi = self.pool_inst_mut.make_float_phi();
                bb.as_mut().push_front(inst_phi);

                if let Some(inst_map) = self.context_mut.bb_map.get_mut(bb.get_name()) {
                    inst_map.insert(name.clone(), inst_phi);
                } else {
                    let mut map = HashMap::new();
                    map.insert(name.clone(), inst_phi);
                    self.context_mut
                        .bb_map
                        .insert(bb.get_name().to_string(), map);
                }
                if let Some((phi_list, _)) = self.context_mut.phi_list.get_mut(bb.get_name()) {
                    phi_list.push((name, inst_phi, false));
                } else {
                    //如果没有,生成新的philist,插入
                    let mut vec = vec![];
                    vec.push((name, inst_phi, false));
                    self.context_mut
                        .phi_list
                        .insert(bb.get_name().to_string(), (vec, false));
                }
                Ok(inst_phi)
            }
            Type::ConstInt | Type::Int => {
                let inst_phi = self.pool_inst_mut.make_int_phi();
                bb.as_mut().push_front(inst_phi);
                if let Some(inst_map) = self.context_mut.bb_map.get_mut(bb.get_name()) {
                    inst_map.insert(name.clone(), inst_phi);
                } else {
                    let mut map = HashMap::new();
                    map.insert(name.clone(), inst_phi);
                    self.context_mut
                        .bb_map
                        .insert(bb.get_name().to_string(), map);
                }
                if let Some((phi_list, _)) = self.context_mut.phi_list.get_mut(bb.get_name()) {
                    //如果有philist
                    phi_list.push((name, inst_phi, false));
                } else {
                    //如果没有,生成新的philist,插入
                    let mut vec = vec![];
                    vec.push((name, inst_phi, false));
                    self.context_mut
                        .phi_list
                        .insert(bb.get_name().to_string(), (vec, false));
                }
                Ok(inst_phi)
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn get_var_symbol(&mut self, s: &str) -> Result<Symbol, Error> {
        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, _)| self.context_mut.symbol_table.get(last_elm))
            .map(|x| x.clone());
        if let Some(sym) = sym_opt {
            return Ok(sym);
        }
        Err(Error::VariableNotFound)
    }

    pub fn get_var(
        &mut self,
        s: &str,
        offset: Option<ObjPtr<Inst>>,
        bool_get_ptr: bool,
    ) -> Result<(ObjPtr<Inst>, Symbol), Error> {
        match self.context_mut.bb_now_mut {
            InfuncChoice::InFunc(bb) => {
                if let Some((inst, symbol)) = self.get_var_bb(s, bb, offset, bool_get_ptr) {
                    return Ok((inst, symbol));
                }
            }
            InfuncChoice::NInFunc() => {
                let inst = self.context_mut.module_mut.get_var(s);
                let mut name_changed = " ".to_string();
                let mut layer_var = 0;

                let sym_opt = self
                    .context_mut
                    .var_map
                    .get(s)
                    .and_then(|vec_temp| vec_temp.last())
                    .and_then(|(last_elm, layer)| {
                        name_changed = last_elm.clone();
                        layer_var = *layer;
                        self.context_mut.symbol_table.get(last_elm)
                    })
                    .map(|x| x.clone());
                let bbname = "notinblock";

                self
                    .context_mut
                    .bb_map
                    .get(bbname)
                    .and_then(|var_inst_map| var_inst_map.get(&name_changed));

                if let Some(sym) = sym_opt {
                    return Ok((inst, sym));
                }
            }
        }
        return Err(Error::VariableNotFound);
    }

    pub fn get_var_bb(
        &mut self,
        s: &str,
        bb: ObjPtr<BasicBlock>,
        offset: Option<ObjPtr<Inst>>,
        bool_get_ptr: bool,
    ) -> Option<(ObjPtr<Inst>, Symbol)> {
        let mut name_changed = " ".to_string();
        let mut layer_var = 0;

        let sym_opt = self
            .context_mut
            .var_map
            .get(s)
            .and_then(|vec_temp| vec_temp.last())
            .and_then(|(last_elm, layer)| {
                name_changed = last_elm.clone();
                layer_var = *layer;
                self.context_mut.symbol_table.get(last_elm)
            })
            .map(|x| x.clone());

        let mut bbname = bb.as_ref().get_name();
        if layer_var == -1 {
            bbname = "notinblock"; //全局变量,const和一般类型需要分开处理吗?
        }
        //函数参数不再单独处理

        let inst_opt = self
            .context_mut
            .bb_map
            .get(bbname)
            .and_then(|var_inst_map| var_inst_map.get(&name_changed));

        if let Some(sym) = sym_opt {
            //应该先判断是不是数组，以防bbmap中找不到报错
            if let Some(inst_array) = sym.array_inst {
                //如果是数组
                let mut inst_ret = self.pool_inst_mut.make_int_const(-1129);
                match sym.tp {
                    Type::Float | Type::ConstFloat => {
                        //判断类型
                        if layer_var < 0 {
                            //是否是全局
                            if let Some(offset) = offset {
                                //有偏移
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let ptr = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                //没给偏移
                                if bool_get_ptr {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(ptr_array, inst_offset_temp); //获得特定指针
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_offset_temp);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                            }
                        } else {
                            //不是全局
                            if let Some(offset) = offset {
                                //有偏移
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    inst_ret = self.pool_inst_mut.make_gep(inst_array, offset); //获得特定指针
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                    inst_ret = self.pool_inst_mut.make_float_load(ptr);
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                if bool_get_ptr {
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(inst_array, inst_offset_temp); //获得特定指针
                                    self.context_mut.push_inst_bb(inst_offset_temp); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                                //没偏移
                            }
                        }
                    }
                    Type::Int | Type::ConstInt => {
                        if layer_var < 0 {
                            //是否是全局
                            if let Some(offset) = offset {
                                //有偏移
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let ptr = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    inst_ret = self.pool_inst_mut.make_int_load(ptr); //获得元素值
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                //没给偏移
                                if bool_get_ptr {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(ptr_array, inst_offset_temp); //获得特定指针
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_offset_temp);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                            }
                        } else {
                            //不是全局
                            if let Some(offset) = offset {
                                //有偏移
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    inst_ret = self.pool_inst_mut.make_gep(inst_array, offset); //获得特定指针
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                    inst_ret = self.pool_inst_mut.make_int_load(ptr);
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                if bool_get_ptr {
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(inst_array, inst_offset_temp); //获得特定指针
                                    self.context_mut.push_inst_bb(inst_offset_temp); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                                //没偏移
                            }
                        }
                    }
                    _ => {
                        todo!()
                    }
                }
                return Some((inst_ret, sym));
            } else {
                //不是数组的情况
                if layer_var < 0 {
                    //全局也肯定能找到
                    //如果是全局变量
                    //全局变量,const和一般类型需要分开处理吗?
                    //这里返回一条load指令
                    match sym.tp {
                        Type::ConstFloat | Type::Float => {
                            if let Some(inst_global) = sym.global_inst {
                                //全局也肯定能找到
                                let inst_ret =
                                    self.pool_inst_mut.make_global_float_load(inst_global);
                                self.context_mut.push_inst_bb(inst_ret); //这里
                                return Some((inst_ret, sym));
                            }
                        }
                        Type::ConstInt | Type::Int => {
                            if let Some(inst_global) = sym.global_inst {
                                let inst_ret = self.pool_inst_mut.make_global_int_load(inst_global);
                                self.context_mut.push_inst_bb(inst_ret); //这里
                                return Some((inst_ret, sym));
                            }
                        }
                        _ => {
                            todo!()
                        }
                    }
                } else {
                    if let Some(inst) = inst_opt {
                        //找到变量
                        let mut inst_ret = *inst;
                        if layer_var < 0 {
                            //如果是全局变量
                            // bbname = "notinblock";//全局变量,const和一般类型需要分开处理吗?
                            //这里返回一条load指令
                            match sym.tp {
                                Type::ConstFloat | Type::Float => {
                                    inst_ret = self.pool_inst_mut.make_global_float_load(inst_ret);
                                    self.context_mut.push_inst_bb(inst_ret); //这里
                                }
                                Type::ConstInt | Type::Int => {
                                    inst_ret = self.pool_inst_mut.make_global_int_load(inst_ret);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                                _ => {
                                    todo!()
                                }
                            }
                        }

                        return Some((inst_ret, sym));
                    } else {
                        //没找到
                        match sym.tp {
                            Type::ConstFloat | Type::Float => {
                                let phi_inst = self
                                    .push_phi(name_changed.clone(), Type::Float, bb)
                                    .unwrap();
                                return Some((phi_inst, sym));
                            }
                            Type::ConstInt | Type::Int => {
                                let phi_inst =
                                    self.push_phi(name_changed.clone(), Type::Int, bb).unwrap();
                                return Some((phi_inst, sym));
                            }
                            _ => {
                                todo!()
                            }
                        }
                    }
                }
            }
        }
        Option::None
    }
}
