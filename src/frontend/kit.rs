use std::collections::HashMap;

use super::context::Type;
use super::InfuncChoice;
use crate::frontend::context::Context;
use crate::frontend::context::Symbol;
use crate::frontend::error::Error;
use crate::ir::basicblock::BasicBlock;
use crate::ir::function::Function;
use crate::ir::instruction::Inst;
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
    }

    pub fn push_inst(&mut self, inst_ptr: ObjPtr<Inst>) {
        self.context_mut.push_inst_bb(inst_ptr);
    }

    pub fn phi_padding_allfunctions(&mut self) {
        //填充所有函数中的phi
        // println!("填phi开始");
        let mut vec_funcs = self.get_functions().unwrap().clone();
        for func in vec_funcs {
            // println!(
            //     "填phi,函数头basicblock名:{:?}",
            //     func.as_ref().get_head().get_name()
            // );
            if func.is_empty_bb() {
                continue;
            }
            let head_bb_temp = func.as_ref().get_head();
            self.phi_padding_bb(head_bb_temp); //填充该函数中所有bb中的phi
        }
    }

    pub fn get_functions(&self) -> Option<Vec<(ObjPtr<Function>)>> {
        //获得函数ptr
        let mut vec_funcs = vec![];
        if !self.context_mut.module_mut.get_all_func().is_empty() {
            vec_funcs = self
                .context_mut
                .module_mut
                .get_all_func()
                .iter()
                .map(|(x, y)| *y)
                .collect();
            Some(vec_funcs)
        } else {
            None
        }
    }

    pub fn phi_padding_bb(&mut self, bb: ObjPtr<BasicBlock>) {
        //填充该bb中的phi
        // println!("bbnow:name:{:?}", bb.get_name());
        let bbname = bb.get_name();
        let option_phi = self.context_mut.phi_list.get(bbname);
        let mut vec_phi = vec![];
        let mut is_padded = false;
        if let Some((vec_phi_temp, is_padded_temp)) = option_phi {
            // println!("有phi");
            vec_phi = vec_phi_temp.clone();
            is_padded = *is_padded_temp;
        } else {
            // println!("没phi");
        }
        if !is_padded {
            //没被填过
            // println!("phi_list长度:{:?}", vec_phi.len());
            for (name_changed, inst_phi, phi_is_padded) in vec_phi.clone() {
                // println!("填phi{:?}:{:?}", name_changed, inst_phi.get_kind());
                self.phi_padding_inst(&name_changed, inst_phi, bb);
            }
        } else {
            // println!("phi填过了,跳过");
        }
        if !is_padded {
            self.context_mut
                .phi_list
                .insert(bbname.to_string(), (vec![], true));
        }
        //判断是否是最后一个bb
        let bb_success = bb.get_next_bb();
        for bb_next in bb_success {
            if bb_next.get_name() == bb.get_name() {
                // println!("自己插过phi了");
                continue;
            }
            if let Some((vec_phi_temp, is_padded_temp)) =
                self.context_mut.phi_list.get(bb_next.get_name())
            {
                // println!("有phi");
                vec_phi = vec_phi_temp.clone();
                is_padded = *is_padded_temp;
                if is_padded {
                    continue;
                }
            } else {
                // println!("没phi");
                continue;
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
        // println!("填phi:{:?},所在bb:{:?}", name_changed, bb.get_name());
        let vec_pre = bb.get_up_bb();
        for pre in vec_pre {
            let inst_find = self.find_var(*pre, &name_changed).unwrap();
            inst_phi.as_mut().add_operand(inst_find); //向上找,填充
            // println!("其参数为:{:?}", inst_find.get_kind());
        }
    }

    pub fn find_var(
        &mut self,
        bb: ObjPtr<BasicBlock>,
        var_name_changed: &str,
    ) -> Result<ObjPtr<Inst>, Error> {
        // println!("在bb:{:?}中找", bb.get_name());
        let bbname = bb.get_name();
        let inst_opt = self
            .context_mut
            .bb_map
            .get(bbname)
            .and_then(|var_inst_map| var_inst_map.get(var_name_changed));
        if let Some(inst_var) = inst_opt {
            // println!("找到了,返回{:?}", inst_var.get_kind());
            Ok(*inst_var)
        } else {
            // println!("没找到,插phi");
            let sym_opt = self.context_mut.symbol_table.get(var_name_changed);
            if let Some(sym) = sym_opt {
                // let inst_phi = self
                //     .push_phi(var_name_changed.to_string(), sym.tp, bb)
                //     .unwrap();
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
                        // self.context_mut
                        //     .bb_map
                        //     .get_mut(bbname)
                        //     .and_then(|var_inst_map_insert| {
                        //         var_inst_map_insert.insert(var_name_changed.to_string(), inst_phi)
                        //     });

                        Ok(inst_phi)
                    }
                    Type::ConstInt | Type::Int => {
                        let inst_phi = self.pool_inst_mut.make_int_phi();
                        bb.as_mut().push_front(inst_phi);
                        //填phi
                        // println!("没找到,向{:?}里插phi", bb.get_name());
                        // self.context_mut.update_var_scope(
                        //     var_name_changed,
                        //     inst_phi,
                        //     bb.get_name(),
                        // );

                        if let Some(inst_map) = self.context_mut.bb_map.get_mut(bbname) {
                            inst_map.insert(var_name_changed.to_string(), inst_phi);
                        } else {
                            let mut map = HashMap::new();
                            map.insert(var_name_changed.to_string(), inst_phi);
                            self.context_mut.bb_map.insert(bbname.to_string(), map);
                        }
                        self.phi_padding_inst(var_name_changed, inst_phi, bb);
                        // self.context_mut
                        //     .bb_map
                        //     .get_mut(bbname)
                        //     .and_then(|var_inst_map_insert| {
                        //         var_inst_map_insert.insert(var_name_changed.to_string(), inst_phi)
                        //     });

                        Ok(inst_phi)
                    }
                    _=>{
                        todo!()
                    }
                }
            } else {
                // println!("没找到符号{:?}", var_name_changed);
                // println!("符号表长度:{:?}", self.context_mut.symbol_table.len());
                Err(Error::FindVarError)
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
        // let inst = self.context_mut.module_mut.get_var(s);

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

    // pub fn update_var(&mut self, s: &str, inst: ObjPtr<Inst>) -> bool {
    //     self.context_mut.update_var_scope(s, inst)
    // }

    pub fn push_phi(
        &mut self,
        name: String,
        tp: Type,
        bb: ObjPtr<BasicBlock>,
    ) -> Result<ObjPtr<Inst>, Error> {
        match tp {
            Type::ConstFloat | Type::Float => {
                // println!()
                let inst_phi = self.pool_inst_mut.make_float_phi();
                // println!("指令{:?}插入bb{:?}中", inst_phi.get_kind(), bb.get_name());
                bb.as_mut().push_front(inst_phi);
                // self.context_mut
                //     .update_var_scope(name.as_str(), inst_phi, bb.get_name());

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
                    // println!(
                    //     "有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     name
                    // );
                    phi_list.push((name, inst_phi, false));
                } else {
                    // println!(
                    //     "没有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     name
                    // );
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
                // println!("指令{:?}插入bb{:?}中", inst_phi.get_kind(), bb.get_name());
                bb.as_mut().push_front(inst_phi);
                // self.context_mut
                //     .update_var_scope(name.as_str(), inst_phi, bb.get_name());
                if let Some(inst_map) = self.context_mut.bb_map.get_mut(bb.get_name()) {
                    inst_map.insert(name.clone(), inst_phi);
                } else {
                    let mut map = HashMap::new();
                    map.insert(name.clone(), inst_phi);
                    self.context_mut
                        .bb_map
                        .insert(bb.get_name().to_string(), map);
                }
                if let Some((phi_list, is_padded)) =
                    self.context_mut.phi_list.get_mut(bb.get_name())
                {
                    //如果有philist
                    // println!(
                    //     "有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     bb.get_name()
                    // );
                    phi_list.push((name, inst_phi, false));
                } else {
                    // println!(
                    //     "没有philist,插入phi{:?}进入philist{:?}",
                    //     inst_phi.get_kind(),
                    //     bb.get_name()
                    // );
                    //如果没有,生成新的philist,插入
                    let mut vec = vec![];
                    vec.push((name, inst_phi, false));
                    self.context_mut
                        .phi_list
                        .insert(bb.get_name().to_string(), (vec, false));
                }
                Ok(inst_phi)
            }
            _=>{todo!()}
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

    // pub fn get_var_symbol()

    pub fn get_var(
        &mut self,
        s: &str,
        offset: Option<ObjPtr<Inst>>,
        bool_get_ptr: bool,
    ) -> Result<(ObjPtr<Inst>, Symbol), Error> {
        // let bb = self.context_mut.bb_now_mut;
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
                let mut bbname = "notinblock";

                let inst_opt = self
                    .context_mut
                    .bb_map
                    .get(bbname)
                    .and_then(|var_inst_map| var_inst_map.get(&name_changed));

                if let Some(sym) = sym_opt {
                    // println!("找到变量{:?}",s);
                    return Ok((inst, sym));
                }
                // InfuncChoice::NInFunc() => {
                //     return todo!();;
                // }
            }
        }
        // println!("没找到变量:{:?}",s);
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

        // let mut is_const = false;

        if layer_var == -1 {
            bbname = "notinblock"; //全局变量,const和一般类型需要分开处理吗?
        }
        // else if layer_var == 0 {
        //     // bbname = "params";
        //     if let Some(is_used) = self.context_mut.param_usage_table.get(&name_changed) {
        //         if !is_used {
        //             // println!("进来了");
        //             bbname = "params";
        //         }

        //     }
        // }//函数参数不再单独处理

        let inst_opt = self
            .context_mut
            .bb_map
            .get(bbname)
            .and_then(|var_inst_map| var_inst_map.get(&name_changed));

        if let Some(sym) = sym_opt {
            // println!("进来了");
            // println!("找到变量{:?}",s);

            //应该先判断是不是数组，以防bbmap中找不到报错
            if let Some(inst_array) = sym.array_inst {
                // println!("是数组{:?}",inst_array.get_ir_type());
                // println!("找到数组变量{:?},不插phi", s);
                // println!("bbname:{:?}", bbname);
                //如果是数组
                let mut inst_ret = self.pool_inst_mut.make_int_const(-1129);
                match sym.tp {
                    Type::Float | Type::ConstFloat => {
                        //判断类型
                        if layer_var < 0 {
                            //是否是全局
                            if let Some(offset) = offset {
                                //有偏移
                                // let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                // inst_ret = self.pool_inst_mut.make_global_float_array_load(ptr);
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                                                                               // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                               // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                               // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                                                              // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_float_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let ptr = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    // println!("var_name:{:?},ptr类型:{:?}",s,ptr.get_ir_type());
                                    // match self.context_mut.bb_now_mut {
                                    //     InfuncChoice::InFunc(bb) =>{
                                    //         println!("所在bb{:?}",bb.get_name());
                                    //     }
                                    //     _=>{}
                                    // }
                                    inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                        // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                        // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
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
                                                                                                  // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                  // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                  // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_offset_temp);
                                    // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                            }
                            //  else {
                            //     //没偏移
                            //     let ptr =
                            //         self.pool_inst_mut.make_global_float_array_load(inst_array);
                            //     inst_ret = self.pool_inst_mut.make_gep(ptr, );
                            //     self.context_mut.push_inst_bb(inst_ret);
                            // }
                        } else {
                            //不是全局
                            if let Some(offset) = offset {
                                //有偏移
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    inst_ret = self.pool_inst_mut.make_gep(inst_array, offset); //获得特定指针
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    // println!("var_name:{:?},arrayinst类型:{:?}",s,inst_array.get_ir_type());
                                    match self.context_mut.bb_now_mut {
                                        InfuncChoice::InFunc(bb) =>{
                                            // println!("所在bb{:?}",bb.get_name());
                                        }
                                        _=>{}
                                    }
                                    let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                    // println!("var_name:{:?},ptr类型:{:?}",s,ptr.get_ir_type());
                                    match self.context_mut.bb_now_mut {
                                        InfuncChoice::InFunc(bb) =>{
                                            // println!("所在bb{:?}",bb.get_name());
                                        }
                                        _=>{}
                                    }
                                    inst_ret = self.pool_inst_mut.make_float_load(ptr);
                                    // inst_ret = self.pool_inst_mut.make_gep(inst_array, offset);
                                    // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                if bool_get_ptr {
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(inst_array, inst_offset_temp); //获得特定指针
                                                                                                   // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                   // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                   // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(inst_offset_temp); //哪些不插入到块中?
                                                                                     // self.context_mut.push_inst_bb(ptr);
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
                                // let ptr = self.pool_inst_mut.make_gep(inst_array, offset);
                                // inst_ret = self.pool_inst_mut.make_global_float_array_load(ptr);
                                if bool_get_ptr {
                                    //如果是需要取指针(向函数传递数组指针)
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                                                                               // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                               // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                               // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                                                              // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    let ptr_array =
                                        self.pool_inst_mut.make_global_int_array_load(inst_array); //获得数组第一个元素(全局变量元素都是指针)
                                    let ptr = self.pool_inst_mut.make_gep(ptr_array, offset); //获得特定指针
                                    inst_ret = self.pool_inst_mut.make_int_load(ptr); //获得元素值
                                                                                      // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                      // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
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
                                                                                                  // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                  // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                  // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr_array); //哪些不插入到块中?
                                    self.context_mut.push_inst_bb(inst_offset_temp);
                                    // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                            }
                            //  else {
                            //     //没偏移
                            //     let ptr =
                            //         self.pool_inst_mut.make_global_float_array_load(inst_array);
                            //     inst_ret = self.pool_inst_mut.make_gep(ptr, );
                            //     self.context_mut.push_inst_bb(inst_ret);
                            // }
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
                                    // inst_ret = self.pool_inst_mut.make_gep(inst_array, offset);
                                    // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                }
                            } else {
                                if bool_get_ptr {
                                    let inst_offset_temp = self.pool_inst_mut.make_int_const(0);
                                    inst_ret =
                                        self.pool_inst_mut.make_gep(inst_array, inst_offset_temp); //获得特定指针
                                                                                                   // inst_ret = self.pool_inst_mut.make_float_load(ptr); //获得元素值
                                                                                                   // inst_ret = self.pool_inst_mut.make_gep(ptr_array, offset);
                                                                                                   // self.context_mut.push_inst_bb(offset); //这里需要向bb插入offset吗
                                    self.context_mut.push_inst_bb(inst_offset_temp); //哪些不插入到块中?
                                                                                     // self.context_mut.push_inst_bb(ptr);
                                    self.context_mut.push_inst_bb(inst_ret);
                                } else {
                                    unreachable!("没给偏移")
                                }
                                //没偏移
                            }
                        }
                    }
                    _=>{todo!()}
                }
                return Some((inst_ret, sym));
            } else {
                //不是数组的情况
                if layer_var < 0 {
                    // println!("找到全局变量{:?},不插phi", s);
                    // println!("bbname:{:?}", bbname);
                    //全局也肯定能找到
                    //如果是全局变量
                    // let mut inst_ret = self.pool_inst_mut.make_int_const(-1129);
                    // bbname = "notinblock";//全局变量,const和一般类型需要分开处理吗?
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

                            // return Some((inst_load, sym));
                        }
                        Type::ConstInt | Type::Int => {
                            if let Some(inst_global) = sym.global_inst {
                                let inst_ret = self.pool_inst_mut.make_global_int_load(inst_global);
                                self.context_mut.push_inst_bb(inst_ret); //这里
                                return Some((inst_ret, sym));
                            }
                            // return Some((inst_load, sym));
                        }
                        _=>{todo!()}
                    }
                } else {
                    if let Some(inst) = inst_opt {
                        // println!("找到变量{:?},不插phi", s);
                        // println!("bbname:{:?}", bbname);
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
                                                                             // return Some((inst_load, sym));
                                }
                                Type::ConstInt | Type::Int => {
                                    inst_ret = self.pool_inst_mut.make_global_int_load(inst_ret);
                                    self.context_mut.push_inst_bb(inst_ret);
                                    // return Some((inst_load, sym));
                                }
                                _=>{todo!()}
                            }
                        }

                        return Some((inst_ret, sym));
                    } else {
                        // println!("没找到变量{:?},插phi", s);
                        // println!("bbname:{:?}", bbname);

                        //没找到
                        // bb.as_ref().
                        match sym.tp {
                            Type::ConstFloat | Type::Float => {
                                let phi_inst = self
                                    .push_phi(name_changed.clone(), Type::Float, bb)
                                    .unwrap();
                                // if let Some(vec) = self.context_mut.phi_list.get_mut(bbname) {
                                //     //有philist,直接加入philist中
                                //     vec.push((name_changed.clone(), phi_inst));
                                // } else {
                                //     //该bb没有philist,新建philist,加入philist中
                                //     let mut v = vec![];
                                //     v.push((name_changed.clone(), phi_inst));
                                //     self.context_mut.phi_list.insert(bbname.to_string(), v);
                                // }
                                return Some((phi_inst, sym));
                            }
                            Type::ConstInt | Type::Int => {
                                let phi_inst =
                                    self.push_phi(name_changed.clone(), Type::Int, bb).unwrap();

                                // let phi_inst = self.push_phi(s.to_string(), Type::Int, bb).unwrap();

                                // if let Some(vec) = self.context_mut.phi_list.get_mut(bbname) {
                                //     //有philist,直接加入philist中
                                //     vec.push((name_changed.clone(), phi_inst));
                                // } else {
                                //     //该bb没有philist,新建philist,加入philist中
                                //     let mut v = vec![];
                                //     v.push((name_changed.clone(), phi_inst));
                                //     self.context_mut.phi_list.insert(bbname.to_string(), v);
                                // }
                                return Some((phi_inst, sym));
                            }
                            _=>{todo!()}
                        }

                        // let phiinst_mut = phiinst.as_mut();
                        // let bb_mut = bb.as_mut();
                        // for preccessor in bb_mut.get_up_bb() {
                        //     if let Some((temp, symbol)) = self.get_var_bb(s, *preccessor) {
                        //         phiinst_mut.add_operand(temp);
                        //     }
                        // }
                        // return Option::Some((phiinst, sym));
                    }
                }
            }
        }
        Option::None
    }
}
